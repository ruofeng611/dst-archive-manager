//! 存档查询与配置接口。
//!
//! 读接口（list_* / get_*）只读磁盘，不做任何写入或删除；
//! 写动作（更新配置、删除存档、清理不一致存档）各自独立成接口。

use crate::service::{ServerRuntimeService, SettingService};
use crate::support::JsonResponse;
use crate::support::SimpleAppWebError;
use crate::utils::app_component;
use crate::utils::decode_file_content;
use crate::utils::find_user_dir;
use crate::utils::{read_token, write_token};
use crate::vo::{ArchiveListVO, ArchivePhaseVO, ArchiveSummaryVO, ServerDetailVO};
use crate::{ConstantComponent, json_response_wrap};
use ini::Ini;
use mlua::{Lua, Value};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri_macros::auto_command;

// ==================== 列表查询（只读，无副作用）====================

/// 存档根目录：只认配置页里的设置（自动发现见 `discover_paths_handler`）
fn resolve_root(configured: Option<String>) -> Option<PathBuf> {
    configured.map(PathBuf::from).filter(|p| p.is_dir())
}

/// 收集指定目录下带前缀的子目录摘要
fn collect_summaries(
    cc: &ConstantComponent,
    dir: &Path,
    prefix: &str,
    statuses: &HashMap<String, String>,
) -> Result<Vec<ArchiveSummaryVO>, SimpleAppWebError> {
    let mut items = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if id.starts_with(prefix) {
            items.push(parse_summary(cc, &entry.path(), &id, statuses)?);
        }
    }

    Ok(items)
}

/// 本地存档（Cluster_X）列表
#[auto_command]
pub async fn list_clusters_handler() -> JsonResponse {
    json_response_wrap!(function_name = "获取本地存档列表", {
        let cc = app_component::<ConstantComponent>()?;
        let setting_service: Arc<SettingService> = app_component()?;

        let mut result = ArchiveListVO::default();
        let Some(root) = resolve_root(setting_service.archive_root().await?) else {
            return Ok(result);
        };
        result.found = true;
        result.path = root.to_string_lossy().into_owned();

        // Cluster_X 位于根目录下唯一的数字用户目录内
        let Some(user_dir) = find_user_dir(&root)? else {
            return Ok(result);
        };

        let runtime_service: Arc<ServerRuntimeService> = app_component()?;
        let statuses = runtime_service.status_map().await?;
        result.items = collect_summaries(&cc, &user_dir, &cc.dst_cluster_prefix, &statuses)?;
        Ok(result)
    })
}

/// 专用服务器（Server_X）列表
#[auto_command]
pub async fn list_servers_handler() -> JsonResponse {
    json_response_wrap!(function_name = "获取服务器列表", {
        let cc = app_component::<ConstantComponent>()?;
        let setting_service: Arc<SettingService> = app_component()?;

        let mut result = ArchiveListVO::default();
        let Some(root) = resolve_root(setting_service.archive_root().await?) else {
            return Ok(result);
        };
        result.found = true;
        result.path = root.to_string_lossy().into_owned();

        let runtime_service: Arc<ServerRuntimeService> = app_component()?;
        let statuses = runtime_service.status_map().await?;
        result.items = collect_summaries(&cc, &root, &cc.dst_server_prefix, &statuses)?;
        Ok(result)
    })
}

// ==================== 详情查询（只读，无副作用）====================

