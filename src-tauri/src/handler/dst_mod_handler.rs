use crate::service::{
    ArchivePathService, KEY_DST_CLIENT_PATH, KEY_DST_SERVER_PATH, KEY_STEAM_WORKSHOP_PATH,
};
use crate::support::JsonResponse;
use crate::utils::get_all_steam_libraries;
use crate::vo::{ModInfoVO, ModScanVO};
use crate::{ConstantComponent, SimpleAppWebError, json_response_wrap};
use regex::Regex;
use simple_starter_core::AppCoreUtil;
use simple_starter_core::tracing::error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri_macros::auto_command;

#[auto_command]
pub async fn scan_dst_mods_handler(custom_path: Option<String>) -> JsonResponse {
    json_response_wrap!(function_name = "获取模组信息", {
        let mut result = ModScanVO::default();
        let mut target_server_path: Option<PathBuf> = None;
        let mut target_client_path: Option<PathBuf> = None;
        let mut target_library_root: Option<PathBuf> = None; // 记录是哪个库找到的，方便找 Workshop

        let constant_component: Arc<ConstantComponent> = AppCoreUtil::get_component()?;
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        // 1. 确定 Dedicated Server 路径和 Client 路径
        if let Some(p) = custom_path {
            // A. 用户传入了自定义路径
            let pb = PathBuf::from(p);
            if pb.exists() {
                if let Some(folder_name) = pb.file_name().and_then(|n| n.to_str()) {
                    if folder_name == &constant_component.dst_server_file_name {
                        target_server_path = Some(pb);
                        // .../steamapps/common路径
                        let common_path = target_server_path
                            .as_ref()
                            .and_then(|p| p.parent()) // -> common
                            .map(|p| p.to_path_buf());

                        target_client_path = common_path
                            .as_ref()
                            .and_then(|p| Some(p.join(&constant_component.dst_client_file_name)))
                            .map(|p| p.to_path_buf());

                        // 尝试反推 Library root 以便查找 Workshop
                        // 假设结构: .../steamapps/common/Don't Starve Together Dedicated Server
                        target_library_root = target_server_path
                            .as_ref()
                            .and_then(|p| p.parent()) // -> common
                            .and_then(|p| p.parent()) // -> steamapps
                            .and_then(|p| p.parent()) // -> Library Root
                            .map(|p| p.to_path_buf());
                    }
                }
            }
        } else {
            // B. 智能扫描 Steam 库
            let libraries = get_all_steam_libraries();
            let server_folder_name = &constant_component.dst_server_file_name;
            let client_folder_name = &constant_component.dst_client_file_name;

            for lib in &libraries {
                // 拼接服务器标准路径: LibraryPath/steamapps/common/Folder
                let server_candidate = lib
                    .join("steamapps")
                    .join("common")
                    .join(server_folder_name);
                if server_candidate.exists() {
                    target_server_path = Some(server_candidate);
                    target_library_root = Some(lib.clone());
                }

                // 拼接客户端标准路径: LibraryPath/steamapps/common/ClientFolder
                let client_candidate = lib
                    .join("steamapps")
                    .join("common")
                    .join(client_folder_name);
                if client_candidate.exists() {
                    target_client_path = Some(client_candidate);
                }

                // 如果都找到了就停止
                if target_server_path.is_some() && target_client_path.is_some() {
                    break;
                }
            }
        }

        // 2. 如果找到了 Server 路径，开始填充数据
        if let Some(ref server_path) = target_server_path {
            result.found = true;
            result.server_path = server_path.to_string_lossy().to_string();

            // 保存 Steam 库路径和 DST 服务器路径到 key-value 表
            if let Some(ref lib_root) = target_library_root {
                let steamapps_workshop = lib_root.join("steamapps").join("workshop");
                if steamapps_workshop.exists() {
                    let _ = archive_path_service
                        .set_value(
                            KEY_STEAM_WORKSHOP_PATH,
                            &steamapps_workshop.to_string_lossy(),
                        )
                        .await;
                }
            }
            let _ = archive_path_service
                .set_value(KEY_DST_SERVER_PATH, &server_path.to_string_lossy())
                .await;

            // 保存客户端路径
            if let Some(ref client_path) = target_client_path {
                let _ = archive_path_service
                    .set_value(KEY_DST_CLIENT_PATH, &client_path.to_string_lossy())
                    .await;
            }

            // 2.1 扫描服务器端 mods (server/mods/workshop*)
            let server_mods_path = server_path.join("mods");
            if server_mods_path.exists() {
                scan_mods_directory(&server_mods_path, &mut result.mods)?;
            }

            // 2.2 扫描 Steam Workshop (322330)
            // 路径: LibraryRoot/steamapps/workshop/content/322330
            if let Some(lib_root) = target_library_root {
                let workshop_path = lib_root
                    .join("steamapps")
                    .join("workshop")
                    .join("content")
                    .join(&constant_component.dst_steam_id);

                if workshop_path.exists() {
                    scan_workshop_directory(&workshop_path, &mut result.mods)?;
                }
            }
        } else {
            result.found = false;
        }

        Ok(result)
    })
}

