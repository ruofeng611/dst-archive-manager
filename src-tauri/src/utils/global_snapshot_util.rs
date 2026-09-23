//! 全局上下文快照访问工具。
//!
//! simple-starter-core 提供全局静态快照（`app_container` / `app_config`），
//! 供无上下文传播的场景读取。Tauri 命令处理器无法注入组件，
//! 统一经此工具获取运行期组件。

use simple_starter_core::app_container;
use simple_starter_core::ComponentError;
use std::any::Any;
use std::sync::Arc;

/// 经全局上下文快照获取组件（运行期合法窗口内可用）
///
/// 快照未安装（组件装配完成前调用）时 panic：命令处理器在应用运行期
/// 才被调用，此时快照必然已安装，违反即编程错误，fail-fast。
/// 组件未注册时返回 Err（组件查询失败，可预期错误）。
pub fn app_component<T>() -> Result<Arc<T>, ComponentError>
where
    T: Any + Send + Sync + 'static,
{
    app_container()
        .expect("global container snapshot must be installed before component access")
        .get_component::<T>()
}
