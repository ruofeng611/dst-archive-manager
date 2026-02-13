use serde::Serialize;

// 模组信息
#[derive(Serialize, Default, Clone)]
pub struct ModInfoVO {
    pub folder_name: String, // 文件夹名 (如 workshop-123456 或 123456)
    pub mod_name: String,    // 模组名称 (从 modinfo.lua 解析的 name 字段)
}

// 模组扫描结果
#[derive(Serialize, Default)]
pub struct ModScanVO {
    pub found: bool,
    pub server_path: String,  // Dedicated Server 的路径
    pub mods: Vec<ModInfoVO>, // 所有模组信息列表
}
