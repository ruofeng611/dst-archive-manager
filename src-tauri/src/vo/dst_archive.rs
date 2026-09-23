use serde::Serialize;

/// 存档列表响应（列表项只含摘要，详情按需另查）
#[derive(Serialize, Default)]
pub struct ArchiveListVO {
    pub found: bool,
    pub path: String, // 实际扫描的存档根目录（DoNotStarveTogether 全路径）
    pub items: Vec<ArchiveSummaryVO>,
}

/// 存档列表项摘要
///
/// 只含列表展示所需字段：不包含阶段列表与 mod_ids，避免列表接口承担详情解析。
#[derive(Serialize, Default, Clone, Debug)]
pub struct ArchiveSummaryVO {
    pub id: String,           // 文件夹名 (Cluster_X / Server_X)
    pub cluster_name: String, // 世界名
    pub has_caves: bool,      // 是否有洞穴（存在 Caves 文件夹）
    pub status: String,       // 服务器状态: stopped, starting, running
    pub latest_phase: Option<ArchivePhaseVO>, // 最新一个存档的阶段信息
}

/// 服务器详情（不含阶段列表，阶段列表见 get_server_saves）
#[derive(Serialize, Default, Clone, Debug)]
pub struct ServerDetailVO {
    pub id: String,
    pub game_mode: String,
    pub max_players: u32,
    pub pvp: bool,
    pub pause_when_empty: bool,
    pub cluster_password: String,
    pub cluster_description: String,
    pub cluster_name: String,
    pub has_caves: bool,
    pub status: String,
    pub mod_ids: Vec<String>, // 存档使用的模组ID列表 (workshop-XXX 格式)
    /// 该服务器自身设置的 cluster token；None 表示未覆盖、使用全局配置
    pub cluster_token: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ArchivePhaseVO {
    pub now_phase: String,       // 当前阶段: day、dusk(白天、黄昏)
    pub cycles: u32,             // 游戏天数
    pub season: String,          // 当前季节: spring、summer、autumn、winter
    pub phase_file_name: String, // 解析出上述三个字段的游戏阶段文件名 ，仅文件名（不含 .meta），如 "0000000005"
    /// 该存档只存在于地面或洞穴单侧（无法加载）；列表项置灰用
    pub mismatched: bool,
}
