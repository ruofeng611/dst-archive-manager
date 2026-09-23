use crate::service::{
    STATUS_STARTING, STATUS_STOPPING, ServerRuntimeService, SettingService,
};
use crate::support::JsonResponse;
use crate::utils::{find_and_send_shutdown_command, find_window_by_title};
use crate::utils::{read_token, write_token};
use crate::{ConstantComponent, SimpleAppWebError, json_response_wrap};
use serde::Deserialize;
use crate::utils::app_component;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::fs;
use tauri_macros::auto_command;

#[derive(Deserialize)]
pub struct ConvertClusterRequest {
    pub cluster_id: String,
    pub has_caves: bool, // 是否有洞穴
}

/// 将 Cluster 转换为 Server
///
/// token 取配置页的全局令牌，只写入 `cluster_token.txt`；**不**为该服务器创建覆盖值
/// （覆盖值只由用户在服务器配置里显式填写才产生，留空即回退全局）。
#[auto_command]
pub async fn convert_cluster_to_server_handler(request: ConvertClusterRequest) -> JsonResponse {
    json_response_wrap!(function_name = "转换为服务器", {
        let constant_component: Arc<ConstantComponent> = app_component()?;
        let setting_service: Arc<SettingService> = app_component()?;

        // 1. 推导 Cluster 的完整路径
        let cluster_path_buf = setting_service
            .resolve_archive_path(&request.cluster_id)
            .await?;

        // 验证路径存在
        if !cluster_path_buf.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("Cluster path does not exist: {}", cluster_path_buf.display()),
            ));
        }

        // 2. 读取配置（根目录、全局令牌、服务端目录一次读齐）
        let setting = setting_service.get().await?;

        let dst_dir_str = setting.archive_root.as_deref().ok_or_else(|| {
            SimpleAppWebError::new(400, "未配置存档根目录：请在设置页配置".to_string())
        })?;
        let dst_dir = PathBuf::from(dst_dir_str);

        let token = setting
            .global_cluster_token
            .as_deref()
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                SimpleAppWebError::new(400, "未配置全局 cluster token：请在设置页填写".to_string())
            })?;

        let dst_server_path = setting.dst_server_path.as_deref().ok_or_else(|| {
            SimpleAppWebError::new(400, "未配置服务端安装目录：请在设置页配置".to_string())
        })?;

        // 3. 扫描 DoNotStarveTogether 目录获取已有的最大 Server 编号，新编号为最大+1
        let server_number = get_next_folder_number_from_fs(&dst_dir, &constant_component.dst_server_prefix)?;
        let server_id = format!("{}{}", constant_component.dst_server_prefix, server_number);
        let server_path = dst_dir.join(&server_id);

        // 4. 复制 Cluster 文件夹到新的 Server 位置
        copy_dir_all(&cluster_path_buf, &server_path)?;

        // 5. 全局令牌落盘（只写文件；服务器是否覆盖由文件本身表达）
        let token_file_path = server_path.join(&constant_component.cluster_token_file);
        write_token(&token_file_path, token)?;

        // 6. 生成 StartServer.bat 文件
        let bat_content = generate_start_server_bat(
            &constant_component,
            &server_id,
            dst_server_path,
            setting.workshop_path.as_deref(),
            request.has_caves,
        );
        let bat_file_path = server_path.join(&constant_component.start_server_bat_name);
        fs::write(&bat_file_path, bat_content)?;

        Ok(server_id)
    })
}

#[derive(Deserialize)]
pub struct ConvertServerToClusterRequest {
    pub server_id: String,
}

/// 将 Server 转为本地 Cluster 存档
#[auto_command]
pub async fn convert_server_to_cluster_handler(request: ConvertServerToClusterRequest) -> JsonResponse {
    json_response_wrap!(function_name = "转为本地存档", {
        let constant_component: Arc<ConstantComponent> = app_component()?;
        let setting_service: Arc<SettingService> = app_component()?;

        // 1. 推导 Server 的完整路径
        let server_path_buf = setting_service
            .resolve_archive_path(&request.server_id)
            .await?;
        if !server_path_buf.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("Server path does not exist: {}", server_path_buf.display()),
            ));
        }

        // 2. 获取用户存档目录（DoNotStarveTogether/{用户ID}，Cluster 存放位置）
        let user_dir = setting_service.user_dir().await?.ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                "DST user directory not found. Please scan archives first.".to_string(),
            )
        })?;

        // 3. 扫描获取已有的最大 Cluster 编号，新编号为最大+1
        let cluster_number = get_next_folder_number_from_fs(&user_dir, &constant_component.dst_cluster_prefix)?;
        let cluster_id = format!("{}{}", constant_component.dst_cluster_prefix, cluster_number);
        let cluster_path = user_dir.join(&cluster_id);

        // 4. 复制 Server 文件夹到新的 Cluster 位置
        copy_dir_all(&server_path_buf, &cluster_path)?;

        Ok(cluster_id)
    })
}

