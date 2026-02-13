use crate::service::{
    ArchivePathService, KEY_DST_CLIENT_PATH, KEY_DST_SERVER_PATH, KEY_STEAM_WORKSHOP_PATH,
};
use crate::support::JsonResponse;
use crate::utils::find_window_by_title;
use crate::{ConstantComponent, SimpleAppWebError, json_response_wrap};
use serde::Deserialize;
use simple_starter_core::AppCoreUtil;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};
use tauri_macros::auto_command;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    SendInput, VIRTUAL_KEY, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow};
use windows::core::{HSTRING, PCWSTR};

static START_SERVER_BAT_NAME: &str = "StartServer.bat";

#[derive(Deserialize)]
pub struct ConvertClusterRequest {
    pub cluster_id: String,
    pub token: String,
    pub has_caves: bool, // 是否有洞穴
}

/// 将 Cluster 转换为 Server
#[auto_command]
pub async fn convert_cluster_to_server_handler(request: ConvertClusterRequest) -> JsonResponse {
    json_response_wrap!(function_name = "转换为服务器", {
        let constant_component: Arc<ConstantComponent> = AppCoreUtil::get_component()?;
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        // 1. 获取 Cluster 的完整路径
        let cluster_path = archive_path_service
            .get_path_by_id(&request.cluster_id)
            .await?
            .ok_or_else(|| {
                SimpleAppWebError::new(
                    404,
                    format!("Cluster path not found: {}", request.cluster_id),
                )
            })?;

        let cluster_path_buf = PathBuf::from(&cluster_path);

        // 验证路径存在
        if !cluster_path_buf.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("Cluster path does not exist: {}", cluster_path),
            ));
        }

        // 2. 获取 DoNotStarveTogether 目录（Cluster 的父目录的父目录）
        let user_dir = cluster_path_buf.parent().ok_or_else(|| {
            SimpleAppWebError::new(500, "Cannot get parent directory".to_string())
        })?;
        let dst_dir = user_dir.parent().ok_or_else(|| {
            SimpleAppWebError::new(500, "Cannot get DoNotStarveTogether directory".to_string())
        })?;

        // 3. 获取下一个可用的 Server 编号
        let server_number = archive_path_service.get_next_server_number().await?;
        let server_id = format!("{}{}", constant_component.dst_server_prefix, server_number);
        let server_path = dst_dir.join(&server_id);

        // 4. 复制 Cluster 文件夹到新的 Server 位置
        copy_dir_all(&cluster_path_buf, &server_path)?;

        // 5. 在 Server 目录下创建 cluster_token.txt 文件并写入 token
        let token_file_path = server_path.join(&constant_component.cluster_token_file);
        fs::write(&token_file_path, &request.token)?;

        // 6. 生成 StartServer.bat 文件
        let bat_content = generate_start_server_bat(
            &server_id,
            &constant_component.dst_steam_id,
            &archive_path_service,
            request.has_caves,
        )
        .await?;
        let bat_file_path = server_path.join(START_SERVER_BAT_NAME);
        fs::write(&bat_file_path, bat_content)?;

        // 7. 保存 Server 路径到数据库
        archive_path_service
            .save_archive_path(
                server_id.clone(),
                "server".to_string(),
                server_path.to_string_lossy().into_owned(),
            )
            .await?;

        // 8. 将客户端模组移动到服务器模组目录
        move_client_mods_to_server(&archive_path_service).await?;

        Ok(server_id)
    })
}

/// 删除服务器
#[auto_command]
pub async fn delete_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "删除服务器", {
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        // 1. 获取 Server 的完整路径
        let server_path = archive_path_service
            .get_path_by_id(&server_id)
            .await?
            .ok_or_else(|| {
                SimpleAppWebError::new(404, format!("Server path not found: {}", server_id))
            })?;

        let server_path_buf = PathBuf::from(&server_path);

        // 2. 删除整个 Server 文件夹
        if server_path_buf.exists() {
            fs::remove_dir_all(&server_path_buf).map_err(|e| {
                SimpleAppWebError::new(500, format!("Failed to delete server directory: {}", e))
            })?;
        }

        // 3. 从数据库中删除路径记录
        archive_path_service.delete_path(&server_id).await?;

        Ok(())
    })
}

