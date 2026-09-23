//! 服务器运行期崩档互保监视。
//!
//! 语义：**只接管本应用启动过的服务器**（`start_server` 成功后写 `server_runtime` 表）。
//! 本任务按 tick 探测各地面/洞穴窗口：
//!
//! - 期望的分片都出现过 → 状态置 `running`；
//! - `running` 下某分片连续多次探测不到（崩档）→ 向仍存活的另一侧发送 `c_shutdown()`，状态置 `stopping`；
//! - 启动宽限期内始终集不齐期望分片 → 视为启动异常，同样关闭已出现的分片；
//! - 两侧窗口都不在（正常退出、崩档后对端也退了、用户手动关闭）→ 删除记录。
//!
//! 表是「应用维护的监视会话 + 最近一轮探测快照」，真相源始终是窗口存在性（快照最多滞后一个 tick）。
//! 前端状态展示读这张表；状态变化时通过 Tauri 事件推送（事件名 [`STATUS_EVENT`]，前端 `listen` 同名事件）。

use crate::entity::server_runtime;
use crate::service::{STATUS_RUNNING, STATUS_STARTING, STATUS_STOPPING, ServerRuntimeService};
use crate::utils::{app_component, find_and_send_shutdown_command, find_window_by_title};
use crate::{ConstantComponent, SimpleAppWebError};
use serde::Serialize;
use simple_starter_core::anyhow;
use simple_starter_core::tracing::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

/// 状态变化事件名（前端 `listen` 同名事件；跨端契约，改动需同步前端）
pub const STATUS_EVENT: &str = "server-status-changed";

/// 关闭命令最多发送次数（超过后只告警一次，交给用户处理）
const MAX_CLOSE_ATTEMPTS: u8 = 3;

/// 全局 AppHandle（主循环钩子里安装），用于向 WebView 推送状态变化
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// 由 Tauri 主循环钩子调用，安装事件推送所需的句柄
pub fn set_app_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

/// 状态变化事件负载
#[derive(Serialize, Clone)]
struct StatusEvent {
    server_id: String,
    /// `starting` / `running` / `stopping` / `stopped`
    status: String,
    /// 触发原因：`peer-crashed` 分片崩档、`startup-failed` 启动异常
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

fn emit_status(server_id: &str, status: &str, reason: Option<&'static str>) {
    let Some(handle) = APP_HANDLE.get() else {
        // 主循环尚未安装句柄（启动极早期）：跳过推送，前端首帧仍会读表
        return;
    };

    let payload = StatusEvent {
        server_id: server_id.to_string(),
        status: status.to_string(),
        reason,
    };

    if let Err(e) = handle.emit(STATUS_EVENT, payload) {
        warn!("推送服务器状态事件失败: {}", e);
    }
}

const SHARD_MASTER: &str = "master";
const SHARD_CAVES: &str = "caves";

/// 每个受管服务器的观测态（不入库：应用重启后重新观察即可）
struct Obs {
    /// 期望的分片是否都至少出现过
    armed: bool,
    /// 启动宽限截止时间（未 armed 期间不判定崩档）
    grace_until: Instant,
    /// 各分片连续缺失次数
    miss: HashMap<&'static str, u32>,
    /// 关闭命令已发送次数
    close_attempts: u8,
}

impl Obs {
    fn new(grace: Duration) -> Self {
        Self {
            armed: false,
            grace_until: Instant::now() + grace,
            miss: HashMap::new(),
            close_attempts: 0,
        }
    }
}

/// 监视任务主体：在启动钩子里经 `add_task_spawn_factory_in_context` 注册，随应用生命周期运行
pub async fn run(token: CancellationToken) -> anyhow::Result<()> {
    let cc = match app_component::<ConstantComponent>() {
        Ok(cc) => cc,
        Err(e) => {
            error!("崩档监视启动失败：拿不到 ConstantComponent（{:?}）", e);
            return Ok(());
        }
    };

    let tick = Duration::from_secs(cc.server_watch_tick_secs.max(1));
    let threshold = cc.server_watch_miss_threshold.max(1);
    let grace = Duration::from_secs(cc.server_watch_startup_grace_secs);

    info!(
        "崩档监视已启动：间隔={:?} 缺失阈值={} 启动宽限={:?}",
        tick, threshold, grace
    );

    let mut obs: HashMap<String, Obs> = HashMap::new();
    let mut interval = tokio::time::interval(tick);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                info!("崩档监视已停止");
                return Ok(());
            }
            _ = interval.tick() => {
                if let Err(e) = tick_once(&cc, threshold, grace, &mut obs).await {
                    error!("崩档监视本轮执行失败: {}", e.message);
                }
            }
        }
    }
}

