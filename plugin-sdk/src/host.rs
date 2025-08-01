//! 主机函数包装
//!
//! 提供对主机函数的高级封装，简化插件开发

use crate::error::{PluginError, PluginResult};
use crate::message::PluginMessage;
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// 声明主机函数
/// 
/// 重要提示：主机系统注册函数时使用了 "_host" 后缀
/// 例如：主机注册的是 "log_message_host"，插件必须使用完整的函数名
/// 根据 Extism 官方文档，#[host_fn] 宏应该会自动添加 "_host" 后缀，
/// 但在我们的系统中，需要显式声明带后缀的函数名
#[host_fn]
extern "ExtismHost" {
    fn store_data_host(plugin_id: &str, key: &str, value: &str) -> String;
    fn get_data_host(plugin_id: &str, key: &str) -> String;
    fn delete_data_host(plugin_id: &str, key: &str) -> String;
    fn list_keys_host(plugin_id: &str) -> String;
    fn send_message_host(from: &str, to: &str, payload: &str) -> String;
    fn log_message_host(level: &str, message: &str) -> String;
    fn sign_message_host(plugin_id: &str, message: &str) -> String;
    fn verify_signature_host(plugin_id: &str, message: &str, signature: &str) -> String;
    fn get_plugin_address_host(plugin_id: &str) -> String;
    fn subscribe_topic_host(plugin_id: &str, topic: &str) -> String;
    fn unsubscribe_topic_host(plugin_id: &str, topic: &str) -> String;
    fn publish_message_host(plugin_id: &str, topic: &str, payload: &str) -> String;
    fn get_config_host(plugin_id: &str) -> String;
    fn set_config_host(plugin_id: &str, config: &str) -> String;
}

/// 主机函数响应结构
#[derive(Debug, Serialize, Deserialize)]
struct HostResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// 存储操作
pub mod storage {
    use super::*;
    