/// 服务器详情（配置 + mod_ids + 运行状态，不含阶段列表）
#[auto_command]
pub async fn get_server_handler(id: String) -> JsonResponse {
    json_response_wrap!(function_name = "获取服务器详情", {
        let cc = app_component::<ConstantComponent>()?;
        let setting_service: Arc<SettingService> = app_component()?;

        let server_path = setting_service.resolve_archive_path(&id).await?;
        if !server_path.is_dir() {
            return Err(SimpleAppWebError::new(
                404,
                format!("服务器目录不存在: {}", server_path.display()),
            ));
        }

        let ini = parse_cluster_ini(&cc, &server_path);
        let runtime_service: Arc<ServerRuntimeService> = app_component()?;

        Ok(ServerDetailVO {
            id: id.clone(),
            game_mode: ini.game_mode,
            max_players: ini.max_players,
            pvp: ini.pvp,
            pause_when_empty: ini.pause_when_empty,
            cluster_password: ini.cluster_password,
            cluster_description: ini.cluster_description,
            cluster_name: ini.cluster_name,
            has_caves: server_path.join(&cc.caves_shard_name).is_dir(),
            // 运行状态来自 server_runtime 表（未受管视为 stopped）
            status: runtime_service
                .status(&id)
                .await?
                .unwrap_or_else(|| "stopped".to_string()),
            mod_ids: parse_level_data_override(&server_path)?,
            // 以该服务器目录下的 cluster_token.txt 为准
            cluster_token: read_token(&server_path.join(&cc.cluster_token_file)),
        })
    })
}

/// 服务器存档阶段列表
#[auto_command]
pub async fn get_server_saves_handler(id: String) -> JsonResponse {
    json_response_wrap!(function_name = "获取服务器存档列表", {
        let setting_service: Arc<SettingService> = app_component()?;

        let server_path = setting_service.resolve_archive_path(&id).await?;
        if !server_path.is_dir() {
            return Err(SimpleAppWebError::new(
                404,
                format!("服务器目录不存在: {}", server_path.display()),
            ));
        }

        Ok(parse_archive_phases(&server_path)?)
    })
}

// ==================== 解析辅助 ====================

/// cluster.ini 中的世界配置
#[derive(Default)]
struct ClusterIni {
    game_mode: String,
    max_players: u32,
    pvp: bool,
    pause_when_empty: bool,
    cluster_password: String,
    cluster_description: String,
    cluster_name: String,
}

/// 解析 cluster.ini；文件或字段缺失时保留默认值
fn parse_cluster_ini(cc: &ConstantComponent, folder_path: &Path) -> ClusterIni {
    let mut vo = ClusterIni::default();

    let Ok(ini) = Ini::load_from_file(folder_path.join(&cc.cluster_ini_file)) else {
        return vo;
    };

    if let Some(gameplay) = ini.section(Some("GAMEPLAY")) {
        if let Some(mode) = gameplay.get("game_mode") {
            vo.game_mode = mode.trim().to_string();
        }
        if let Some(players) = gameplay
            .get("max_players")
            .and_then(|v| v.trim().parse::<u32>().ok())
        {
            vo.max_players = players;
        }
        if let Some(pvp) = gameplay.get("pvp") {
            vo.pvp = pvp.trim().to_lowercase() == "true";
        }
        if let Some(pause) = gameplay.get("pause_when_empty") {
            vo.pause_when_empty = pause.trim().to_lowercase() == "true";
        }
    }

    if let Some(network) = ini.section(Some("NETWORK")) {
        if let Some(password) = network.get("cluster_password") {
            vo.cluster_password = password.trim().to_string();
        }
        if let Some(description) = network.get("cluster_description") {
            vo.cluster_description = description.trim().to_string();
        }
        if let Some(name) = network.get("cluster_name") {
            vo.cluster_name = name.trim().to_string();
        }
    }

    vo
}

/// 列表项摘要：只读 cluster.ini、Caves 目录与最新一个 .meta
fn parse_summary(
    cc: &ConstantComponent,
    folder_path: &Path,
    id: &str,
    statuses: &HashMap<String, String>,
) -> Result<ArchiveSummaryVO, SimpleAppWebError> {
    Ok(ArchiveSummaryVO {
        id: id.to_string(),
        cluster_name: parse_cluster_ini(cc, folder_path).cluster_name,
        has_caves: folder_path.join(&cc.caves_shard_name).is_dir(),
        // 运行状态来自 server_runtime 表（不受管的服务器视为 stopped）
        status: statuses
            .get(id)
            .cloned()
            .unwrap_or_else(|| "stopped".to_string()),
        latest_phase: parse_latest_phase(cc, folder_path)?,
    })
}

