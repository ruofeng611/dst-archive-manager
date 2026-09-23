//! 模组查询接口。
//!
//! 列表只读目录树（快）；模组名称需要执行 `modinfo.lua`（慢），单独成接口，
//! 并按 `(mod_id, locale)` + 目录 mtime 走 `mod_cache` 缓存。

use crate::service::{AppSetting, ModCacheService, SettingService};
use crate::support::JsonResponse;
use crate::utils::{app_component, decode_file_content};
use crate::vo::{ModListVO, ModNameVO};
use crate::{ConstantComponent, json_response_wrap};
use mlua::{Lua, Value};
use serde::Deserialize;
use simple_starter_core::tracing::error;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tauri_macros::auto_command;

/// DST 相关目录布局
#[derive(Default)]
struct ModDirs {
    /// 服务端安装目录
    server_install: Option<PathBuf>,
    /// Steam 创意工坊目录（.../steamapps/workshop）
    workshop: Option<PathBuf>,
}

impl ModDirs {
    /// 创意工坊 content 目录（.../steamapps/workshop/content/<dst_steam_id>）
    fn workshop_content(&self, cc: &ConstantComponent) -> Option<PathBuf> {
        self.workshop
            .as_ref()
            .map(|w| w.join(&cc.content_folder_name).join(&cc.dst_steam_id))
            .filter(|p| p.is_dir())
    }
}

/// 定位 DST 相关目录：只认配置页里的设置（自动发现见 `discover_paths_handler`）
fn resolve_mod_dirs(setting: &AppSetting) -> ModDirs {
    ModDirs {
        server_install: setting
            .dst_server_path
            .clone()
            .map(PathBuf::from)
            .filter(|p| p.is_dir()),
        workshop: setting
            .workshop_path
            .clone()
            .map(PathBuf::from)
            .filter(|p| p.is_dir()),
    }
}

/// 目录名归一化为模组 id（数字目录补 `workshop-` 前缀）
fn normalize_mod_id(cc: &ConstantComponent, folder_name: &str) -> String {
    if folder_name.chars().all(|c| c.is_ascii_digit()) {
        format!("{}{}", cc.workshop_folder_prefix, folder_name)
    } else {
        folder_name.to_string()
    }
}

/// 枚举本地可用模组：`id -> 目录`（服务端 mods 优先，其次创意工坊 content）
fn collect_mod_dirs(cc: &ConstantComponent, dirs: &ModDirs) -> BTreeMap<String, PathBuf> {
    let mut mods: BTreeMap<String, PathBuf> = BTreeMap::new();

    // 服务端安装目录下的 mods/
    if let Some(server) = &dirs.server_install {
        let mods_dir = server.join(&cc.mods_folder_name);
        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let folder_name = entry.file_name().to_string_lossy().to_string();
                // 只认 workshop-N 与其纯数字形式，避免把无关目录当成模组
                if folder_name.starts_with(&cc.workshop_folder_prefix)
                    || folder_name.chars().all(|c| c.is_ascii_digit())
                {
                    mods.insert(normalize_mod_id(&cc, &folder_name), entry.path());
                }
            }
        }
    }

    // 创意工坊 content/<appid>/ 下的数字目录
    if let Some(content) = dirs.workshop_content(cc) {
        if let Ok(entries) = fs::read_dir(&content) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                mods.entry(normalize_mod_id(cc, &entry.file_name().to_string_lossy()))
                    .or_insert_with(|| entry.path());
            }
        }
    }

    mods
}

/// 本地可用模组列表
#[auto_command]
pub async fn list_mods_handler() -> JsonResponse {
    json_response_wrap!(function_name = "获取模组列表", {
        let cc = app_component::<ConstantComponent>()?;
        let setting_service: Arc<SettingService> = app_component()?;
        let setting = setting_service.get().await?;

        let dirs = resolve_mod_dirs(&setting);

        let mut result = ModListVO::default();
        if dirs.server_install.is_none() {
            return Ok(result);
        }

        result.found = true;
        result.mod_ids = collect_mod_dirs(&cc, &dirs).into_keys().collect();

        Ok(result)
    })
}

/// 解析模组名称请求
#[derive(Deserialize)]
pub struct ResolveModNamesRequest {
    pub mod_ids: Vec<String>,
}

