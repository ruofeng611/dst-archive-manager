//! Steam 安装路径与库路径发现（Windows 注册表 + libraryfolders.vdf）。

use crate::ConstantComponent;
use regex::Regex;
use simple_starter_core::tracing::info;
use std::fs;
use std::path::PathBuf;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

/// 获取 Steam 主安装路径（先读注册表，失败时退回默认安装位置）
pub fn get_steam_install_path(cc: &ConstantComponent) -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam_key) = hkcu.open_subkey(&cc.steam_registry_key) {
        if let Ok(path_str) = steam_key.get_value::<String, _>(&cc.steam_path_value_name) {
            // 注册表里的路径通常是 forward slash (/)，Rust PathBuf 能处理
            let path = PathBuf::from(path_str);
            if path.exists() {
                info!("Steam path find by hkcu: {}", path.display());
                return Some(path);
            }
        }
    }

    let default = PathBuf::from(&cc.steam_default_install_path);
    if default.exists() {
        info!("Steam path find by fallback: {}", default.display());
        return Some(default);
    }

    None
}

/// 解析 libraryfolders.vdf 获取所有 Steam 库路径
pub fn get_all_steam_libraries(cc: &ConstantComponent) -> Vec<PathBuf> {
    let mut libraries = Vec::new();

    let Some(base_path) = get_steam_install_path(cc) else {
        return libraries; // 连 Steam 都找不到，返回空
    };

    // 主路径本身也是一个库（通常包含 steamapps）
    libraries.push(base_path.clone());

    let vdf_path = base_path
        .join(&cc.steamapps_folder_name)
        .join(&cc.library_folders_file);
    if let Ok(content) = fs::read_to_string(&vdf_path) {
        // VDF 格式示例: "path"		"D:\\SteamLibrary"
        let re = Regex::new(r#"(?i)"path"\s+"(.+?)""#).unwrap();

        for cap in re.captures_iter(&content) {
            if let Some(matched) = cap.get(1) {
                // VDF 里的路径是双反斜杠转义的；Steam 有时混用 \\ 和 /
                let path_str = matched.as_str().replace("\\\\", "\\");
                let lib_path = PathBuf::from(path_str);
                if lib_path.exists() {
                    libraries.push(lib_path);
                }
            }
        }
    }

    // 去重
    libraries.sort();
    libraries.dedup();

    libraries
}
