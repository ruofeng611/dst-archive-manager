//! 存档目录路径推导。
//!
//! 路径不入库，统一由 (archive_root, id) 推导：
//! - `Server_X`  → `{archive_root}/Server_X`
//! - `Cluster_X` → `{archive_root}/{用户ID}/Cluster_X`

use crate::{ConstantComponent, SimpleAppWebError};
use std::fs;
use std::path::{Path, PathBuf};

/// 推导存档目录的绝对路径
pub fn resolve_archive_path(
    cc: &ConstantComponent,
    archive_root: &Path,
    id: &str,
) -> Result<PathBuf, SimpleAppWebError> {
    if id.starts_with(&cc.dst_server_prefix) {
        return Ok(archive_root.join(id));
    }
    if id.starts_with(&cc.dst_cluster_prefix) {
        let user_dir = find_user_dir(archive_root)?.ok_or_else(|| {
            SimpleAppWebError::new(
                500,
                format!("未找到用户存档目录: {}", archive_root.display()),
            )
        })?;
        return Ok(user_dir.join(id));
    }
    Err(SimpleAppWebError::new(
        400,
        format!("无法识别的存档 ID: {}", id),
    ))
}

/// 查找存档根目录下唯一的数字命名用户目录（Cluster 的父目录）
///
/// 存在多个数字目录时返回错误：无法判断 Cluster 归属，不做静默取舍。
pub fn find_user_dir(archive_root: &Path) -> Result<Option<PathBuf>, SimpleAppWebError> {
    let mut found: Option<PathBuf> = None;

    for entry in fs::read_dir(archive_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if found.is_some() {
            return Err(SimpleAppWebError::new(
                500,
                format!(
                    "存档根目录下存在多个用户目录，无法确定 Cluster 位置: {}",
                    archive_root.display()
                ),
            ));
        }
        found = Some(entry.path());
    }

    Ok(found)
}