/// 按需解析模组名称（执行 modinfo.lua，命中缓存则跳过）
#[auto_command]
pub async fn resolve_mod_names_handler(request: ResolveModNamesRequest) -> JsonResponse {
    json_response_wrap!(function_name = "解析模组名称", {
        let cc = app_component::<ConstantComponent>()?;
        let setting_service: Arc<SettingService> = app_component()?;
        let cache_service: Arc<ModCacheService> = app_component()?;

        let setting = setting_service.get().await?;
        let dirs = resolve_mod_dirs(&setting);
        let index = collect_mod_dirs(&cc, &dirs);
        let locale = setting_service.language().await?;

        let mut names = Vec::with_capacity(request.mod_ids.len());
        for mod_id in request.mod_ids {
            let name = match index.get(&mod_id) {
                // 本地没有该模组：不缓存，避免它后续被下载后仍拿到旧值
                None => None,
                Some(dir) => resolve_name(&cc, &cache_service, &locale, &mod_id, dir).await?,
            };
            names.push(ModNameVO { mod_id, name });
        }

        Ok(names)
    })
}

/// 取模组名：目录 mtime 未变化时直接用缓存，否则执行 modinfo.lua 并写回缓存
async fn resolve_name(
    cc: &ConstantComponent,
    cache_service: &ModCacheService,
    locale: &str,
    mod_id: &str,
    mod_dir: &Path,
) -> Result<Option<String>, crate::SimpleAppWebError> {
    let mtime = dir_mtime(mod_dir);

    if let Some((name, cached_mtime)) = cache_service.get(mod_id, locale).await? {
        if cached_mtime == mtime {
            return Ok(name);
        }
    }

    let name = extract_name_from_modinfo(cc, mod_dir, locale);
    cache_service
        .set(mod_id, locale, name.as_deref(), mtime)
        .await?;

    Ok(name)
}

/// 模组目录的修改时间（Unix 秒）；读取失败返回 0，等效于每次重新解析
fn dir_mtime(dir: &Path) -> i64 {
    fs::metadata(dir)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 从 modinfo.lua 中提取 name 字段（通过执行 Lua 脚本）
fn extract_name_from_modinfo(cc: &ConstantComponent, mod_path: &Path, locale: &str) -> Option<String> {
    let modinfo_path = mod_path.join(&cc.modinfo_file_name);
    if !modinfo_path.exists() {
        return None;
    }

    // 读取文件为字节，然后尝试多种编码解析
    let bytes = match fs::read(&modinfo_path) {
        Ok(b) => b,
        Err(e) => {
            error!("读取 {} 失败: {}", cc.modinfo_file_name, e);
            return None;
        }
    };

    let content = decode_file_content(&bytes);

    match extract_name_via_lua(&content, &modinfo_path, locale) {
        Some(name) => Some(name),
        None => {
            error!(
                "无法从 {} 中提取 name 字段: {}",
                cc.modinfo_file_name,
                modinfo_path.display()
            );
            None
        }
    }
}

/// 使用 Lua 解释器执行 modinfo.lua 脚本，从中提取 name 全局变量的值
/// 模拟 DST 游戏环境：设置 locale 全局变量（DST 加载 modinfo.lua 前会预置此变量）
/// 模组脚本内部根据 locale 自行推导语言（如 L / CH / lang 等），无需我们逐一伪造
fn extract_name_via_lua(content: &str, modinfo_path: &Path, locale: &str) -> Option<String> {
    let lua = Lua::new();
    let file_name = modinfo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("modinfo.lua");

    // 模拟 DST 环境：设置 locale 全局变量，让 Lua 脚本内部自行推导语言
    let globals = lua.globals();
    if let Err(e) = globals.set("locale", locale) {
        error!("设置 Lua 全局变量 locale 失败: {}", e);
    }

    // 执行 Lua 脚本
    if let Err(e) = lua.load(content).set_name(file_name).exec() {
        error!("执行 {} 脚本失败: {}", file_name, e);
        return None;
    }

    // 从全局变量中提取 name 值
    let name_value: Value = match globals.get("name") {
        Ok(v) => v,
        Err(e) => {
            error!("读取 Lua 全局变量 name 失败: {}", e);
            return None;
        }
    };

    match name_value {
        Value::String(s) => match s.to_str() {
            Ok(rs) if !rs.is_empty() => Some(rs.to_string()),
            Ok(_) => None,
            Err(e) => {
                error!("Lua 字符串转换失败: {}", e);
                None
            }
        },
        Value::Nil => {
            error!("{} 中 name 字段为 nil", file_name);
            None
        }
        // 如果 name 是其他类型（如数字），尝试转为字符串
        other => {
            let result = other.to_string().unwrap_or_default();
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        }
    }
}
