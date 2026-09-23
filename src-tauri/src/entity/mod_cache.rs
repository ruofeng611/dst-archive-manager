use sea_orm::entity::prelude::*;

/// modinfo.lua 解析结果缓存
///
/// 目录 mtime 未变化时直接复用，避免每次查询都执行 Lua 脚本。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mod_cache")]
pub struct Model {
    /// 模组 id（统一为 workshop-N 形式）
    #[sea_orm(primary_key, auto_increment = false)]
    pub mod_id: String,
    /// 解析时使用的语言
    #[sea_orm(primary_key, auto_increment = false)]
    pub locale: String,
    /// modinfo.lua 中的 name；None 表示解析失败
    pub name: Option<String>,
    /// 模组目录的修改时间（Unix 秒），用于判定缓存是否失效
    pub dir_mtime: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
