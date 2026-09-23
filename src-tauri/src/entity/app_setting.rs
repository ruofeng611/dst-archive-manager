use sea_orm::entity::prelude::*;

/// 应用级配置表（单行表，id 固定为 1）
///
/// 只存用户显式设置的配置；可推导的数据（存档目录、路径）不入库。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "app_setting")]
pub struct Model {
    /// 固定为 1，保证单行
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    /// DST 客户端安装目录
    pub dst_client_path: Option<String>,
    /// DST 服务端安装目录
    pub dst_server_path: Option<String>,
    /// Steam 创意工坊 content 目录
    pub workshop_path: Option<String>,
    /// 存档根目录（含 Cluster_X 与 Server_X 的 DoNotStarveTogether 目录）
    pub archive_root: Option<String>,
    /// 全局默认 cluster token（服务级未设置时使用）
    pub global_cluster_token: Option<String>,
    /// 界面语言。未设置时取 ConstantComponent.default_language
    pub language: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
