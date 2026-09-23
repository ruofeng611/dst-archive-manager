use serde::Serialize;

/// 本地可用模组列表（只读目录树，不含名称）
#[derive(Serialize, Default)]
pub struct ModListVO {
    /// 服务端安装目录配置有效且存在时为 true
    pub found: bool,
    /// 模组 id 列表（统一为 workshop-N 形式）
    pub mod_ids: Vec<String>,
}

/// 单个模组的名称解析结果
#[derive(Serialize, Clone, Debug)]
pub struct ModNameVO {
    pub mod_id: String,
    /// modinfo.lua 中的 name；模组不在本地或解析失败时为 None
    pub name: Option<String>,
}
