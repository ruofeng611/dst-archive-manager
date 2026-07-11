use crate::service::ArchivePathService;
use crate::service::{KEY_DST_ARCHIVE_DIR, KEY_DST_USER_DIR};
use crate::support::JsonResponse;
use crate::support::SimpleAppWebError;
use crate::utils::decode_file_content;
use crate::utils::find_window_by_title;
use crate::vo::{ArchiveDetailVO, ArchivePhaseVO, ArchiveScanVO};
use crate::{ConstantComponent, json_response_wrap};
use ini::Ini;
use mlua::{Lua, Value};
use regex::Regex;
use serde::Deserialize;
use simple_starter_core::AppCoreUtil;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri_macros::auto_command;

#[auto_command]
pub async fn scan_dst_archives_handler(custom_path: Option<String>) -> JsonResponse {
    json_response_wrap!(function_name = "获取存档信息", {
        let mut result = ArchiveScanVO::default();
        let constant_component: Arc<ConstantComponent> = AppCoreUtil::get_component()?;
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        let base_path = if let Some(p) = custom_path {
            let path = PathBuf::from(p);
            if let Some(last_segment) = path.file_name().and_then(|n| n.to_str()) {
                if last_segment != &constant_component.dst_archive_file_name {
                    return Ok(result);
                }
            } else {
                return Ok(result);
            }
            path
        } else {
            if let Some(doc) = dirs::document_dir() {
                doc.join(&constant_component.klei_folder_name)
                    .join(&constant_component.dst_archive_file_name)
            } else {
                return Ok(result);
            }
        };

        if !base_path.exists() || !base_path.is_dir() {
            return Ok(result);
        } else {
            result.found = true;
            // DoNotStarveTogether文件夹全路径
            result.path = base_path.to_string_lossy().into_owned();
            // 存储 dst 存档目录到 key-value 表，供后续转换接口使用
            let _ = archive_path_service
                .set_value(KEY_DST_ARCHIVE_DIR, &result.path)
                .await;
        }

        let mut all_clusters = Vec::new();
        let mut all_servers = Vec::new();

        // 遍历DoNotStarveTogether目录下的所有条目
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(folder_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        // 检查是否为纯数字文件夹（用户ID文件夹）
                        if folder_name.chars().all(|c| c.is_ascii_digit()) {
                            // 存储用户目录到 key-value 表，供后续 Server→Cluster 转换使用
                            let _ = archive_path_service
                                .set_value(KEY_DST_USER_DIR, &entry_path.to_string_lossy())
                                .await;
                            // 在数字文件夹下查找Cluster_X目录
                            if let Ok(cluster_entries) = fs::read_dir(&entry_path) {
                                for cluster_entry in cluster_entries.flatten() {
                                    let cluster_path = cluster_entry.path();
                                    if cluster_path.is_dir() {
                                        if let Some(cluster_name) =
                                            cluster_path.file_name().and_then(|n| n.to_str())
                                        {
                                            if cluster_name.starts_with(&constant_component.dst_cluster_prefix) {
                                                // 解析Cluster信息
                                                let cluster_detail = parse_archive_detail(
                                                    &cluster_path,
                                                    cluster_name,
                                                )?;

                                                // 保存路径到数据库
                                                let _ = archive_path_service
                                                    .save_archive_path(
                                                        cluster_name.to_string(),
                                                        "cluster".to_string(),
                                                        cluster_path.to_string_lossy().into_owned(),
                                                    )
                                                    .await;

                                                all_clusters.push(cluster_detail);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // 检查是否为Server_X目录（直接在DoNotStarveTogether下的服务器目录）
                        else if folder_name.starts_with(&constant_component.dst_server_prefix) {
                            let server_detail = parse_archive_detail(&entry_path, folder_name)?;

                            // 保存路径到数据库
                            let _ = archive_path_service
                                .save_archive_path(
                                    folder_name.to_string(),
                                    "server".to_string(),
                                    entry_path.to_string_lossy().into_owned(),
                                )
                                .await;

                            all_servers.push(server_detail);
                        }
                    }
                }
            }
        }

        result.clusters = all_clusters;
        result.servers = all_servers;

        Ok(result)
    })
}

/// 检查指定服务器是否正在运行（通过查找命令行窗口标题）
fn check_server_status(server_id: &str) -> Result<String, SimpleAppWebError> {
    let cc = AppCoreUtil::get_component::<ConstantComponent>()?;
    let found_master = find_window_by_title(&format!("{}_{}", server_id, cc.master_shard_name));
    let found_caves = find_window_by_title(&format!("{}_{}", server_id, cc.caves_shard_name));

    if found_master || found_caves {
        Ok("running".to_string())
    } else {
        Ok("stopped".to_string())
    }
}

/// 解析指定存档文件夹内的信息 (cluster.ini)
/// @param folder_path 存档文件夹路径即Cluster_X的全路径
fn parse_archive_detail(
    folder_path: &Path,
    id: &str,
) -> Result<ArchiveDetailVO, SimpleAppWebError> {
    let mut vo = ArchiveDetailVO::default();
    vo.id = id.to_string();

    // 构建 cluster.ini 路径
    let ini_path =
        folder_path.join(&AppCoreUtil::get_component::<ConstantComponent>()?.cluster_ini_file);

    // 使用 ini 库解析配置文件
    if let Ok(ini) = Ini::load_from_file(&ini_path) {
        // 解析 [GAMEPLAY] 部分
        if let Some(gameplay_section) = ini.section(Some("GAMEPLAY")) {
            // game_mode
            if let Some(mode) = gameplay_section.get("game_mode") {
                vo.game_mode = mode.trim().to_string();
            }
            // max_players
            if let Some(players_str) = gameplay_section.get("max_players") {
                if let Ok(players) = players_str.trim().parse::<u32>() {
                    vo.max_players = players;
                }
            }
            // pvp
            if let Some(pvp_str) = gameplay_section.get("pvp") {
                vo.pvp = pvp_str.trim().to_lowercase() == "true";
            }
            // pause_when_empty
            if let Some(pause_str) = gameplay_section.get("pause_when_empty") {
                vo.pause_when_empty = pause_str.trim().to_lowercase() == "true";
            }
        }

        // 解析 [NETWORK] 部分
        if let Some(network_section) = ini.section(Some("NETWORK")) {
            // cluster_password
            if let Some(password) = network_section.get("cluster_password") {
                vo.cluster_password = password.trim().to_string();
            }
            // cluster_description
            if let Some(description) = network_section.get("cluster_description") {
                vo.cluster_description = description.trim().to_string();
            }
            // cluster_name
            if let Some(name) = network_section.get("cluster_name") {
                vo.cluster_name = name.trim().to_string();
            }
        }
    }

    // 再解析Cluster_X\Master\save\session\6C1C2DF44161E003(随机的sessionID,似乎session文件夹下只会有一个这个文件)下的*.meta最大数字的那个
    vo.archive_phase_vo = parse_archive_phases(&folder_path)?;

    // 检测是否有洞穴（Caves 文件夹是否存在）
    let cc = AppCoreUtil::get_component::<ConstantComponent>()?;
    let caves_path = folder_path.join(&cc.caves_shard_name);
    vo.has_caves = caves_path.exists() && caves_path.is_dir();

    // 检测服务器运行状态（仅对 Server_X 类型的 ID）
    if id.starts_with(&cc.dst_server_prefix) {
        vo.status = check_server_status(id)?;
    } else {
        vo.status = "stopped".to_string();
    }

    // 解析存档使用的模组信息
    vo.mod_ids = parse_level_data_override(folder_path)?;

    Ok(vo)
}

/// 解析指定存档文件夹内的所有 .meta 文件，返回阶段信息列表（按编号降序：最新在前）
/// 在解析前会先同步 Master 和 Caves 的存档文件
fn parse_archive_phases(folder_path: &Path) -> Result<Vec<ArchivePhaseVO>, SimpleAppWebError> {
    let mut phases = Vec::new();

    let cc = AppCoreUtil::get_component::<ConstantComponent>()?;
    let session_dir = folder_path
        .join(&cc.master_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name);
    if !session_dir.is_dir() {
        return Ok(phases);
    }

    // 获取唯一的 session 子目录（如 6C1C2DF44161E003）
    let session_sub_dirs: Vec<_> = match fs::read_dir(&session_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect(),
        Err(_) => return Ok(phases),
    };

    if session_sub_dirs.len() != 1 {
        return Ok(phases);
    }

    let session_path = session_sub_dirs[0].path();

    // 在解析前先同步 Master 和 Caves 的存档文件
    sync_archive_files(folder_path)?;

    // 收集所有 .meta 文件：(num_for_sort, path, base_name_without_ext)
    let mut meta_files: Vec<(u64, PathBuf, String)> = Vec::new();

    if let Ok(meta_entries) = fs::read_dir(&session_path) {
        for entry in meta_entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("meta")) {
                let file_name_os = entry.file_name();
                let file_name_with_ext = file_name_os.to_string_lossy();
                let base_name = file_name_with_ext.trim_end_matches(".meta").to_string();

                // 尝试解析为数字用于排序（即使失败也保留，但通常都是数字）
                let num = base_name.parse::<u64>().unwrap_or(0);

                meta_files.push((num, path, base_name));
            }
        }
    }

    // 按编号 **降序** 排序（最新存档在前）
    meta_files.sort_by(|a, b| b.0.cmp(&a.0));

    // 正则表达式（可复用）
    let phase_re = Regex::new(r#"phase\s*=\s*"([^"]+)"#)?;
    let cycles_re = Regex::new(r#"cycles\s*=\s*(\d+)"#)?;
    let season_re = Regex::new(r#"season\s*=\s*"([^"]+)"#)?;

    // 解析每个 .meta 文件
    for (_, meta_path, base_name) in meta_files {
        if let Ok(content) = fs::read_to_string(&meta_path) {
            let now_phase = phase_re
                .captures(&content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let cycles = cycles_re
                .captures(&content)
                .and_then(|c| c[1].parse::<u32>().ok())
                .unwrap_or(0);

            let season = season_re
                .captures(&content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "unknown".to_string());

            phases.push(ArchivePhaseVO {
                now_phase,
                cycles,
                season,
                phase_file_name: base_name,
            });
        }
    }

    Ok(phases)
}

/// 同步 Master 和 Caves 的存档文件，删除不一致的存档
fn sync_archive_files(folder_path: &Path) -> Result<(), SimpleAppWebError> {
    let cc = AppCoreUtil::get_component::<ConstantComponent>()?;
    let master_session_dir = folder_path
        .join(&cc.master_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name);
    let caves_session_dir = folder_path
        .join(&cc.caves_shard_name)
        .join(&cc.save_folder_name)
        .join(&cc.session_folder_name);

    // 如果 Caves 目录不存在，则无需同步
    if !caves_session_dir.exists() {
        return Ok(());
    }

    // 获取 Master 的 session 子目录
    let master_session_path = get_session_path(&master_session_dir)?;
    if master_session_path.is_none() {
        return Ok(());
    }
    let master_session_path = master_session_path.unwrap();

    // 获取 Caves 的 session 子目录
    let caves_session_path = get_session_path(&caves_session_dir)?;
    if caves_session_path.is_none() {
        return Ok(());
    }
    let caves_session_path = caves_session_path.unwrap();

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

/// 服务器配置更新请求
#[derive(Deserialize)]
pub struct UpdateServerConfigRequest {
    pub server_id: String,
    pub max_players: u32,
    pub pvp: bool,
    pub pause_when_empty: bool,
    pub cluster_password: String,
    pub cluster_description: String,
    pub cluster_name: String,
}

/// 更新服务器配置接口
#[auto_command]
pub async fn update_server_config_handler(config: UpdateServerConfigRequest) -> JsonResponse {
    json_response_wrap!(function_name = "更新服务器配置", {
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;
        let constant_component: Arc<ConstantComponent> = AppCoreUtil::get_component()?;

        // 从数据库获取服务器路径
        let server_path = archive_path_service
            .get_path_by_id(&config.server_id)
            .await?
            .ok_or_else(|| {
                SimpleAppWebError::new(404, format!("Server path not found: {}", config.server_id))
            })?;

        let server_path = PathBuf::from(server_path);
        let ini_path = server_path.join(&constant_component.cluster_ini_file);

        // 读取现有的 ini 文件
        let mut ini = Ini::load_from_file(&ini_path)?;

        // 更新 [GAMEPLAY] 部分
        ini.with_section(Some("GAMEPLAY"))
            .set("max_players", config.max_players.to_string())
            .set("pvp", config.pvp.to_string())
            .set("pause_when_empty", config.pause_when_empty.to_string());

        // 更新 [NETWORK] 部分
        ini.with_section(Some("NETWORK"))
            .set("cluster_name", &config.cluster_name)
            .set("cluster_description", &config.cluster_description)
            .set("cluster_password", &config.cluster_password);

        // 写回文件
        ini.write_to_file(&ini_path)?;

        Ok(())
    })
}

/// 解析 modoverrides.lua 文件，通过执行 Lua 脚本提取模组ID列表
/// 返回 workshop-XXX 格式的模组ID列表
fn parse_level_data_override(folder_path: &Path) -> Result<Vec<String>, SimpleAppWebError> {
    let mut mod_ids = Vec::new();

    let cc = AppCoreUtil::get_component::<ConstantComponent>()?;
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
                        if key_str.starts_with("workshop-") {
                            mod_ids.push(key_str.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(mod_ids)
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
        let archive_path_service: Arc<ArchivePathService> = AppCoreUtil::get_component()?;

        // 从数据库获取服务器路径
        let server_path = archive_path_service
            .get_path_by_id(&request.server_id)
            .await?
            .ok_or_else(|| {
                SimpleAppWebError::new(404, format!("Server path not found: {}", request.server_id))
            })?;

        let server_path = PathBuf::from(server_path);
        let cc = AppCoreUtil::get_component::<ConstantComponent>()?;

        // 删除 Master 中的存档文件
        let master_session_dir = server_path
            .join(&cc.master_shard_name)
            .join(&cc.save_folder_name)
            .join(&cc.session_folder_name);
        if let Some(master_session_path) = get_session_path(&master_session_dir)? {
            delete_archive_files(&master_session_path, &request.phase_file_name)?;
        }

        // 删除 Caves 中的存档文件（如果存在）
        let caves_session_dir = server_path
            .join(&cc.caves_shard_name)
            .join(&cc.save_folder_name)
            .join(&cc.session_folder_name);
        if caves_session_dir.exists() {
            if let Some(caves_session_path) = get_session_path(&caves_session_dir)? {
                delete_archive_files(&caves_session_path, &request.phase_file_name)?;
            }
        }

        Ok(())
    })
}
