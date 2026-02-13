use sea_orm::entity::prelude::*;

/// Key-Value 存储表
/// 用于存储各种配置和路径信息
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "key_value_store")]
pub struct Model {
    /// Key，主键
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    /// Value 值
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
