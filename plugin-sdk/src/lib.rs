//! 最小化内核插件 SDK
//!
//! 提供插件开发的基础类型和工具

// 重新导出常用依赖
pub use extism_pdk::*;
pub use serde::{Deserialize, Serialize};
pub use serde_json;

// 导出核心模块
pub mod error;
pub mod host;
pub mod macros;
pub mod message;
pub mod plugin;
pub mod utils;

// 导出测试辅助（仅在测试时）
#[cfg(test)]
pub mod testing;

// 便捷的重新导出
pub use error::{ErrorContext, PluginError, PluginErrorExt, PluginResult};
pub use host::LogLevel;
pub use message::{MessageBuilder, MessageFilter, MessageHandler, MessagePriority, PluginMessage};
pub use plugin::{BasePlugin, Plugin, PluginConfig, PluginEvent, PluginMetadata, PluginStatus};

/// 插件 SDK 版本
pub const SDK_VERSION: &str = "0.1.0";

/// 插件 SDK 预导入
///
/// 包含插件开发最常用的类型和宏
pub mod prelude {
    pub use crate::error::*;
    pub use crate::host;
    pub use crate::message::*;
    pub use crate::plugin::*;
    pub use crate::utils::*;
    pub use crate::{debug_log, ensure, plugin_error, time_it, try_or_log};
    pub use crate::{get_data, store_data, subscribe_topics};
    pub use crate::{log_debug, log_error, log_info, log_trace, log_warn};
    pub use crate::{plugin_handler, plugin_info, plugin_json_handler, plugin_main};
    pub use extism_pdk::*;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json;
}
