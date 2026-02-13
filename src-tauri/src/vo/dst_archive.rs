use serde::Serialize;

// 存档扫描结果
#[derive(Serialize, Default)]
pub struct ArchiveScanVO {
    pub found: bool,
    pub path: String, // DoNotStarveTogether 的完整路径
    pub clusters: Vec<ArchiveDetailVO>,
    pub servers: Vec<ArchiveDetailVO>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct ArchiveDetailVO {
    pub id: String,                  // 文件夹名 (Cluster_X)
    pub game_mode: String,           // 游戏模式
    pub max_players: u32,            // 最大玩家数
    pub pvp: bool,                   // 玩家PVP模式是否开启
    pub pause_when_empty: bool,      // 当房间内没有玩家时是否暂停游戏
    pub cluster_password: String,    // 世界密码
    pub cluster_description: String, // 世界描述
    pub cluster_name: String,        // 世界名
    pub archive_phase_vo: Vec<ArchivePhaseVO>,
    pub has_caves: bool,      // 是否有洞穴（存在 Caves 文件夹）
    pub status: String,       // 服务器状态: stopped, starting, running
    pub mod_ids: Vec<String>, // 存档使用的模组ID列表 (workshop-XXX 格式)
}

#[derive(Serialize, Clone, Debug)]
pub struct ArchivePhaseVO {
    pub now_phase: String,       // 当前阶段: day、dusk(白天、黄昏)
    pub cycles: u32,             // 游戏天数
    pub season: String,          // 当前季节: spring、summer、autumn、winter
    pub phase_file_name: String, // 解析出上述三个字段的游戏阶段文件名 ，仅文件名（不含 .meta），如 "0000000005"
}