/// 删除服务器
#[auto_command]
pub async fn delete_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "删除服务器", {
        let setting_service: Arc<SettingService> = app_component()?;

        // 1. 推导 Server 的完整路径
        let server_path_buf = setting_service
            .resolve_archive_path(&server_id)
            .await?;

        // 2. 删除整个 Server 文件夹
        if server_path_buf.exists() {
            fs::remove_dir_all(&server_path_buf).map_err(|e| {
                SimpleAppWebError::new(500, format!("Failed to delete server directory: {}", e))
            })?;
        }

        // 3. 移除运行记录（若该服务器正被监视）
        let runtime_service: Arc<ServerRuntimeService> = app_component()?;
        runtime_service.remove(&server_id).await?;

        Ok(())
    })
}

/// 同步客户端模组到服务器（供外部调用的命令）
#[auto_command]
pub async fn sync_client_mods_to_server_handler() -> JsonResponse {
    json_response_wrap!(function_name = "同步模组", {
        let setting_service: Arc<SettingService> = app_component()?;
        move_client_mods_to_server(&setting_service).await?;
        Ok(())
    })
}

/// 将客户端模组移动到服务器模组目录
async fn move_client_mods_to_server(
    setting_service: &SettingService,
) -> Result<(), SimpleAppWebError> {
    let setting = setting_service.get().await?;

    // 获取客户端路径
    let client_path = setting.dst_client_path.ok_or_else(|| {
        SimpleAppWebError::new(400, "未配置客户端安装目录：请在设置页配置".to_string())
    })?;

    // 获取服务器路径
    let server_path = setting.dst_server_path.ok_or_else(|| {
        SimpleAppWebError::new(400, "未配置服务端安装目录：请在设置页配置".to_string())
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
fn generate_start_server_bat(
    cc: &ConstantComponent,
    server_id: &str,
    dst_server_path: &str,
    workshop_path: Option<&str>,
    has_caves: bool,
) -> String {
    // 构建 bin 目录路径
    let bin_path = PathBuf::from(dst_server_path).join("bin");
    let workshop_path = workshop_path.unwrap_or_default();

    // 生成批处理文件内容
    let mut bat_content = format!(
        r#"@ECHO OFF

set SteamAppId={}

set SteamGameId={}

set COMMON_UGC_PATH="{}"

cd /D "{}"

start "{}_{}" dontstarve_dedicated_server_nullrenderer.exe -console -cluster {} -shard {} -ugc_directory %COMMON_UGC_PATH%
"#,
        cc.dst_steam_id,
        cc.dst_steam_id,
        workshop_path,
        bin_path.to_string_lossy(),
        server_id,
        cc.master_shard_name,
        server_id,
        cc.master_shard_name
    );

    // 如果有洞穴，添加洞穴服务器启动命令
    if has_caves {
        bat_content.push_str(&format!(r#"
start "{}_{}" dontstarve_dedicated_server_nullrenderer.exe -console -cluster {} -shard {} -ugc_directory %COMMON_UGC_PATH%
"#,
            server_id,
            cc.caves_shard_name,
            server_id,
            cc.caves_shard_name
        ));
    }

    bat_content
}

/// 启动服务器
#[auto_command]
pub async fn start_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "启动服务器", {
        let setting_service: Arc<SettingService> = app_component()?;
        let cc = app_component::<ConstantComponent>()?;

        // 1. 推导 Server 的完整路径
        let server_path_buf = setting_service
            .resolve_archive_path(&server_id)
            .await?;

        // 验证路径存在
        if !server_path_buf.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!("Server path does not exist: {}", server_path_buf.display()),
            ));
        }

        // 2. 检查 StartServer.bat 文件是否存在
        let bat_file_path = server_path_buf.join(&cc.start_server_bat_name);
        if !bat_file_path.exists() {
            return Err(SimpleAppWebError::new(
                404,
                format!(
                    "{} not found in: {}",
                    cc.start_server_bat_name,
                    server_path_buf.display()
                ),
            ));
        }

        // 3. 检查服务器是否已在运行
        let master_title = format!("{}_{}", server_id, cc.master_shard_name);
        let caves_title = format!("{}_{}", server_id, cc.caves_shard_name);
        let master_running = find_window_by_title(&master_title);
        let caves_running = find_window_by_title(&caves_title);

        if master_running || caves_running {
            return Err(SimpleAppWebError::new(
                409,
                format!("Server is already running: {}", server_id),
            ));
        }

        // 4. 前置校验：读取该服务器目录下的 cluster_token.txt
        let token_file = server_path_buf.join(&cc.cluster_token_file);
        if read_token(&token_file).is_none() {
            return Err(SimpleAppWebError::new(
                400,
                "该服务器缺少 cluster_token.txt（或内容为空）：请在服务器配置里填写令牌".to_string(),
            ));
        }

        // 5. 执行 StartServer.bat 文件
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

        // 6. 登记运行状态：崩档监视只接管本应用启动过的服务器
        //    洞穴分片是否期望存在，按 Caves 目录是否存在判断（与详情页的 has_caves 一致）
        let runtime_service: Arc<ServerRuntimeService> = app_component()?;
        let expect_caves = server_path_buf.join(&cc.caves_shard_name).is_dir();
        runtime_service
            .upsert(&server_id, STATUS_STARTING, expect_caves)
            .await?;

        Ok(())
    })
}

