use sea_orm::{Database, DatabaseConnection, DbErr};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use simple_starter_core::{configuration, provider};

#[derive(Deserialize)]
#[configuration("database")]
pub struct DatabaseConfig {
    pub db_path: String,
    pub sqlx_logging: bool,
    pub max_connections: u32,
    pub min_connections: u32,
    pub time_out: u64,
}

#[provider]
pub async fn create_database_connection(
    database_config: Arc<DatabaseConfig>,
) -> Result<DatabaseConnection, DbErr> {
    if let Some(parent) = Path::new(&database_config.db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| DbErr::Custom("Failed to create parent directory".to_string()))?;
    }
    let db_url = format!("sqlite://{}?mode=rwc", database_config.db_path);
    let mut opt = sea_orm::ConnectOptions::new(db_url.to_owned());
    opt.max_connections(database_config.max_connections)
        .min_connections(database_config.min_connections)
        .sqlx_logging(database_config.sqlx_logging)
        .acquire_timeout(std::time::Duration::from_secs(database_config.time_out));
    let db = Database::connect(opt).await?;
    Ok(db)
}
