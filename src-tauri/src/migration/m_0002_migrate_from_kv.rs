//! 把旧 key_value_store 中的配置搬迁到 app_setting，并删除废弃的 archive_path / key_value_store 表。
//!
//! 数据映射：
//! - dst_client_path    → app_setting.dst_client_path
//! - dst_server_path    → app_setting.dst_server_path
//! - steam_workshop_path→ app_setting.workshop_path
//! - dst_archive_dir    → app_setting.archive_root
//! - app_language       → app_setting.language
//! - dst_user_dir       → 丢弃（改为扫描 archive_root 下的数字目录）
//! - archive_path 表    → 丢弃（路径由 archive_root + id 推导）

use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// 旧 KV key 到新列的映射；返回 None 表示该 key 不再保留
fn target_column(key: &str) -> Option<&'static str> {
    match key {
        "dst_client_path" => Some("dst_client_path"),
        "dst_server_path" => Some("dst_server_path"),
        "steam_workshop_path" => Some("workshop_path"),
        "dst_archive_dir" => Some("archive_root"),
        "app_language" => Some("language"),
        _ => None,
    }
}

async fn table_exists<C>(conn: &C, table: &str) -> Result<bool, DbErr>
where
    C: ConnectionTrait,
{
    let stmt = Statement::from_sql_and_values(
        conn.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?",
        [Value::from(table.to_owned())],
    );
    Ok(conn.query_one(stmt).await?.is_some())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let backend = conn.get_database_backend();

        if table_exists(conn, "key_value_store").await? {
            let rows = conn
                .query_all(Statement::from_string(
                    backend,
                    "SELECT key, value FROM key_value_store",
                ))
                .await?;

            for row in rows {
                let key: String = row.try_get("", "key")?;
                let value: String = row.try_get("", "value")?;

                let Some(column) = target_column(&key) else {
                    continue;
                };

                // 只在目标列为空时写入：重复执行不会覆盖用户已设置的值
                conn.execute(Statement::from_sql_and_values(
                    backend,
                    format!("UPDATE app_setting SET {column} = ? WHERE id = 1 AND {column} IS NULL"),
                    [Value::from(value)],
                ))
                .await?;
            }
        }

        conn.execute(Statement::from_string(
            backend,
            "DROP TABLE IF EXISTS archive_path",
        ))
        .await?;
        conn.execute(Statement::from_string(
            backend,
            "DROP TABLE IF EXISTS key_value_store",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 旧表已删除且数据可从磁盘重建，不做回滚
        Ok(())
    }
}
