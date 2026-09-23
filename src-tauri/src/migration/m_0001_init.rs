//! 建立配置类表：app_setting / server_setting / mod_cache。
//!
//! 注：`server_setting` 在 m_0003 中被移除（服务器令牌以文件为准）。已应用的迁移不再修改，
//! 这里保留原语句以维持迁移链的线性。

use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppSetting {
    Table,
    Id,
    DstClientPath,
    DstServerPath,
    WorkshopPath,
    ArchiveRoot,
    GlobalClusterToken,
    Language,
}

#[derive(DeriveIden)]
enum ServerSetting {
    Table,
    ServerId,
    ClusterToken,
}

#[derive(DeriveIden)]
enum ModCache {
    Table,
    ModId,
    Locale,
    Name,
    DirMtime,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppSetting::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppSetting::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppSetting::DstClientPath).string().null())
                    .col(ColumnDef::new(AppSetting::DstServerPath).string().null())
                    .col(ColumnDef::new(AppSetting::WorkshopPath).string().null())
                    .col(ColumnDef::new(AppSetting::ArchiveRoot).string().null())
                    .col(
                        ColumnDef::new(AppSetting::GlobalClusterToken)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(AppSetting::Language).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ServerSetting::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerSetting::ServerId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ServerSetting::ClusterToken).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModCache::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ModCache::ModId).string().not_null())
                    .col(ColumnDef::new(ModCache::Locale).string().not_null())
                    .col(ColumnDef::new(ModCache::Name).string().null())
                    .col(ColumnDef::new(ModCache::DirMtime).big_integer().not_null())
                    // 复合主键必须用 primary_key(Index) 声明；
                    // 逐列 .primary_key() 会生成两个 PRIMARY KEY 子句，SQLite 直接报错
                    .primary_key(Index::create().col(ModCache::ModId).col(ModCache::Locale))
                    .to_owned(),
            )
            .await?;

        // 单行表：占位插入固定行，后续读写都针对 id = 1
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "INSERT OR IGNORE INTO app_setting (id) VALUES (1)",
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ModCache::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ServerSetting::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AppSetting::Table).to_owned())
            .await?;
        Ok(())
    }
}
