//! 移除 `server_setting` 表：服务器令牌以 `Server_X/cluster_token.txt` 为准。
//!
//! 全局令牌仍存在 `app_setting.global_cluster_token`（转服务器时的默认值）；
//! 每个服务器的令牌由该服务器目录下的文件表达。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ServerSetting {
    Table,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ServerSetting::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 令牌已以文件为准，数据不可从表结构回滚
        Ok(())
    }
}
