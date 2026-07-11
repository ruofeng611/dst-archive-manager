mod component;
mod entity;
mod handler;
mod service;
mod support;
mod utils;
mod vo;

use serde::Deserialize;
use simple_starter_core::tracing::info;
use simple_starter_core::{Application, LogExpectExt};
use simple_starter_macro::configuration;
use support::dynamic_invoke_handler;
use toml::{Value, toml};

pub use support::AutoCommand;
pub use support::SimpleAppWebError;
pub use support::process_data;

#[derive(Deserialize)]
#[configuration("constant")]
pub struct ConstantComponent {
    pub dst_server_file_name: String,
    pub dst_client_file_name: String,
    pub dst_archive_file_name: String,
    pub dst_steam_id: String,
    pub dst_cluster_prefix: String,
    pub dst_server_prefix: String,
    pub cluster_token_file: String,
    pub dst_end_command: String,
    pub cluster_ini_file: String,
    pub klei_folder_name: String,
    pub master_shard_name: String,
    pub caves_shard_name: String,
    pub modinfo_file_name: String,
    pub modoverrides_file_name: String,
    pub start_server_bat_name: String,
    pub save_folder_name: String,
    pub session_folder_name: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Application::new()
        .add_default_config(Value::Table(toml! {
            [app]
            name = "dst-archive-manager"

            [logger]
            level = "debug"

            [constant]
            dst_server_file_name = "Don't Starve Together Dedicated Server"
            dst_client_file_name = "Don't Starve Together"
            dst_archive_file_name = "DoNotStarveTogether"
            dst_steam_id = "322330"
            dst_cluster_prefix = "Cluster_"
            dst_server_prefix = "Server_"
            cluster_token_file = "cluster_token.txt"
            dst_end_command = "c_shutdown()"
            cluster_ini_file = "cluster.ini"
            klei_folder_name = "Klei"
            master_shard_name = "Master"
            caves_shard_name = "Caves"
            modinfo_file_name = "modinfo.lua"
            modoverrides_file_name = "modoverrides.lua"
            start_server_bat_name = "StartServer.bat"
            save_folder_name = "save"
            session_folder_name = "session"

            [database]
            db_path = "dst_data.db"
            sqlx_logging = true
            max_connections = 1
            min_connections = 1
            time_out = 10

            [runtime]
            worker_thread_num = 1
        }))
        .set_main_loop_hook(|mut application| {
            // 构建 App
            let app = tauri::Builder::default()
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
                .invoke_handler(dynamic_invoke_handler)
                .build(tauri::generate_context!())
                .log_expect("error while building tauri application");
            // 运行 App (这里可以传入回调闭包)
            app.run(move |_app_handle, event| match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    info!("Received ExitRequested event, exiting...");
                    application.shutdown();
                }
                _ => {}
            });
        })
        .run();
}