/// 停止服务器
#[auto_command]
pub async fn stop_server_handler(server_id: String) -> JsonResponse {
    json_response_wrap!(function_name = "停止服务器", {
        let cc = app_component::<ConstantComponent>()?;
        let master_title = format!("{}_{}", server_id, cc.master_shard_name);
        let caves_title = format!("{}_{}", server_id, cc.caves_shard_name);

        // 标记为关闭中：崩档监视据此不做"崩档"判定，只等窗口消失后移除记录
        let runtime_service: Arc<ServerRuntimeService> = app_component()?;
        runtime_service.set_status(&server_id, STATUS_STOPPING).await?;

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

/// 语言设置请求
#[derive(Deserialize)]
pub struct SetLanguageRequest {
    pub language: String, // "zh" 或 "en"
}

/// 设置应用语言
#[auto_command]
pub async fn set_app_language_handler(request: SetLanguageRequest) -> JsonResponse {
    json_response_wrap!(function_name = "设置语言", {
        let setting_service: Arc<SettingService> = app_component()?;
        setting_service.set_language(&request.language).await?;
        Ok(())
    })
}

/// 获取应用语言
#[auto_command]
pub async fn get_app_language_handler() -> JsonResponse {
    json_response_wrap!(function_name = "获取语言", {
        let setting_service: Arc<SettingService> = app_component()?;
        let language = setting_service.language().await?;
        Ok(language)
    })
}

/// 在 Windows 资源管理器中打开指定存档的文件夹
#[auto_command]
pub async fn open_archive_in_folder_handler(id: String) -> JsonResponse {
    json_response_wrap!(function_name = "在文件夹中打开", {
        let setting_service: Arc<SettingService> = app_component()?;
        let path = setting_service.resolve_archive_path(&id).await?;
        // 在 Windows 资源管理器中打开文件夹
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| SimpleAppWebError::new(500, format!("Failed to open folder: {}", e)))?;
        Ok(())
    })
}

/// 扫描指定目录下所有带指定前缀的文件夹，返回最大编号 + 1
/// 例如存在 Cluster_1、Cluster_3 → 返回 4
fn get_next_folder_number_from_fs(
    dst_dir: &std::path::Path,
    prefix: &str,
) -> Result<i32, SimpleAppWebError> {
    let mut max_num = 0;
    if let Ok(entries) = fs::read_dir(dst_dir) {
        for entry in entries.flatten() {
            if let Some(folder_name) = entry.file_name().to_str() {
                if let Some(num_str) = folder_name.strip_prefix(prefix) {
                    if let Ok(num) = num_str.parse::<i32>() {
                        if num > max_num {
                            max_num = num;
                        }
                    }
                }
            }
        }
    }
    Ok(max_num + 1)
}
