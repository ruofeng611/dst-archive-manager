mod component;
mod entity;
mod handler;
mod migration;
mod service;
mod support;
mod utils;
mod vo;
mod watch;

use serde::Deserialize;
use simple_starter_core::tracing::info;
use simple_starter_core::{Application, LogExpectExt, configuration};
use std::path::{Path, PathBuf};
use support::dynamic_invoke_handler;
use support::init_service_name;
use toml::{Value, toml};

pub use support::AutoCommand;
pub use support::SimpleAppWebError;
pub use support::process_data;

/// 应用数据目录名（%LOCALAPPDATA% 下的子目录）
const APP_DIR_NAME: &str = "dst-archive-manager";
/// 数据库文件名（放在应用数据目录下）
const DB_FILE_NAME: &str = "dst_data.db";
/// 日志子目录名
const LOG_DIR_NAME: &str = "logs";

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
    pub default_language: String,
    pub mods_folder_name: String,
    pub workshop_folder_prefix: String,
    pub steamapps_folder_name: String,
    pub common_folder_name: String,
    pub workshop_folder_name: String,
    pub content_folder_name: String,
    pub library_folders_file: String,
    pub steam_registry_key: String,
    pub steam_path_value_name: String,
    pub steam_default_install_path: String,
    pub server_watch_tick_secs: u64,
    pub server_watch_miss_threshold: u32,
    pub server_watch_startup_grace_secs: u64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Application::new()
        .add_default_config(build_default_config())
        .add_startup_hook(|ctx| {
            // 服务器崩溃互保监视：随应用生命周期运行，收到取消令牌即退出。
            // 注册是同步动作，放在闭包体外；返回的 Future 不借用 ctx（须满足 'static）
            ctx.add_task_spawn_factory_in_context(|token| watch::run(token));
            async move { Ok(()) }
        })
        .set_main_loop_hook(|application| {
            // 配置快照已就绪，运行期只读的全局常量在此一次性初始化
            init_service_name();
            // 构建 App
            let app = tauri::Builder::default()
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
                .invoke_handler(dynamic_invoke_handler)
                .build(tauri::generate_context!())
                .log_expect("error while building tauri application");
            // 监视任务需要 AppHandle 才能向 WebView 推送状态变化
            watch::set_app_handle(app.handle().clone());
            // 运行 App（run_return 在事件循环结束后正常返回；run 会经 std::process::exit
            // 直接终止进程，跳过所有析构函数，Application 的 Drop 关闭流程将永不执行）
            let exit_code = app.run_return(|_app_handle, event| match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    info!("Received ExitRequested event, exiting...");
                }
                _ => {}
            });
            // 进程退出前确定性触发 Drop（内部执行 shutdown 关闭流程，恰好一次）
            drop(application);
            std::process::exit(exit_code);
        })
        .run();
}

/// 构建默认配置
///
/// 数据库与日志放在程序所在目录（卸载时删掉整个目录即可，不会散落在别处）。
/// 这两个路径是**引导值**：必须在框架加载配置之前确定（`db_path` 决定连哪个库），
/// 因此在这里直接算好注入，不走 ConstantComponent 配置化。
fn build_default_config() -> Value {
    let data_dir = resolve_data_dir();
    let db_path = data_dir.join(DB_FILE_NAME).to_string_lossy().into_owned();
    let log_dir = data_dir.join(LOG_DIR_NAME).to_string_lossy().into_owned();

    // 开发构建：进程带控制台（见 main.rs 的 windows_subsystem 条件），日志只输出到控制台、不落盘；
    // 部署构建：GUI 子系统没有控制台，日志只落盘（否则等于什么都不留）。
    // 用 debug_assertions 而不是 tauri::is_dev()：前者是确定性的 profile 开关，且与"有没有控制台"
    // 是同一个条件；后者依赖 tauri/custom-protocol 特性是否被 CLI 打开。
    let is_dev = cfg!(debug_assertions);
    let log_level = if is_dev { "debug" } else { "warn" };

    let mut config = Value::Table(toml! {
        [app]
        name = APP_DIR_NAME

        [logger]
        level = log_level
        enable_console = is_dev
        max_file_number = 7

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
        default_language = "zh"
        mods_folder_name = "mods"
        workshop_folder_prefix = "workshop-"
        steamapps_folder_name = "steamapps"
        common_folder_name = "common"
        workshop_folder_name = "workshop"
        content_folder_name = "content"
        library_folders_file = "libraryfolders.vdf"
        steam_registry_key = "Software\\Valve\\Steam"
        steam_path_value_name = "SteamPath"
        steam_default_install_path = "C:\\Program Files (x86)\\Steam"
        server_watch_tick_secs = 5
        server_watch_miss_threshold = 2
        server_watch_startup_grace_secs = 90

        [database]
        db_path = db_path
        sqlx_logging = true
        max_connections = 1
        min_connections = 1
        time_out = 10

        [runtime]
        worker_thread_num = 1
    });

    // 只有部署构建才配日志文件目录（开发构建用控制台，不产生日志文件）
    if !is_dev {
        if let Some(logger) = config.get_mut("logger").and_then(Value::as_table_mut) {
            logger.insert("log_dir".to_string(), Value::String(log_dir));
        }
    }

    config
}

/// 解析数据目录：优先程序所在目录，不可写时退回本地数据目录
///
/// 程序目录不可写的典型场景是把 MSI 装到了 `Program Files`（需要管理员权限才能写入）。
/// 那种情况下静默启动失败最难排查，所以这里探测一次可写性并退到 `%LOCALAPPDATA%`。
fn resolve_data_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if is_writable(exe_dir) {
                return exe_dir.to_path_buf();
            }
            eprintln!(
                "WARN: 程序目录不可写（{}），数据与日志改放到本地数据目录",
                exe_dir.display()
            );
        }
    }

    dirs::data_local_dir()
        .map(|base| base.join(APP_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 目录是否可写（试写并删除一个临时文件）
fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(".dst_write_probe");
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}