/// 同步客户端模组到服务器（供外部调用的命令）
#[auto_command]
pub async fn sync_client_mods_to_server_handler() -> JsonResponse {
    json_response_wrap!(function_name = "同步模组", {
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;
        move_client_mods_to_server(&archive_path_service).await?;
        Ok(())
    })
}

/// 将客户端模组移动到服务器模组目录
async fn move_client_mods_to_server(
    archive_path_service: &ArchivePathService,
) -> Result<(), SimpleAppWebError> {
    // 获取客户端路径
    let client_path = archive_path_service
        .get_value(KEY_DST_CLIENT_PATH)
        .await?
        .ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                "DST client path not found. Please scan mods first.".to_string(),
            )
        })?;

    // 获取服务器路径
    let server_path = archive_path_service
        .get_value(KEY_DST_SERVER_PATH)
        .await?
        .ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                "DST server path not found. Please scan mods first.".to_string(),
            )
        })?;

    let client_mods_path = PathBuf::from(&client_path).join("mods");
    let server_mods_path = PathBuf::from(&server_path).join("mods");

    // 检查客户端 mods 目录是否存在
    if !client_mods_path.exists() {
        return Ok(()); // 客户端没有 mods 目录，直接返回
    }

    // 确保服务器 mods 目录存在
    if !server_mods_path.exists() {
        fs::create_dir_all(&server_mods_path)?;
    }

    // 遍历客户端 mods 目录下的 workshop* 文件夹
    for entry in fs::read_dir(&client_mods_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if entry.file_type()?.is_dir() && name.starts_with("workshop") {
            let src_path = entry.path();
            let dst_path = server_mods_path.join(&file_name);

            // 如果服务器端已存在同名文件夹，先删除
            if dst_path.exists() {
                fs::remove_dir_all(&dst_path)?;
            }

            // 复制整个 workshop 文件夹到服务器
            copy_dir_all(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// 递归复制目录
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), SimpleAppWebError> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// 生成 StartServer.bat 文件内容
async fn generate_start_server_bat(
    server_id: &str,
    steam_app_id: &str,
    archive_path_service: &ArchivePathService,
    has_caves: bool,
) -> Result<String, SimpleAppWebError> {
    // 获取 DST 服务器路径和 Steam Workshop 路径
    let dst_server_path = archive_path_service
        .get_value(KEY_DST_SERVER_PATH)
        .await?
        .ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                "DST server path not found. Please scan mods first.".to_string(),
            )
        })?;

    let workshop_path = archive_path_service
        .get_value(KEY_STEAM_WORKSHOP_PATH)
        .await?
        .unwrap_or_default();

    // 构建 bin 目录路径
    let bin_path = PathBuf::from(&dst_server_path).join("bin");

    // 生成批处理文件内容
    let mut bat_content = format!(
        r#"@ECHO OFF

set SteamAppId={}
set SteamGameId={}

set COMMON_UGC_PATH="{}"

cd /D "{}"

start "{}_Master" dontstarve_dedicated_server_nullrenderer.exe -console -cluster {} -shard Master -ugc_directory %COMMON_UGC_PATH%
"#,
        steam_app_id,
        steam_app_id,
        workshop_path,
        bin_path.to_string_lossy(),
        server_id,
        server_id
    );

    // 如果有洞穴，添加洞穴服务器启动命令
    if has_caves {
        bat_content.push_str(&format!(r#"
start "{}_Caves" dontstarve_dedicated_server_nullrenderer.exe -console -cluster {} -shard Caves -ugc_directory %COMMON_UGC_PATH%
"#,
            server_id,
            server_id
        ));
    }

    Ok(bat_content)
}

