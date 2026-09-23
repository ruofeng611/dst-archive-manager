use sea_orm::entity::prelude::*;

/// 服务器运行状态（应用维护的监视会话 + 最近一轮探测快照）
///
/// - 行存在 = 该服务器「未完全退出」；行不存在 = 已停止
/// - `status`：`starting`（已启动进程、等待窗口）/ `running`（期望的分片都出现过）/ `stopping`（正在关闭）
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "server_runtime")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub server_id: String,
    pub status: String,
    /// 本次启动是否应存在洞穴分片（按 Caves 目录是否存在判断）
    pub expect_caves: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
