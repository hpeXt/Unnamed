//! 插件核心接口定义
//!
//! 提供统一的插件开发接口

use crate::error::PluginResult;
use crate::message::PluginMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 插件作者
    pub author: Option<String>,
    /// 依赖的插件列表
    pub dependencies: Vec<String>,
    /// 插件标签
    pub tags: Vec<String>,
    /// 插件配置 schema
    pub config_schema: Option<serde_json::Value>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            version: "0.1.0".to_string(),
            description: "A minimal kernel plugin".to_string(),
            author: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
            config_schema: None,
        }
    }
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 配置数据
    pub data: HashMap<String, serde_json::Value>,
    /// 是否启用
    pub enabled: bool,
    /// 日志级别
    pub log_level: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            enabled: true,
            log_level: "info".to_string(),
        }
    }
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 未初始化
    Uninitialized,
    /// 正在初始化
    Initializing,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 正在关闭
    Shutting,
    /// 已关闭
    Shutdown,
    /// 错误状态
    Error,
}

/// 插件生命周期事件
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// 初始化事件
    Initialize,
    /// 配置更新事件
    ConfigUpdate(PluginConfig),
    /// 消息事件
    Message(PluginMessage),
    /// 定时器事件
    Timer(String),
    /// 关闭事件
    Shutdown,
}

/// 核心插件接口
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata;
    
    /// 获取插件当前状态
    fn status(&self) -> PluginStatus;
    
    /// 初始化插件
    /// 
    /// 这个方法在插件加载时调用，用于初始化资源、订阅主题等
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()>;
    
    /// 处理插件事件
    /// 
    /// 这是插件的主要处理逻辑，所有事件都通过这个方法处理
    fn handle_event(&mut self, event: PluginEvent) -> PluginResult<()>;
    
    /// 插件定时任务
    /// 
    /// 这个方法会定期被调用，用于执行周期性任务
    fn tick(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    /// 处理插件消息
    /// 
    /// 这是一个便捷方法，用于处理来自其他插件的消息
    fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()> {
        self.handle_event(PluginEvent::Message(message))
    }
    
    /// 获取插件配置
    fn get_config(&self) -> Option<&PluginConfig>;
    
    /// 更新插件配置
    fn update_config(&mut self, config: PluginConfig) -> PluginResult<()> {
        self.handle_event(PluginEvent::ConfigUpdate(config))
    }
    
    /// 暂停插件
    fn pause(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    /// 恢复插件
    fn resume(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    /// 关闭插件
    /// 
    /// 这个方法在插件卸载时调用，用于清理资源
    fn shutdown(&mut self) -> PluginResult<()> {
        self.handle_event(PluginEvent::Shutdown)
    }
    
    /// 获取插件健康状态
    fn health_check(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
    
    /// 获取插件统计信息
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
}

/// 基础插件实现
/// 
/// 提供插件的基本功能，可以作为其他插件的基础
pub struct BasePlugin {
    metadata: PluginMetadata,
    config: Option<PluginConfig>,
    status: PluginStatus,
    stats: HashMap<String, serde_json::Value>,
}

impl BasePlugin {
    /// 创建新的基础插件
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            config: None,
            status: PluginStatus::Uninitialized,
            stats: HashMap::new(),
        }
    }
    
    /// 设置插件状态
    pub fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
    
    /// 更新统计信息
    pub fn update_stat(&mut self, key: &str, value: serde_json::Value) {
        self.stats.insert(key.to_string(), value);
    }
}

impl Plugin for BasePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        self.config = Some(config);
        self.status = PluginStatus::Running;
        Ok(())
    }
    
    fn handle_event(&mut self, _event: PluginEvent) -> PluginResult<()> {
        Ok(())
    }
    
    fn get_config(&self) -> Option<&PluginConfig> {
        self.config.as_ref()
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        self.status = PluginStatus::Shutdown;
        Ok(())
    }
    
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(self.stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: Some("tester".to_string()),
            dependencies: vec!["dep1".to_string()],
            tags: vec!["test".to_string()],
            config_schema: None,
        };
        
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dependencies.len(), 1);
    }
    
    #[test]
    fn test_base_plugin() {
        let metadata = PluginMetadata::default();
        let mut plugin = BasePlugin::new(metadata);
        
        assert_eq!(plugin.status(), PluginStatus::Uninitialized);
        
        let config = PluginConfig::default();
        plugin.initialize(config).unwrap();
        
        assert_eq!(plugin.status(), PluginStatus::Running);
        assert!(plugin.get_config().is_some());
    }
}