    /// 存储数据
    pub fn store<T: Serialize>(plugin_id: &str, key: &str, value: &T) -> PluginResult<()> {
        let json_value = serde_json::to_string(value)?;
        let result = unsafe { store_data_host(plugin_id, key, &json_value)? };
        
        let response: HostResponse<()> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(())
        } else {
            Err(PluginError::Storage(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 获取数据
    pub fn get<T: for<'de> Deserialize<'de>>(plugin_id: &str, key: &str) -> PluginResult<Option<T>> {
        let result = unsafe { get_data_host(plugin_id, key)? };
        
        let response: HostResponse<serde_json::Value> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            if let Some(value) = response.data {
                let typed_value = serde_json::from_value(value)?;
                Ok(Some(typed_value))
            } else {
                Ok(None)
            }
        } else {
            Err(PluginError::Storage(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 删除数据
    pub fn delete(plugin_id: &str, key: &str) -> PluginResult<bool> {
        let result = unsafe { delete_data_host(plugin_id, key)? };
        
        let response: HostResponse<bool> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(response.data.unwrap_or(false))
        } else {
            Err(PluginError::Storage(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 列出所有键
    pub fn list(plugin_id: &str) -> PluginResult<Vec<String>> {
        let result = unsafe { list_keys_host(plugin_id)? };
        
        let response: HostResponse<Vec<String>> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(PluginError::Storage(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 检查键是否存在
    pub fn exists(plugin_id: &str, key: &str) -> PluginResult<bool> {
        let keys = list(plugin_id)?;
        Ok(keys.contains(&key.to_string()))
    }
    
    /// 批量存储
    pub fn store_batch<T: Serialize>(plugin_id: &str, data: HashMap<String, T>) -> PluginResult<()> {
        for (key, value) in data {
            store(plugin_id, &key, &value)?;
        }
        Ok(())
    }
    
    /// 批量获取
    pub fn get_batch<T: for<'de> Deserialize<'de>>(plugin_id: &str, keys: &[String]) -> PluginResult<HashMap<String, T>> {
        let mut result = HashMap::new();
        for key in keys {
            if let Some(value) = get(plugin_id, key)? {
                result.insert(key.clone(), value);
            }
        }
        Ok(result)
    }
}

/// 消息操作
pub mod messaging {
    use super::*;
    
    /// 发送消息
    pub fn send(message: &PluginMessage) -> PluginResult<String> {
        let payload = serde_json::to_string(message)?;
        let result = unsafe { send_message_host(&message.from, &message.to, &payload)? };
        
        let response: HostResponse<String> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(PluginError::MessageProcessing(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 发送简单消息
    pub fn send_simple(from: &str, to: &str, payload: &str) -> PluginResult<String> {
        let message = PluginMessage::builder(from)
            .to(to)
            .payload_string(payload)
            .build()
            .map_err(|e| PluginError::MessageProcessing(e))?;
        
        send(&message)
    }
    
    /// 发送 JSON 消息
    pub fn send_json<T: Serialize>(from: &str, to: &str, payload: &T) -> PluginResult<String> {
        let message = PluginMessage::builder(from)
            .to(to)
            .payload_json(payload)?
            .build()
            .map_err(|e| PluginError::MessageProcessing(e))?;
        
        send(&message)
    }
    
    /// 订阅主题
    pub fn subscribe(plugin_id: &str, topic: &str) -> PluginResult<()> {
        let result = unsafe { subscribe_topic_host(plugin_id, topic)? };
        
        let response: HostResponse<()> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(())
        } else {
            Err(PluginError::MessageProcessing(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 取消订阅主题
    pub fn unsubscribe(plugin_id: &str, topic: &str) -> PluginResult<()> {
        let result = unsafe { unsubscribe_topic_host(plugin_id, topic)? };
        
        let response: HostResponse<()> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(())
        } else {
            Err(PluginError::MessageProcessing(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 发布消息到主题
    pub fn publish<T: Serialize>(plugin_id: &str, topic: &str, payload: &T) -> PluginResult<String> {
        let json_payload = serde_json::to_string(payload)?;
        let result = unsafe { publish_message_host(plugin_id, topic, &json_payload)? };
        
        let response: HostResponse<String> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(PluginError::MessageProcessing(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
}

/// 日志操作
pub mod logging {
    use super::*;
    
    /// 记录日志
    pub fn log(level: LogLevel, message: &str) -> PluginResult<()> {
        let result = unsafe { log_message_host(&level.to_string(), message)? };
        
        let response: HostResponse<()> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(())
        } else {
            Err(PluginError::Generic(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 记录错误日志
    pub fn error(message: &str) -> PluginResult<()> {
        log(LogLevel::Error, message)
    }
    
    /// 记录警告日志
    pub fn warn(message: &str) -> PluginResult<()> {
        log(LogLevel::Warn, message)
    }
    
    /// 记录信息日志
    pub fn info(message: &str) -> PluginResult<()> {
        log(LogLevel::Info, message)
    }
    
    /// 记录调试日志
    pub fn debug(message: &str) -> PluginResult<()> {
        log(LogLevel::Debug, message)
    }
    
    /// 记录跟踪日志
    pub fn trace(message: &str) -> PluginResult<()> {
        log(LogLevel::Trace, message)
    }
}

/// 配置操作
pub mod config {
    use super::*;
    
    /// 获取配置
    pub fn get<T: for<'de> Deserialize<'de>>(plugin_id: &str) -> PluginResult<Option<T>> {
        let result = unsafe { get_config_host(plugin_id)? };
        
        let response: HostResponse<serde_json::Value> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            if let Some(value) = response.data {
                let typed_value = serde_json::from_value(value)?;
                Ok(Some(typed_value))
            } else {
                Ok(None)
            }
        } else {
            Err(PluginError::Configuration(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
    
    /// 设置配置
    pub fn set<T: Serialize>(plugin_id: &str, config: &T) -> PluginResult<()> {
        let json_config = serde_json::to_string(config)?;
        let result = unsafe { set_config_host(plugin_id, &json_config)? };
        
        let response: HostResponse<()> = serde_json::from_str(&result)
            .map_err(|e| PluginError::HostFunction(format!("Failed to parse response: {}", e)))?;
        
        if response.success {
            Ok(())
        } else {
            Err(PluginError::Configuration(response.error.unwrap_or("Unknown error".to_string())))
        }
    }
}

/// 便捷的日志宏
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        let _ = $crate::host::logging::error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        let _ = $crate::host::logging::warn(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        let _ = $crate::host::logging::info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        let _ = $crate::host::logging::debug(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        let _ = $crate::host::logging::trace(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Trace.to_string(), "trace");
    }
}