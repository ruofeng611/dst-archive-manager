use serde::Serialize;

/// 应用配置（配置页数据源）
///
/// 未设置的项为 `None`（前端显示为空）。
#[derive(Serialize)]
pub struct AppSettingsVO {
    /// DST 客户端安装目录
    pub dst_client_path: Option<String>,
    /// DST 服务端安装目录
    pub dst_server_path: Option<String>,
    /// Steam 创意工坊 content 目录
    pub workshop_path: Option<String>,
    /// 存档根目录
    pub archive_root: Option<String>,
    /// 全局 cluster token（服务级未设置时的默认值）
    pub global_cluster_token: Option<String>,
}
