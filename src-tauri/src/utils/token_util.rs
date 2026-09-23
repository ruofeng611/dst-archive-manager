//! 服务器令牌文件（`Server_X/cluster_token.txt`）读写。
//!
//! 该文件是每个服务器令牌的唯一真相源；全局令牌只作为转服务器时的默认值。

use crate::SimpleAppWebError;
use std::fs;
use std::path::Path;

/// 读取令牌；文件不存在或内容为空时返回 `None`
pub fn read_token(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let token = content.trim();

    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// 写入令牌（会先 trim）
pub fn write_token(path: &Path, token: &str) -> Result<(), SimpleAppWebError> {
    fs::write(path, token.trim()).map_err(|e| {
        SimpleAppWebError::new(500, format!("写入令牌文件失败: {}", e))
    })
}
