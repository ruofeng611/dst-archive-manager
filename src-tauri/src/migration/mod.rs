//! 数据库迁移模块。
//!
//! 表结构的唯一来源：所有变更通过新增迁移脚本演进。
//!
//! 约束：迁移脚本内**只用 raw SQL / SchemaManager，不引用 entity**——
//! entity 会随后续演进变化，引用它们会让旧迁移编译不过。

pub use sea_orm_migration::prelude::*;

mod m_0001_init;
mod m_0002_migrate_from_kv;
mod m_0003_drop_server_setting;
mod m_0004_create_server_runtime;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m_0001_init::Migration),
            Box::new(m_0002_migrate_from_kv::Migration),
            Box::new(m_0003_drop_server_setting::Migration),
            Box::new(m_0004_create_server_runtime::Migration),
        ]
    }
}
