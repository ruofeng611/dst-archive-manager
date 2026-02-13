use crate::entity::{archive_path, key_value_store};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Schema,
};
use simple_starter_core::anyhow;
use simple_starter_macro::component;
use std::sync::Arc;

/// Key 常量定义
pub const KEY_NEXT_SERVER_NUMBER: &str = "next_server_number";
pub const KEY_STEAM_WORKSHOP_PATH: &str = "steam_workshop_path";
pub const KEY_DST_SERVER_PATH: &str = "dst_server_path";
pub const KEY_DST_CLIENT_PATH: &str = "dst_client_path";

#[component(init_method = "init")]
pub struct ArchivePathService {
    #[inject]
    pub database_connection: Arc<DatabaseConnection>,
}

impl ArchivePathService {
    pub async fn init(&self) -> anyhow::Result<()> {
        let connection = self.database_connection.as_ref();
        let builder = connection.get_database_backend();
        let schema = Schema::new(builder);

        // 创建 archive_path 表
        connection
            .execute(
                connection.get_database_backend().build(
                    schema
                        .create_table_from_entity(archive_path::Entity)
                        .if_not_exists(),
                ),
            )
            .await?;

        // 创建 key_value_store 表
        connection
            .execute(
                connection.get_database_backend().build(
                    schema
                        .create_table_from_entity(key_value_store::Entity)
                        .if_not_exists(),
                ),
            )
            .await?;

        // 初始化 next_server_number，如果不存在
        if self.get_value(KEY_NEXT_SERVER_NUMBER).await?.is_none() {
            self.set_value(KEY_NEXT_SERVER_NUMBER, "1").await?;
        }

        Ok(())
    }

    /// 获取 Value
    pub async fn get_value(&self, key: &str) -> Result<Option<String>, DbErr> {
        let connection = self.database_connection.as_ref();

        if let Some(model) = key_value_store::Entity::find_by_id(key)
            .one(connection)
            .await?
        {
            Ok(Some(model.value))
        } else {
            Ok(None)
        }
    }

    /// 设置 Value
    pub async fn set_value(&self, key: &str, value: &str) -> Result<(), DbErr> {
        let connection = self.database_connection.as_ref();

        // 先尝试查找
        if let Some(existing) = key_value_store::Entity::find_by_id(key)
            .one(connection)
            .await?
        {
            // 存在则更新
            let mut active_model: key_value_store::ActiveModel = existing.into();
            active_model.value = ActiveValue::Set(value.to_string());
            active_model.update(connection).await?;
        } else {
            // 不存在则创建
            let model = key_value_store::ActiveModel {
                key: ActiveValue::Set(key.to_string()),
                value: ActiveValue::Set(value.to_string()),
            };
            model.insert(connection).await?;
        }

        Ok(())
    }

    /// 保存或更新存档路径
    pub async fn save_archive_path(
        &self,
        id: String,
        archive_type: String,
        full_path: String,
    ) -> Result<(), DbErr> {
        let connection = self.database_connection.as_ref();

        // 先尝试查找
        if let Some(existing) = archive_path::Entity::find_by_id(&id)
            .one(connection)
            .await?
        {
            // 存在则更新
            let mut active_model: archive_path::ActiveModel = existing.into();
            active_model.archive_type = ActiveValue::Set(archive_type);
            active_model.full_path = ActiveValue::Set(full_path);
            active_model.update(connection).await?;
        } else {
            // 不存在则创建
            let model = archive_path::ActiveModel {
                id: ActiveValue::Set(id),
                archive_type: ActiveValue::Set(archive_type),
                full_path: ActiveValue::Set(full_path),
            };
            model.insert(connection).await?;
        }

        Ok(())
    }

    /// 根据 ID 获取路径
    pub async fn get_path_by_id(&self, id: &str) -> Result<Option<String>, DbErr> {
        let connection = self.database_connection.as_ref();

        if let Some(model) = archive_path::Entity::find_by_id(id).one(connection).await? {
            Ok(Some(model.full_path))
        } else {
            Ok(None)
        }
    }

    /// 删除路径记录
    pub async fn delete_path(&self, id: &str) -> Result<(), DbErr> {
        let connection = self.database_connection.as_ref();

        archive_path::Entity::delete_many()
            .filter(archive_path::Column::Id.eq(id))
            .exec(connection)
            .await?;

        Ok(())
    }

    /// 获取下一个可用的 Server 编号并自动增加
    pub async fn get_next_server_number(&self) -> Result<i32, DbErr> {
        let current_value = self.get_value(KEY_NEXT_SERVER_NUMBER).await?;
        let next_number = current_value
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1);

        // 更新计数器
        self.set_value(KEY_NEXT_SERVER_NUMBER, &(next_number + 1).to_string())
            .await?;

        Ok(next_number)
    }
}
