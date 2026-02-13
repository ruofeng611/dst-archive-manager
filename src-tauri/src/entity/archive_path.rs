use sea_orm::entity::prelude::*;

/// 存档/服务器路径表
/// 存储每个 Cluster 或 Server 的完整路径
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "archive_path")]
pub struct Model {
    /// ID，对应 Cluster_X 或 Server_X
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// 类型：cluster 或 server
    pub archive_type: String,
    /// 完整路径
    pub full_path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
