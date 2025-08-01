//! 插件错误处理
//!
//! 提供统一的错误类型和处理机制

use thiserror::Error;

/// 插件错误类型
#[derive(Error, Debug)]
pub enum PluginError {
    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 主机函数调用错误
    #[error("Host function error: {0}")]
    HostFunction(String),

    /// 初始化错误
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// 消息处理错误
    #[error("Message processing error: {0}")]
    MessageProcessing(String),

    /// 存储错误
    #[error("Storage error: {0}")]
    Storage(String),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// 权限错误
    #[error("Permission error: {0}")]
    Permission(String),

    /// 资源不足错误
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// 超时错误
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// 依赖错误
    #[error("Dependency error: {0}")]
    Dependency(String),

    /// 插件已关闭
    #[error("Plugin is shutdown")]
    PluginShutdown,

    /// 插件状态错误
    #[error("Invalid plugin state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    /// 不支持的操作
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// 通用错误
    #[error("Generic error: {0}")]
    Generic(String),

    /// 包装其他错误
    #[error("External error: {0}")]
    External(#[from] anyhow::Error),
}

/// 插件结果类型
pub type PluginResult<T> = Result<T, PluginError>;

/// 错误上下文，用于提供更多的错误信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 插件名称
    pub plugin_name: String,
    /// 操作名称
    pub operation: String,
    /// 错误发生时间戳（毫秒）
    pub timestamp: u64,
    /// 额外的上下文信息
    pub context: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(plugin_name: &str, operation: &str) -> Self {
        Self {
            plugin_name: plugin_name.to_string(),
            operation: operation.to_string(),
            timestamp: crate::utils::time::now_millis(),
            context: std::collections::HashMap::new(),
        }
    }

    /// 添加上下文信息
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

/// 错误处理辅助 trait
pub trait PluginErrorExt<T> {
    /// 添加错误上下文
    fn with_context(self, context: ErrorContext) -> PluginResult<T>;

    /// 添加简单上下文
    fn with_plugin_context(self, plugin_name: &str, operation: &str) -> PluginResult<T>;
}

impl<T> PluginErrorExt<T> for PluginResult<T> {
    fn with_context(self, context: ErrorContext) -> PluginResult<T> {
        self.map_err(|e| {
            PluginError::Generic(format!(
                "Plugin '{}' operation '{}' failed at {}: {}",
                context.plugin_name, context.operation, context.timestamp, e
            ))
        })
    }

    fn with_plugin_context(self, plugin_name: &str, operation: &str) -> PluginResult<T> {
        let context = ErrorContext::new(plugin_name, operation);
        self.with_context(context)
    }
}

/// 便捷的错误创建宏
#[macro_export]
macro_rules! plugin_error {
    ($kind:ident, $msg:expr) => {
        $crate::error::PluginError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::PluginError::$kind(format!($fmt, $($arg)*))
    };
}

/// 条件检查宏
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

/// 将 FromUtf8Error 转换为插件错误
impl From<std::string::FromUtf8Error> for PluginError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        PluginError::MessageProcessing(format!("String conversion error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_creation() {
        let error = plugin_error!(Configuration, "Invalid config");
        assert!(matches!(error, PluginError::Configuration(_)));
    }

    #[test]
    fn test_plugin_error_formatting() {
        let error = plugin_error!(MessageProcessing, "Failed to process message: {}", "test");
        assert!(error
            .to_string()
            .contains("Failed to process message: test"));
    }

    #[test]
    fn test_error_context() {
        let context =
            ErrorContext::new("test_plugin", "initialize").with_context("config", "test_config");

        assert_eq!(context.plugin_name, "test_plugin");
        assert_eq!(context.operation, "initialize");
        assert_eq!(
            context.context.get("config"),
            Some(&"test_config".to_string())
        );
    }
}
