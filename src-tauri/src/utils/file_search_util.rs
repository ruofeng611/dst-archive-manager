use regex::Regex;
use simple_starter_core::tracing::info;
use std::fs;
use std::path::PathBuf;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

/// 获取 Steam 主安装路径 (通过注册表)
pub fn get_steam_install_path() -> Option<PathBuf> {
    // 尝试读取 HKEY_CURRENT_USER\Software\Valve\Steam 的 SteamPath
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam_key) = hkcu.open_subkey("Software\\Valve\\Steam") {
        if let Ok(path_str) = steam_key.get_value::<String, _>("SteamPath") {
            // Steam 注册表里的路径通常是 forward slash (/)，Rust PathBuf 能处理
            let path = PathBuf::from(path_str);
            if path.exists() {
                info!("Steam path find by hkcu: {}", path.display());
                return Some(path);
            }
        }
    }

    // 注册表失败时的常见默认路径 Fallback
    let default = PathBuf::from(r"C:\Program Files (x86)\Steam");
    if default.exists() {
        info!("Steam path find by fallback: {}", default.display());
        return Some(default);
    }

    None
}

/// 解析 libraryfolders.vdf 获取所有 Steam 库路径
pub fn get_all_steam_libraries() -> Vec<PathBuf> {
    let mut libraries = Vec::new();

    // 1. 获取主路径
    let base_path = match get_steam_install_path() {
        Some(p) => p,
        None => return vec![], // 连 Steam 都找不到，返回空
    };

    // 主路径本身也是一个库（通常包含 steamapps）
    libraries.push(base_path.clone());

    // 2. 读取 library folders.vdf
    let vdf_path = base_path.join("steamapps").join("libraryfolders.vdf");
    if vdf_path.exists() {
        if let Ok(content) = fs::read_to_string(vdf_path) {
            // 使用正则提取 "path" "..." 字段
            // VDF 格式示例: "path"		"D:\\SteamLibrary"
            let re = Regex::new(r#"(?i)"path"\s+"(.+?)""#).unwrap();

            for cap in re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    // VDF 里的路径是双反斜杠转义的，通常 Regex 提取出来就是正常的字符串
                    // 注意：Steam 有时混用 \\ 和 /
                    let path_str = matched.as_str().replace("\\\\", "\\");
                    let lib_path = PathBuf::from(path_str);
                    if lib_path.exists() {
                        libraries.push(lib_path);
                    }
                }
            }
        }
    }

    // 去重
    libraries.sort();
    libraries.dedup();

    libraries
}
