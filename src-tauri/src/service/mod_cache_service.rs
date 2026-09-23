//! 模组名称解析缓存。
//!
//! 名称来自执行 `modinfo.lua`（较慢），按 `(mod_id, locale)` 缓存，
//! 并以模组目录 mtime 判定是否失效。

use crate::entity::mod_cache;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use simple_starter_core::component;
use std::sync::Arc;

#[component]
pub struct ModCacheService {
    #[inject]
    pub database_connection: Arc<DatabaseConnection>,
}

type CacheEntry = (Option<String>, i64);

impl ModCacheService {
    fn conn(&self) -> &DatabaseConnection {
        self.database_connection.as_ref()
    }

    /// 读取缓存：返回 (名称, 解析时的目录 mtime)
    pub async fn get(&self, mod_id: &str, locale: &str) -> Result<Option<CacheEntry>, DbErr> {
        Ok(mod_cache::Entity::find_by_id((mod_id.to_string(), locale.to_string()))
            .one(self.conn())
            .await?
            .map(|model| (model.name, model.dir_mtime)))
    }

    /// 写入缓存（名称解析失败时存 `None`，避免反复执行注定失败的脚本）
    pub async fn set(
        &self,
        mod_id: &str,
        locale: &str,
        name: Option<&str>,
        dir_mtime: i64,
    ) -> Result<(), DbErr> {
        let conn = self.conn();
        let existing = mod_cache::Entity::find_by_id((mod_id.to_string(), locale.to_string()))
            .one(conn)
            .await?;

        match existing {
            Some(model) => {
                let mut active: mod_cache::ActiveModel = model.into();
                active.name = ActiveValue::Set(name.map(str::to_string));
                active.dir_mtime = ActiveValue::Set(dir_mtime);
                active.update(conn).await?;
            }
            None => {
                mod_cache::ActiveModel {
                    mod_id: ActiveValue::Set(mod_id.to_string()),
                    locale: ActiveValue::Set(locale.to_string()),
                    name: ActiveValue::Set(name.map(str::to_string)),
                    dir_mtime: ActiveValue::Set(dir_mtime),
                }
                .insert(conn)
                .await?;
            }
        }

        Ok(())
    }
}
