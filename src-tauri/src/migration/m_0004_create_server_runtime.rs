//! 建立服务器运行状态表：`server_runtime`。
//!
//! 该表是应用维护的「监视会话 + 最近一轮探测快照」：启动服务器时写入，完全退出后删除。
//! 前端的状态展示统一读这张表（真相源仍是窗口，快照最多滞后一个 tick）。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ServerRuntime {
    Table,
    ServerId,
    Status,
    ExpectCaves,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServerRuntime::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerRuntime::ServerId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ServerRuntime::Status).string().not_null())
                    .col(
                        ColumnDef::new(ServerRuntime::ExpectCaves)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServerRuntime::Table).to_owned())
            .await?;
        Ok(())
    }
}
