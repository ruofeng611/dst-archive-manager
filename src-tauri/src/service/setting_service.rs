//! 配置服务。
//!
//! 管理应用级配置（`app_setting` 单行表）。服务器令牌不以数据库存储，见 `utils::read_token` /
//! `utils::write_token`（真相源是各服务器目录下的 `cluster_token.txt`）。
//! 可推导的数据（存档目录路径）不入库，由 [`SettingService::resolve_archive_path`] 推导。

use crate::entity::app_setting;
use crate::migration::{Migrator, MigratorTrait};
use crate::utils::{find_user_dir, resolve_archive_path};
use crate::{ConstantComponent, SimpleAppWebError};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use simple_starter_core::{anyhow, component};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// `app_setting` 单行表的固定主键
const APP_SETTING_ROW_ID: i32 = 1;

/// 应用级配置
#[derive(Debug, Clone, Default)]
pub struct AppSetting {
    pub dst_client_path: Option<String>,
    pub dst_server_path: Option<String>,
    pub workshop_path: Option<String>,
    pub archive_root: Option<String>,
    pub global_cluster_token: Option<String>,
}

/// 应用级配置的局部更新
///
/// 字段语义：`None` 表示不修改；`Some` 为空串表示清除该字段。
#[derive(Debug, Clone, Default)]
pub struct AppSettingPatch {
    pub dst_client_path: Option<String>,
    pub dst_server_path: Option<String>,
    pub workshop_path: Option<String>,
    pub archive_root: Option<String>,
    pub global_cluster_token: Option<String>,
    pub language: Option<String>,
}

/// 空串视为清除（置为 NULL），避免把空配置写进库
fn clear_if_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[component(init_method = "init")]
pub struct SettingService {
    #[inject]
    pub database_connection: Arc<DatabaseConnection>,
    #[inject]
    pub constant_component: Arc<ConstantComponent>,
}

impl SettingService {
    pub async fn init(&self) -> anyhow::Result<()> {
        Migrator::up(self.database_connection.as_ref(), None).await?;
        Ok(())
    }

    fn conn(&self) -> &DatabaseConnection {
        self.database_connection.as_ref()
    }

    /// 读取应用级配置
    pub async fn get(&self) -> Result<AppSetting, DbErr> {
        let model = app_setting::Entity::find_by_id(APP_SETTING_ROW_ID)
            .one(self.conn())
            .await?;

        Ok(match model {
            Some(m) => AppSetting {
                dst_client_path: m.dst_client_path,
                dst_server_path: m.dst_server_path,
                workshop_path: m.workshop_path,
                archive_root: m.archive_root,
                global_cluster_token: m.global_cluster_token,
            },
            None => AppSetting::default(),
        })
    }

    /// 局部更新应用级配置
    pub async fn update(&self, patch: AppSettingPatch) -> Result<(), DbErr> {
        let conn = self.conn();
        let existing = app_setting::Entity::find_by_id(APP_SETTING_ROW_ID)
            .one(conn)
            .await?;

        match existing {
            Some(model) => {
                let mut active: app_setting::ActiveModel = model.into();
                if let Some(v) = patch.dst_client_path {
                    active.dst_client_path = ActiveValue::Set(clear_if_empty(v));
                }
                if let Some(v) = patch.dst_server_path {
                    active.dst_server_path = ActiveValue::Set(clear_if_empty(v));
                }
                if let Some(v) = patch.workshop_path {
                    active.workshop_path = ActiveValue::Set(clear_if_empty(v));
                }
                if let Some(v) = patch.archive_root {
                    active.archive_root = ActiveValue::Set(clear_if_empty(v));
                }
                if let Some(v) = patch.global_cluster_token {
                    active.global_cluster_token = ActiveValue::Set(clear_if_empty(v));
                }
                if let Some(v) = patch.language {
                    active.language = ActiveValue::Set(clear_if_empty(v));
                }
                active.update(conn).await?;
            }
            None => {
                let active = app_setting::ActiveModel {
                    id: ActiveValue::Set(APP_SETTING_ROW_ID),
                    dst_client_path: ActiveValue::Set(patch.dst_client_path),
                    dst_server_path: ActiveValue::Set(patch.dst_server_path),
                    workshop_path: ActiveValue::Set(patch.workshop_path),
                    archive_root: ActiveValue::Set(patch.archive_root),
                    global_cluster_token: ActiveValue::Set(patch.global_cluster_token),
                    language: ActiveValue::Set(patch.language),
                };
                active.insert(conn).await?;
            }
        }

        Ok(())
    }

    /// 界面语言（未设置时取常量中的默认值）
    pub async fn language(&self) -> Result<String, DbErr> {
        let stored = app_setting::Entity::find_by_id(APP_SETTING_ROW_ID)
            .one(self.conn())
            .await?
            .and_then(|m| m.language);

        Ok(stored.unwrap_or_else(|| self.constant_component.default_language.clone()))
    }

    /// 设置界面语言
    pub async fn set_language(&self, language: &str) -> Result<(), DbErr> {
        self.update(AppSettingPatch {
            language: Some(language.to_string()),
            ..Default::default()
        })
        .await
    }

    /// 存档根目录
    pub async fn archive_root(&self) -> Result<Option<String>, DbErr> {
        Ok(app_setting::Entity::find_by_id(APP_SETTING_ROW_ID)
            .one(self.conn())
            .await?
            .and_then(|m| m.archive_root))
    }

    /// 存档根目录下唯一的数字用户目录（Cluster 的父目录）
    ///
    /// 根目录未配置或不存在数字目录时返回 `None`；存在多个时返回错误。
    pub async fn user_dir(&self) -> Result<Option<PathBuf>, SimpleAppWebError> {
        let Some(root) = self.archive_root().await? else {
            return Ok(None);
        };
        let root = PathBuf::from(root);
        if !root.is_dir() {
            return Ok(None);
        }
        find_user_dir(&root)
    }

    /// 推导某个存档目录的绝对路径
    pub async fn resolve_archive_path(&self, id: &str) -> Result<PathBuf, SimpleAppWebError> {
        let root = self.archive_root().await?.ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                "存档根目录未配置，请先在配置页设置存档目录".to_string(),
            )
        })?;

        resolve_archive_path(
            &self.constant_component,
            Path::new(&root),
            id,
        )
    }
}