/// 扫描 mods 目录下的 workshop* 文件夹
fn scan_mods_directory(
    mods_path: &PathBuf,
    mods: &mut Vec<ModInfoVO>,
) -> Result<(), SimpleAppWebError> {
    for entry in fs::read_dir(mods_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();

        if entry.file_type()?.is_dir() && name.starts_with("workshop") {
            let mod_path = entry.path();
            // 只有存在 modinfo.lua 文件时才添加到列表
            if let Some(mod_name) = extract_mod_name_from_modinfo(&mod_path) {
                mods.push(ModInfoVO {
                    folder_name: name,
                    mod_name,
                });
            }
        }
    }
    Ok(())
}

/// 扫描 Steam Workshop 目录下的数字文件夹
fn scan_workshop_directory(
    workshop_path: &PathBuf,
    mods: &mut Vec<ModInfoVO>,
) -> Result<(), SimpleAppWebError> {
    for entry in fs::read_dir(workshop_path)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            let folder_name = entry.file_name().to_string_lossy().to_string();
            let mod_path = entry.path();
            // 只有存在 modinfo.lua 文件时才添加到列表
            if let Some(mod_name) = extract_mod_name_from_modinfo(&mod_path) {
                mods.push(ModInfoVO {
                    folder_name,
                    mod_name,
                });
            }
        }
    }
    Ok(())
}

/// 从 modinfo.lua 文件中提取 name 字段
fn extract_mod_name_from_modinfo(mod_path: &PathBuf) -> Option<String> {
    let modinfo_path = mod_path.join("modinfo.lua");

    if !modinfo_path.exists() {
        return None;
    }

    // 读取文件为字节，然后尝试多种编码解析
    let bytes = match fs::read(&modinfo_path) {
        Ok(b) => b,
        Err(e) => {
            error!("读取 modinfo.lua 失败: {}", e);
            return None;
        }
    };

    // 尝试多种编码解析文件内容
    let content = decode_file_content(&bytes);

    // 优先匹配 xxx and "..." or "..." 格式（xxx可以是L、CH等任意标识符），提取 or 后面的部分
    let complex_re =
        Regex::new(r#"name\s*=\s*\w+\s+and\s+["'][^"']*["']\s+or\s+["']([^"']+)["']"#).ok()?;
    if let Some(caps) = complex_re.captures(&content) {
        if let Some(m) = caps.get(1) {
            let result = m.as_str().to_string();
            return Some(result);
        }
    }

    // 如果不是复杂格式，使用原有的简单匹配 name = "XXX" 或 name='XXX'
    let simple_re = Regex::new(r#"name\s*=\s*["']([^"']+)["']"#).ok()?;
    if let Some(caps) = simple_re.captures(&content) {
        if let Some(m) = caps.get(1) {
            let result = m.as_str().to_string();
            return Some(result);
        }
    }

    None
}

/// 尝试多种编码解析文件内容
fn decode_file_content(bytes: &[u8]) -> String {
    // 尝试的编码列表：UTF-8, GBK(Windows简体中文), GB18030, Latin1
    let encodings = [
        encoding_rs::UTF_8,
        encoding_rs::GBK,
        encoding_rs::GB18030,
        encoding_rs::WINDOWS_1252, // Latin1/Windows西欧
    ];

    for encoding in &encodings {
        let (cow, _, had_errors) = encoding.decode(bytes);
        if !had_errors {
            return cow.into_owned();
        }
    }

    // 如果所有编码都失败，使用 UTF-8 lossy 解码（替换非法字符为 �）
    String::from_utf8_lossy(bytes).into_owned()
}
