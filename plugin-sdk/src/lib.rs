//! 最小化内核插件 SDK
//! 
//! 提供插件开发的基础类型和工具

// 重新导出常用依赖
pub use extism_pdk::*;
pub use serde::{Deserialize, Serialize};
pub use serde_json;

// 导出核心模块
pub mod plugin;
pub mod error;
pub mod message;
pub mod host;
pub mod macros;
pub mod utils;

// 导出测试辅助（仅在测试时）
#[cfg(test)]
pub mod testing;

// 便捷的重新导出
pub use plugin::{Plugin, PluginMetadata, PluginConfig, PluginStatus, PluginEvent, BasePlugin};
pub use error::{PluginError, PluginResult, ErrorContext, PluginErrorExt};
pub use message::{PluginMessage, MessagePriority, MessageBuilder, MessageHandler, MessageFilter};
pub use host::LogLevel;

/// 插件 SDK 版本
pub const SDK_VERSION: &str = "0.1.0";

/// 插件 SDK 预导入
/// 
/// 包含插件开发最常用的类型和宏
pub mod prelude {
    pub use crate::plugin::*;
    pub use crate::error::*;
    pub use crate::message::*;
    pub use crate::host;
    pub use crate::utils::*;
    pub use crate::{plugin_main, plugin_handler, plugin_json_handler, plugin_info};
    pub use crate::{log_error, log_warn, log_info, log_debug, log_trace};
    pub use crate::{store_data, get_data, subscribe_topics};
    pub use crate::{plugin_error, ensure, try_or_log, debug_log, time_it};
    pub use extism_pdk::*;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json;
}