/// 单轮检查
async fn tick_once(
    cc: &ConstantComponent,
    threshold: u32,
    grace: Duration,
    obs: &mut HashMap<String, Obs>,
) -> Result<(), SimpleAppWebError> {
    let service: Arc<ServerRuntimeService> = app_component()?;
    let rows = service.list().await?;

    // 已不在表里的条目清掉观测态
    obs.retain(|server_id, _| rows.iter().any(|row| &row.server_id == server_id));

    for row in rows {
        let master_title = format!("{}_{}", row.server_id, cc.master_shard_name);
        let caves_title = format!("{}_{}", row.server_id, cc.caves_shard_name);
        let master_alive = find_window_by_title(&master_title);
        let caves_alive = find_window_by_title(&caves_title);

        let expect_caves = row.expect_caves;
        let alive = |shard: &str| match shard {
            SHARD_MASTER => master_alive,
            _ => caves_alive,
        };
        let expected: &[&str] = if expect_caves {
            &[SHARD_MASTER, SHARD_CAVES]
        } else {
            &[SHARD_MASTER]
        };

        // 完全退出：正常停止已完成、用户手动关闭、或启动就没起来
        if !master_alive && !caves_alive {
            let entry = obs
                .entry(row.server_id.clone())
                .or_insert_with(|| Obs::new(grace));

            // 刚启动（starting）且仍在宽限期内：窗口还没起来是正常的，先等等
            if row.status == STATUS_STARTING && Instant::now() < entry.grace_until {
                continue;
            }

            service.remove(&row.server_id).await?;
            obs.remove(&row.server_id);
            info!(server_id = %row.server_id, "服务器已完全退出，移除运行记录");
            emit_status(&row.server_id, "stopped", None);
            continue;
        }

        let entry = obs
            .entry(row.server_id.clone())
            .or_insert_with(|| Obs::new(grace));
        let missing: Vec<&str> = expected.iter().copied().filter(|s| !alive(s)).collect();

        // 尚未集齐期望分片：先尝试"武装"，超宽限仍未齐则按启动异常处理
        if !entry.armed {
            if missing.is_empty() {
                entry.armed = true;
                entry.miss.clear();
                if row.status != STATUS_RUNNING {
                    service.set_status(&row.server_id, STATUS_RUNNING).await?;
                    emit_status(&row.server_id, STATUS_RUNNING, None);
                }
            } else if Instant::now() >= entry.grace_until {
                warn!(
                    server_id = %row.server_id,
                    missing = ?missing,
                    "启动宽限期结束仍未集齐期望分片，按启动异常关闭已出现的分片"
                );
                close_alive(
                    &row,
                    &master_title,
                    &caves_title,
                    master_alive,
                    caves_alive,
                    entry,
                )
                .await;

                if row.status != STATUS_STOPPING {
                    service.set_status(&row.server_id, STATUS_STOPPING).await?;
                    emit_status(&row.server_id, STATUS_STOPPING, Some("startup-failed"));
                }
            }
            continue;
        }

        // 已进入监视：统计各期望分片的连续缺失次数
        let mut crashed = false;
        for shard in expected {
            if alive(shard) {
                entry.miss.remove(shard);
            } else {
                let counter = entry.miss.entry(shard).or_insert(0);
                *counter += 1;
                if *counter >= threshold {
                    crashed = true;
                }
            }
        }

        if !crashed {
            continue;
        }

        // 崩档（或正常停止中某侧已先退出）：向仍存活的分片补一刀，幂等
        close_alive(
            &row,
            &master_title,
            &caves_title,
            master_alive,
            caves_alive,
            entry,
        )
        .await;

        if row.status != STATUS_STOPPING {
            warn!(
                server_id = %row.server_id,
                missing = ?missing,
                "检测到分片崩档，关闭仍存活的对端"
            );
            service.set_status(&row.server_id, STATUS_STOPPING).await?;
            emit_status(&row.server_id, STATUS_STOPPING, Some("peer-crashed"));
        }
    }

    Ok(())
}

/// 向仍存活的分片发送关闭命令（最多 [`MAX_CLOSE_ATTEMPTS`] 次）
///
/// `find_and_send_shutdown_command` 内部有阻塞等待与 SendInput，因此走 `spawn_blocking`
/// ——后端只有 1 个 worker 线程，直接阻塞会卡住所有 IPC。
async fn close_alive(
    row: &server_runtime::Model,
    master_title: &str,
    caves_title: &str,
    master_alive: bool,
    caves_alive: bool,
    entry: &mut Obs,
) {
    if entry.close_attempts >= MAX_CLOSE_ATTEMPTS {
        if entry.close_attempts == MAX_CLOSE_ATTEMPTS {
            warn!(
                server_id = %row.server_id,
                "关闭命令已发送 {} 次仍未生效，停止重试，请手动处理",
                MAX_CLOSE_ATTEMPTS
            );
            entry.close_attempts += 1;
        }
        return;
    }
    entry.close_attempts += 1;

    let mut targets: Vec<String> = Vec::new();
    if master_alive {
        targets.push(master_title.to_string());
    }
    if caves_alive && row.expect_caves {
        targets.push(caves_title.to_string());
    }

    for title in targets {
        let title_for_task = title.clone();
        let result =
            tokio::task::spawn_blocking(move || find_and_send_shutdown_command(&title_for_task)).await;

        match result {
            Ok(Ok(true)) => info!(server_id = %row.server_id, window = %title, "已发送关闭命令"),
            Ok(Ok(false)) => warn!(server_id = %row.server_id, window = %title, "窗口未找到，跳过"),
            Ok(Err(e)) => warn!(server_id = %row.server_id, window = %title, "关闭命令发送失败: {}", e.message),
            Err(e) => warn!(server_id = %row.server_id, window = %title, "关闭命令任务异常: {}", e),
        }
    }
}
