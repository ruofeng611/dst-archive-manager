//! 应用配置的读写接口。

use crate::service::{AppSettingPatch, SettingService};
use crate::support::JsonResponse;
use crate::utils::{app_component, get_all_steam_libraries};
use crate::vo::AppSettingsVO;
use crate::{ConstantComponent, json_response_wrap};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri_macros::auto_command;

/// 获取应用配置
#[auto_command]
pub async fn get_settings_handler() -> JsonResponse {
    json_response_wrap!(function_name = "获取配置", {
        let setting_service: Arc<SettingService> = app_component()?;
        let setting = setting_service.get().await?;

        Ok(AppSettingsVO {
            dst_client_path: setting.dst_client_path,
            dst_server_path: setting.dst_server_path,
            workshop_path: setting.workshop_path,
            archive_root: setting.archive_root,
            global_cluster_token: setting.global_cluster_token,
        })
    })
}

/// 更新应用配置请求
///
/// 字段为 `None` 表示不修改；传空串表示清除该项。
#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub dst_client_path: Option<String>,
    pub dst_server_path: Option<String>,
    pub workshop_path: Option<String>,
    pub archive_root: Option<String>,
    pub global_cluster_token: Option<String>,
}

/// 更新应用配置
#[auto_command]
pub async fn update_settings_handler(request: UpdateSettingsRequest) -> JsonResponse {
    json_response_wrap!(function_name = "更新配置", {
        let setting_service: Arc<SettingService> = app_component()?;

        // 路径类配置必须是已存在的目录（空串表示清除，不校验）
        for (label, value) in [
            ("客户端安装目录", &request.dst_client_path),
            ("服务端安装目录", &request.dst_server_path),
            ("创意工坊目录", &request.workshop_path),
            ("存档根目录", &request.archive_root),
        ] {
            if let Some(path) = value {
                if !path.trim().is_empty() && !Path::new(path).is_dir() {
                    return Err(SimpleAppWebError::new(
                        400,
                        format!("{}不存在或不是目录: {}", label, path),
                    ));
                }
            }
        }

        setting_service
            .update(AppSettingPatch {
                dst_client_path: request.dst_client_path,
                dst_server_path: request.dst_server_path,
                workshop_path: request.workshop_path,
                archive_root: request.archive_root,
                global_cluster_token: request.global_cluster_token,
                language: None,
            })
            .await?;

        Ok(())
    })
}

/// 自动扫描结果
///
/// 字段名与 `update_settings` 的 patch 保持一致，前端可直接把扫到的项作为 patch 提交；
/// 未检测到的项为 `None`。本接口只读，是否写入配置由配置页决定。
#[derive(Serialize, Default)]
pub struct DiscoveredPathsVO {
    pub dst_client_path: Option<String>,
    pub dst_server_path: Option<String>,
    pub workshop_path: Option<String>,
    pub archive_root: Option<String>,
}

/// 自动扫描本机上的 DST 相关目录（仅配置页的「自动扫描」使用）
#[auto_command]
pub async fn discover_paths_handler() -> JsonResponse {
    json_response_wrap!(function_name = "自动扫描目录", {
        let cc = app_component::<ConstantComponent>()?;
        let mut discovered = DiscoveredPathsVO::default();

        // 存档根目录：文档目录下的 Klei/DoNotStarveTogether
        if let Some(doc) = dirs::document_dir() {
            let candidate = doc
                .join(&cc.klei_folder_name)
                .join(&cc.dst_archive_file_name);
            if candidate.is_dir() {
                discovered.archive_root = Some(candidate.to_string_lossy().into_owned());
            }
        }

        // 安装目录与创意工坊目录：遍历 Steam 库（读注册表 + libraryfolders.vdf）
        for lib in get_all_steam_libraries(&cc) {
            let common = lib
                .join(&cc.steamapps_folder_name)
                .join(&cc.common_folder_name);

            if discovered.dst_server_path.is_none() {
                let candidate = common.join(&cc.dst_server_file_name);
                if candidate.is_dir() {
                    discovered.dst_server_path = Some(candidate.to_string_lossy().into_owned());
                }
            }
            if discovered.dst_client_path.is_none() {
                let candidate = common.join(&cc.dst_client_file_name);
                if candidate.is_dir() {
                    discovered.dst_client_path = Some(candidate.to_string_lossy().into_owned());
                }
            }
            if discovered.workshop_path.is_none() {
                let candidate = lib
                    .join(&cc.steamapps_folder_name)
                    .join(&cc.workshop_folder_name);
                if candidate.is_dir() {
                    discovered.workshop_path = Some(candidate.to_string_lossy().into_owned());
                }
            }

            if discovered.dst_server_path.is_some()
                && discovered.dst_client_path.is_some()
                && discovered.workshop_path.is_some()
            {
                break;
            }
        }

        Ok(discovered)
    })
}

