//! 服务器运行状态服务。
//!
//! 维护 `server_runtime` 表：应用启动服务器时写入，完全退出后删除。
//! 前端的状态展示统一读这张表；写操作只由后端发起。

use crate::entity::server_runtime;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait,
};
use simple_starter_core::component;
use std::collections::HashMap;
use std::sync::Arc;

/// 运行状态取值
pub const STATUS_STARTING: &str = "starting";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPING: &str = "stopping";

#[component]
pub struct ServerRuntimeService {
    #[inject]
    pub database_connection: Arc<DatabaseConnection>,
}

impl ServerRuntimeService {
    fn conn(&self) -> &DatabaseConnection {
        self.database_connection.as_ref()
    }

    /// 全部运行中记录
    pub async fn list(&self) -> Result<Vec<server_runtime::Model>, DbErr> {
        server_runtime::Entity::find().all(self.conn()).await
    }

    /// `server_id -> status` 映射（列表展示用；缺失即已停止）
    pub async fn status_map(&self) -> Result<HashMap<String, String>, DbErr> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .map(|model| (model.server_id, model.status))
            .collect())
    }

    /// 单个服务器的状态（不受管则返回 `None`）
    pub async fn status(&self, server_id: &str) -> Result<Option<String>, DbErr> {
        Ok(server_runtime::Entity::find_by_id(server_id)
            .one(self.conn())
            .await?
            .map(|model| model.status))
    }

    /// 登记/更新运行状态
    pub async fn upsert(
        &self,
        server_id: &str,
        status: &str,
        expect_caves: bool,
    ) -> Result<(), DbErr> {
        let conn = self.conn();
        let existing = server_runtime::Entity::find_by_id(server_id).one(conn).await?;

        match existing {
            Some(model) => {
                let mut active: server_runtime::ActiveModel = model.into();
                active.status = ActiveValue::Set(status.to_string());
                active.expect_caves = ActiveValue::Set(expect_caves);
                active.update(conn).await?;
            }
            None => {
                server_runtime::ActiveModel {
                    server_id: ActiveValue::Set(server_id.to_string()),
                    status: ActiveValue::Set(status.to_string()),
                    expect_caves: ActiveValue::Set(expect_caves),
                }
                .insert(conn)
                .await?;
            }
        }

        Ok(())
    }

    /// 仅更新状态（不存在时不做任何事）
    pub async fn set_status(&self, server_id: &str, status: &str) -> Result<(), DbErr> {
        let conn = self.conn();
        if let Some(model) = server_runtime::Entity::find_by_id(server_id).one(conn).await? {
            let mut active: server_runtime::ActiveModel = model.into();
            active.status = ActiveValue::Set(status.to_string());
            active.update(conn).await?;
        }
        Ok(())
    }

    /// 移除记录（服务器已完全退出 / 被删除）
    pub async fn remove(&self, server_id: &str) -> Result<(), DbErr> {
        server_runtime::Entity::delete_by_id(server_id)
            .exec(self.conn())
            .await?;
        Ok(())
    }
}