/// .meta 三个字段的正则（一次构建、多次解析）
struct MetaRegexes {
    phase: Regex,
    cycles: Regex,
    season: Regex,
}

impl MetaRegexes {
    fn new() -> Result<Self, SimpleAppWebError> {
        Ok(Self {
            phase: Regex::new(r#"phase\s*=\s*"([^"]+)"#)?,
            cycles: Regex::new(r#"cycles\s*=\s*(\d+)"#)?,
            season: Regex::new(r#"season\s*=\s*"([^"]+)"#)?,
        })
    }

    fn parse(&self, meta_path: &Path, phase_file_name: String) -> Option<ArchivePhaseVO> {
        let content = fs::read_to_string(meta_path).ok()?;

        Some(ArchivePhaseVO {
            now_phase: self
                .phase
                .captures(&content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            cycles: self
                .cycles
                .captures(&content)
                .and_then(|c| c[1].parse::<u32>().ok())
                .unwrap_or(0),
            season: self
                .season
                .captures(&content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            phase_file_name,
            // 一致性由 parse_archive_phases 统一判定
            mismatched: false,
        })
    }
}

/// Master 分片的 session 目录
fn master_session_dir(cc: &ConstantComponent, folder_path: &Path) -> PathBuf {
    folder_path
        .join(&cc.master_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name)
}

/// session 目录下编号最大的 .meta 文件
fn find_latest_meta(session_path: &Path) -> Option<(u64, PathBuf, String)> {
    let mut latest: Option<(u64, PathBuf, String)> = None;

    let Ok(entries) = fs::read_dir(session_path) else {
        return None;
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension() != Some(OsStr::new("meta")) {
            continue;
        }
        let base_name = entry
            .file_name()
            .to_string_lossy()
            .trim_end_matches(".meta")
            .to_string();
        let num = base_name.parse::<u64>().unwrap_or(0);

        if latest.as_ref().is_none_or(|(n, _, _)| num > *n) {
            latest = Some((num, path, base_name));
        }
    }

    latest
}

/// 列表用：只解析最新一个存档的阶段信息
fn parse_latest_phase(
    cc: &ConstantComponent,
    folder_path: &Path,
) -> Result<Option<ArchivePhaseVO>, SimpleAppWebError> {
    let Some(session_path) = get_session_path(&master_session_dir(cc, folder_path))? else {
        return Ok(None);
    };

    let Some((_, meta_path, base_name)) = find_latest_meta(&session_path) else {
        return Ok(None);
    };

    Ok(MetaRegexes::new()?.parse(&meta_path, base_name))
}

/// 收集 session 目录下的 .meta 文件：(编号, 路径, 文件名不含扩展)
fn collect_meta_files(session_path: &Path) -> Vec<(u64, PathBuf, String)> {
    let mut files = Vec::new();

    let Ok(entries) = fs::read_dir(session_path) else {
        return files;
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension() != Some(OsStr::new("meta")) {
            continue;
        }
        let base_name = entry
            .file_name()
            .to_string_lossy()
            .trim_end_matches(".meta")
            .to_string();
        let num = base_name.parse::<u64>().unwrap_or(0);
        files.push((num, path, base_name));
    }

    files
}

/// 解析存档阶段列表（按编号降序：最新在前）
///
/// 编号取地面与洞穴两侧的**并集**，展示字段优先用地面侧的 `.meta`；
/// `mismatched` 标记只存在于单侧的存档（任一侧没有 session 时不标记，
/// 与 `sync_archive_files` 的清理条件保持一致）。本接口只读，不删除任何文件。
fn parse_archive_phases(folder_path: &Path) -> Result<Vec<ArchivePhaseVO>, SimpleAppWebError> {
    let cc = app_component::<ConstantComponent>()?;

    let Some(master_session_path) = get_session_path(&master_session_dir(&cc, folder_path))? else {
        return Ok(Vec::new());
    };
    let master_files = collect_meta_files(&master_session_path);

    let caves_session_dir = folder_path
        .join(&cc.caves_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name);
    let caves_files = match get_session_path(&caves_session_dir)? {
        Some(path) => collect_meta_files(&path),
        None => Vec::new(),
    };

    let master_numbers: BTreeSet<u64> = master_files.iter().map(|(number, _, _)| *number).collect();
    let caves_numbers: BTreeSet<u64> = caves_files.iter().map(|(number, _, _)| *number).collect();
    // 两侧都有 session 才谈得上"不一致"（单侧无 session 时清理逻辑也不会动）
    let comparable = !master_numbers.is_empty() && !caves_numbers.is_empty();

    // 同编号时地面侧覆盖洞穴侧，展示字段跟随地面
    let mut by_number: BTreeMap<u64, (PathBuf, String)> = caves_files
        .into_iter()
        .map(|(number, path, name)| (number, (path, name)))
        .collect();
    for (number, path, name) in master_files {
        by_number.insert(number, (path, name));
    }

    let regexes = MetaRegexes::new()?;
    let mut phases = Vec::with_capacity(by_number.len());
    for (number, (path, base_name)) in by_number.into_iter().rev() {
        let Some(mut phase) = regexes.parse(&path, base_name) else {
            continue;
        };
        phase.mismatched =
            comparable && !(master_numbers.contains(&number) && caves_numbers.contains(&number));
        phases.push(phase);
    }

    Ok(phases)
}

// ==================== 写操作 ====================

/// 服务器配置更新请求
///
/// 按 patch 语义：字段为 `None` 表示不修改（对应的 cluster.ini 项也不会被写入）。
#[derive(Deserialize)]
pub struct UpdateServerConfigRequest {
    pub server_id: String,
    pub max_players: Option<u32>,
    pub pvp: Option<bool>,
    pub pause_when_empty: Option<bool>,
    pub cluster_password: Option<String>,
    pub cluster_description: Option<String>,
    pub cluster_name: Option<String>,
    /// 该服务器的 cluster token：不传表示不修改；传空串表示回填配置页的全局令牌
    pub token: Option<String>,
}

/// 更新服务器配置接口
#[auto_command]
pub async fn update_server_config_handler(config: UpdateServerConfigRequest) -> JsonResponse {
    json_response_wrap!(function_name = "更新服务器配置", {
        let setting_service: Arc<SettingService> = app_component()?;
        let constant_component: Arc<ConstantComponent> = app_component()?;

        // 推导服务器路径
        let server_path = setting_service
            .resolve_archive_path(&config.server_id)
            .await?;

        let ini_path = server_path.join(&constant_component.cluster_ini_file);

        // 只在传了 cluster.ini 相关字段时才读写 ini（例如"设置为全局 token"只提交 token）
        if config.max_players.is_some()
            || config.pvp.is_some()
            || config.pause_when_empty.is_some()
            || config.cluster_password.is_some()
            || config.cluster_description.is_some()
            || config.cluster_name.is_some()
        {
            // 读取现有的 ini 文件
            let mut ini = Ini::load_from_file(&ini_path)?;

            {
                // 更新 [GAMEPLAY] 部分
                let mut gameplay = ini.with_section(Some("GAMEPLAY"));
                if let Some(max_players) = config.max_players {
                    gameplay.set("max_players", max_players.to_string());
                }
                if let Some(pvp) = config.pvp {
                    gameplay.set("pvp", pvp.to_string());
                }
                if let Some(pause_when_empty) = config.pause_when_empty {
                    gameplay.set("pause_when_empty", pause_when_empty.to_string());
                }
            }

            {
                // 更新 [NETWORK] 部分
                let mut network = ini.with_section(Some("NETWORK"));
                if let Some(cluster_name) = &config.cluster_name {
                    network.set("cluster_name", cluster_name);
                }
                if let Some(description) = &config.cluster_description {
                    network.set("cluster_description", description);
                }
                if let Some(password) = &config.cluster_password {
                    network.set("cluster_password", password);
                }
            }

            ini.write_to_file(&ini_path)?;
        }

        // token：以服务器目录下的 cluster_token.txt 为准（数据库不存服务器令牌）
        if let Some(token) = &config.token {
            let token = token.trim();
            let token_file = server_path.join(&constant_component.cluster_token_file);

            if token.is_empty() {
                // 留空 = 回填配置页的全局令牌
                let global = setting_service
                    .get()
                    .await?
                    .global_cluster_token
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        SimpleAppWebError::new(
                            400,
                            "未配置全局 cluster token：请在设置页填写".to_string(),
                        )
                    })?;
                write_token(&token_file, &global)?;
            } else {
                write_token(&token_file, token)?;
            }
        }

        Ok(())
    })
}

/// 清理 Master/Caves 不一致的存档
///
/// 只存在于一个分片的存档无法加载，这里删除它们以保持两分片一致。
/// 删除磁盘文件只发生在这个接口里：查询接口（list_* / get_*）只读。
#[auto_command]
pub async fn repair_server_saves_handler(id: String) -> JsonResponse {
    json_response_wrap!(function_name = "清理不一致存档", {
        let setting_service: Arc<SettingService> = app_component()?;

        let server_path = setting_service.resolve_archive_path(&id).await?;
        if !server_path.is_dir() {
            return Err(SimpleAppWebError::new(
                404,
                format!("服务器目录不存在: {}", server_path.display()),
            ));
        }

        sync_archive_files(&server_path)?;
        Ok(())
    })
}

/// 同步 Master 和 Caves 的存档文件，删除不一致的存档
fn sync_archive_files(folder_path: &Path) -> Result<(), SimpleAppWebError> {
    let cc = app_component::<ConstantComponent>()?;
    let master_session_dir = master_session_dir(&cc, folder_path);
    let caves_session_dir = folder_path
        .join(&cc.caves_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name);

    // 如果 Caves 目录不存在，则无需同步
    if !caves_session_dir.exists() {
        return Ok(());
    }

    // 获取 Master 的 session 子目录
    let Some(master_session_path) = get_session_path(&master_session_dir)? else {
        return Ok(());
    };

    // 获取 Caves 的 session 子目录
    let Some(caves_session_path) = get_session_path(&caves_session_dir)? else {
        return Ok(());
    };

    // 获取 Master 和 Caves 的所有存档文件编号
    let master_archives = get_archive_numbers(&master_session_path)?;
    let caves_archives = get_archive_numbers(&caves_session_path)?;

    // 找出不一致的存档（只存在于其中一个目录的）
    let master_only: Vec<_> = master_archives
        .iter()
        .filter(|n| !caves_archives.contains(n))
        .collect();
    let caves_only: Vec<_> = caves_archives
        .iter()
        .filter(|n| !master_archives.contains(n))
        .collect();

    // 删除 Master 中多余的存档
    for num in master_only {
        delete_archive_files(&master_session_path, num)?;
    }

    // 删除 Caves 中多余的存档
    for num in caves_only {
        delete_archive_files(&caves_session_path, num)?;
    }

    Ok(())
}

/// 获取 session 目录下的唯一子目录路径
fn get_session_path(session_dir: &Path) -> Result<Option<PathBuf>, SimpleAppWebError> {
    if !session_dir.is_dir() {
        return Ok(None);
    }

    let session_sub_dirs: Vec<_> = fs::read_dir(session_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .collect();

    if session_sub_dirs.len() == 1 {
        Ok(Some(session_sub_dirs[0].path()))
    } else {
        Ok(None)
    }
}

/// 获取指定目录下所有存档文件的编号
fn get_archive_numbers(session_path: &Path) -> Result<Vec<String>, SimpleAppWebError> {
    let mut numbers = Vec::new();

    if let Ok(entries) = fs::read_dir(session_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("meta")) {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                let base_name = file_name_str.trim_end_matches(".meta");
                numbers.push(base_name.to_string());
            }
        }
    }

    Ok(numbers)
}

/// 删除指定编号的存档文件（包括 .meta 和纯数字文件）
fn delete_archive_files(session_path: &Path, number: &str) -> Result<(), SimpleAppWebError> {
    let meta_file = session_path.join(format!("{}.meta", number));
    let data_file = session_path.join(number);

    // 删除 .meta 文件
    if meta_file.exists() {
        fs::remove_file(&meta_file)?;
    }

    // 删除纯数字文件
    if data_file.exists() {
        fs::remove_file(&data_file)?;
    }

    Ok(())
}

/// 删除存档阶段请求
#[derive(Deserialize)]
pub struct DeleteArchivePhaseRequest {
    pub server_id: String,
    pub phase_file_name: String, // 存档文件名，如 "0000007"
}

/// 删除存档阶段接口
#[auto_command]
pub async fn delete_archive_phase_handler(request: DeleteArchivePhaseRequest) -> JsonResponse {
    json_response_wrap!(function_name = "删除存档阶段", {
        let setting_service: Arc<SettingService> = app_component()?;

        // 推导服务器路径
        let server_path = setting_service
            .resolve_archive_path(&request.server_id)
            .await?;

        let cc = app_component::<ConstantComponent>()?;

        // 删除 Master 中的存档文件
        if let Some(master_session_path) = get_session_path(&master_session_dir(&cc, &server_path))? {
            delete_archive_files(&master_session_path, &request.phase_file_name)?;
        }

        // 删除 Caves 中的存档文件（如果存在）
        let caves_session_dir = server_path
            .join(&cc.caves_shard_name)
            .join(&cc.save_folder_name)
            .join(&cc.session_folder_name);
        if let Some(caves_session_path) = get_session_path(&caves_session_dir)? {
            delete_archive_files(&caves_session_path, &request.phase_file_name)?;
        }

        Ok(())
    })
}

/// 解析 modoverrides.lua 文件，通过执行 Lua 脚本提取模组ID列表
/// 返回 workshop-XXX 格式的模组ID列表
fn parse_level_data_override(folder_path: &Path) -> Result<Vec<String>, SimpleAppWebError> {
    let mut mod_ids = Vec::new();

    let cc = app_component::<ConstantComponent>()?;
    let level_data_path = folder_path
        .join(&cc.master_shard_name)
        .join(&cc.modoverrides_file_name);

    // 如果文件不存在，返回空列表
    if !level_data_path.exists() {
        return Ok(mod_ids);
    }

    // 读取文件字节并用 decode_file_content 解码（处理 GBK 等非 UTF-8 编码）
    let bytes = fs::read(&level_data_path)?;
    let content = decode_file_content(&bytes);

    // 使用 Lua 解释器执行 modoverrides.lua 脚本
    let lua = Lua::new();

    // 执行脚本并获取返回值（modoverrides.lua 返回一个 Table）
    let result: Value = match lua
        .load(&content)
        .set_name(&cc.modoverrides_file_name)
        .eval()
    {
        Ok(v) => v,
        Err(_) => return Ok(mod_ids), // 执行失败返回空列表
    };

    // 遍历返回的 Table，提取所有 workshop-xxx 格式的键
    if let Value::Table(table) = result {
        if let Ok(pairs) = table
            .pairs()
            .collect::<Result<Vec<(Value, Value)>, _>>()
        {
            for (key, _value) in pairs {
                if let Value::String(s) = key {
                    if let Ok(key_str) = s.to_str() {
                        if key_str.starts_with(&cc.workshop_folder_prefix) {
                            mod_ids.push(key_str.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(mod_ids)
}