/// 启动服务器
#[auto_command]
pub async fn start_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "启动服务器", {
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        // 1. 获取 Server 的完整路径
        let server_path = archive_path_service
            .get_path_by_id(&server_id)
            .await?
            .ok_or_else(|| {
                SimpleAppWebError::new(404, format!("Server path not found: {}", server_id))
            })?;

        let server_path_buf = PathBuf::from(&server_path);

        // 验证路径存在
        if !server_path_buf.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("Server path does not exist: {}", server_path),
            ));
        }

        // 2. 检查 StartServer.bat 文件是否存在
        let bat_file_path = server_path_buf.join(START_SERVER_BAT_NAME);
        if !bat_file_path.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("{} not found in: {}", START_SERVER_BAT_NAME, server_path),
            ));
        }

        // 3. 检查服务器是否已在运行
        let master_title = format!("{}_Master", server_id);
        let caves_title = format!("{}_Caves", server_id);
        let master_running = find_window_by_title(&master_title);
        let caves_running = find_window_by_title(&caves_title);

        if master_running || caves_running {
            return Err(SimpleAppWebError::new(
                409,
                format!("Server is already running: {}", server_id),
            ));
        }

        // 4. 执行 StartServer.bat 文件
        // 直接使用 cmd /C 来执行批处理文件，执行完毕后 cmd 窗口自动关闭
        let mut command = std::process::Command::new("cmd");
        command
            .args(["/C", bat_file_path.to_string_lossy().as_ref()])
            .current_dir(&server_path_buf)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x00000008); // DETACHED_PROCESS

        let _ = command
            .spawn()
            .map_err(|e| SimpleAppWebError::new(500, format!("Failed to start server: {}", e)))?;

        Ok(())
    })
}

/// 查询服务器状态
#[auto_command]
pub async fn query_server_status_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "查询服务器状态", {
        let master_title = format!("{}_Master", server_id);
        let caves_title = format!("{}_Caves", server_id);

        let master_running = find_window_by_title(&master_title);
        let caves_running = find_window_by_title(&caves_title);

        // 如果 Master 或 Caves 任一在运行，则认为服务器在运行
        let status = if master_running || caves_running {
            "running"
        } else {
            "stopped"
        };

        Ok(serde_json::json!({
            "server_id": server_id,
            "status": status,
            "master_running": master_running,
            "caves_running": caves_running
        }))
    })
}

/// 停止服务器
#[auto_command]
pub async fn stop_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "停止服务器", {
        let master_title = format!("{}_Master", server_id);
        let caves_title = format!("{}_Caves", server_id);

        // 查找并停止洞穴服务器（洞穴服务器可能不存在，不检查结果）
        let _ = find_and_send_shutdown_command(&caves_title);

        // 等待1秒后再关闭地面服务器
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 查找并停止地面服务器
        let master_stopped = find_and_send_shutdown_command(&master_title)?;

        if !master_stopped {
            return Err(SimpleAppWebError::new(
                404,
                format!("Master server window not found for: {}", server_id),
            ));
        }

        Ok(())
    })
}

/// 查找窗口并发送关闭命令
fn find_and_send_shutdown_command(window_title: &str) -> Result<bool, SimpleAppWebError> {
    unsafe {
        let wide_title = HSTRING::from(window_title);
        let hwnd: HWND = FindWindowW(None, PCWSTR::from_raw(wide_title.as_ptr()))?;

        if !hwnd.is_invalid() {
            // 找到窗口，激活它（必须激活才能接收输入）
            let _ = SetForegroundWindow(hwnd);

            // 等待窗口激活
            thread::sleep(Duration::from_millis(200));

            // 使用 SendInput 发送 Unicode 字符（模拟真实键盘输入）
            send_unicode_string(
                &AppCoreUtil::get_component::<ConstantComponent>()?.dst_end_command,
            );

            // 发送回车键
            send_key_press(VK_RETURN);

            return Ok(true);
        }
    }

    Ok(false)
}

/// 使用 SendInput 发送 Unicode 字符串
fn send_unicode_string(text: &str) {
    unsafe {
        for ch in text.chars() {
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let _ = SendInput(&[input], size_of::<INPUT>() as i32);

            // 发送按键释放
            let input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let _ = SendInput(&[input_up], size_of::<INPUT>() as i32);

            // 小延迟确保按键顺序
            thread::sleep(Duration::from_millis(10));
        }
    }
}

/// 发送虚拟键码按键（用于回车等功能键）
fn send_key_press(vk: VIRTUAL_KEY) {
    unsafe {
        // 按下
        let input_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let _ = SendInput(&[input_down], size_of::<INPUT>() as i32);

        // 释放
        let input_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let _ = SendInput(&[input_up], size_of::<INPUT>() as i32);
    }
}
