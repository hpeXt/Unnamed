//! 插件测试辅助工具
//!
//! 提供插件开发和测试中的辅助工具和模拟对象

use crate::plugin::*;
use crate::message::*;
use crate::error::*;
use crate::utils::time;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 模拟插件，用于测试
pub struct MockPlugin {
    metadata: PluginMetadata,
    config: Option<PluginConfig>,
    status: PluginStatus,
    messages: Vec<PluginMessage>,
    events: Vec<PluginEvent>,
    stats: HashMap<String, serde_json::Value>,
    fail_on_init: bool,
    fail_on_message: bool,
    fail_on_shutdown: bool,
}

impl MockPlugin {
    /// 创建新的模拟插件
    pub fn new(name: &str) -> Self {
        let metadata = PluginMetadata {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Mock plugin for testing".to_string(),
            author: Some("Test Framework".to_string()),
            dependencies: Vec::new(),
            tags: vec!["mock".to_string(), "test".to_string()],
            config_schema: None,
        };
        
        Self {
            metadata,
            config: None,
            status: PluginStatus::Uninitialized,
            messages: Vec::new(),
            events: Vec::new(),
            stats: HashMap::new(),
            fail_on_init: false,
            fail_on_message: false,
            fail_on_shutdown: false,
        }
    }
    
    /// 设置初始化失败
    pub fn fail_on_init(mut self, fail: bool) -> Self {
        self.fail_on_init = fail;
        self
    }
    
    /// 设置消息处理失败
    pub fn fail_on_message(mut self, fail: bool) -> Self {
        self.fail_on_message = fail;
        self
    }
    
    /// 设置关闭失败
    pub fn fail_on_shutdown(mut self, fail: bool) -> Self {
        self.fail_on_shutdown = fail;
        self
    }
    
    /// 获取收到的消息
    pub fn received_messages(&self) -> &[PluginMessage] {
        &self.messages
    }
    
    /// 获取收到的事件
    pub fn received_events(&self) -> &[PluginEvent] {
        &self.events
    }
    
    /// 清空消息历史
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
    
    /// 清空事件历史
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
    
    /// 添加统计信息
    pub fn add_stat(&mut self, key: &str, value: serde_json::Value) {
        self.stats.insert(key.to_string(), value);
    }
}

impl Plugin for MockPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        if self.fail_on_init {
            return Err(PluginError::Initialization("Mock initialization failure".to_string()));
        }
        
        self.config = Some(config);
        self.status = PluginStatus::Running;
        Ok(())
    }
    
    fn handle_event(&mut self, event: PluginEvent) -> PluginResult<()> {
        self.events.push(event.clone());
        
        match event {
            PluginEvent::Message(msg) => {
                if self.fail_on_message {
                    return Err(PluginError::MessageProcessing("Mock message processing failure".to_string()));
                }
                self.messages.push(msg);
            }
            PluginEvent::Shutdown => {
                if self.fail_on_shutdown {
                    return Err(PluginError::Generic("Mock shutdown failure".to_string()));
                }
                self.status = PluginStatus::Shutdown;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn get_config(&self) -> Option<&PluginConfig> {
        self.config.as_ref()
    }
    
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(self.stats.clone())
    }
}

/// 测试消息构建器
pub struct TestMessageBuilder;

impl TestMessageBuilder {
    /// 创建简单的测试消息
    pub fn simple(from: &str, to: &str, content: &str) -> PluginMessage {
        PluginMessage::builder(from)
            .to(to)
            .topic("test")
            .payload_string(content)
            .build()
            .unwrap()
    }
    
    /// 创建 JSON 测试消息
    pub fn json<T: Serialize>(from: &str, to: &str, payload: &T) -> PluginMessage {
        PluginMessage::builder(from)
            .to(to)
            .topic("test")
            .payload_json(payload)
            .unwrap()
            .build()
            .unwrap()
    }
    
    /// 创建高优先级测试消息
    pub fn high_priority(from: &str, to: &str, content: &str) -> PluginMessage {
        PluginMessage::builder(from)
            .to(to)
            .topic("urgent")
            .payload_string(content)
            .priority(MessagePriority::High)
            .build()
            .unwrap()
    }
    
    /// 创建带过期时间的测试消息
    pub fn with_ttl(from: &str, to: &str, content: &str, ttl_secs: i64) -> PluginMessage {
        PluginMessage::builder(from)
            .to(to)
            .topic("test")
            .payload_string(content)
            .ttl(ttl_secs as u64)
            .build()
            .unwrap()
    }
    
    /// 创建已过期的测试消息
    pub fn expired(from: &str, to: &str, content: &str) -> PluginMessage {
        PluginMessage::builder(from)
            .to(to)
            .topic("test")
            .payload_string(content)
            .expires_at(crate::utils::time::now_millis() - 1000)
            .build()
            .unwrap()
    }
}

/// 测试配置构建器
pub struct TestConfigBuilder {
    data: HashMap<String, serde_json::Value>,
    enabled: bool,
    log_level: String,
}

impl TestConfigBuilder {
    /// 创建新的测试配置构建器
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            enabled: true,
            log_level: "info".to_string(),
        }
    }
    
    /// 添加字符串配置
    pub fn add_string(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        self
    }
    
    /// 添加数字配置
    pub fn add_number<T: Into<serde_json::Number>>(mut self, key: &str, value: T) -> Self {
        self.data.insert(key.to_string(), serde_json::Value::Number(value.into()));
        self
    }
    
    /// 添加布尔配置
    pub fn add_bool(mut self, key: &str, value: bool) -> Self {
        self.data.insert(key.to_string(), serde_json::Value::Bool(value));
        self
    }
    
    /// 添加数组配置
    pub fn add_array<T: Serialize>(mut self, key: &str, value: &[T]) -> Self {
        self.data.insert(key.to_string(), serde_json::to_value(value).unwrap_or_default());
        self
    }
    
    /// 添加 JSON 配置
    pub fn add_json<T: Serialize>(mut self, key: &str, value: &T) -> Self {
        self.data.insert(key.to_string(), serde_json::to_value(value).unwrap_or_default());
        self
    }
    
    /// 设置启用状态
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// 设置日志级别
    pub fn log_level(mut self, level: &str) -> Self {
        self.log_level = level.to_string();
        self
    }
    
    /// 构建配置
    pub fn build(self) -> PluginConfig {
        PluginConfig {
            data: self.data,
            enabled: self.enabled,
            log_level: self.log_level,
        }
    }
}

/// 内存存储模拟器
pub struct MockStorage {
    data: Arc<Mutex<HashMap<String, HashMap<String, serde_json::Value>>>>,
}

impl MockStorage {
    /// 创建新的模拟存储
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 存储数据
    pub fn store<T: Serialize>(&self, plugin_id: &str, key: &str, value: &T) -> PluginResult<()> {
        let json_value = serde_json::to_value(value)?;
        let mut data = self.data.lock().unwrap();
        data.entry(plugin_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), json_value);
        Ok(())
    }
    
    /// 获取数据
    pub fn get<T: for<'de> Deserialize<'de>>(&self, plugin_id: &str, key: &str) -> PluginResult<Option<T>> {
        let data = self.data.lock().unwrap();
        if let Some(plugin_data) = data.get(plugin_id) {
            if let Some(value) = plugin_data.get(key) {
                let typed_value = serde_json::from_value(value.clone())?;
                return Ok(Some(typed_value));
            }
        }
        Ok(None)
    }
    
    /// 删除数据
    pub fn delete(&self, plugin_id: &str, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(plugin_data) = data.get_mut(plugin_id) {
            plugin_data.remove(key).is_some()
        } else {
            false
        }
    }
    
    /// 列出所有键
    pub fn list_keys(&self, plugin_id: &str) -> Vec<String> {
        let data = self.data.lock().unwrap();
        if let Some(plugin_data) = data.get(plugin_id) {
            plugin_data.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// 清空所有数据
    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }
    
    /// 获取插件数据大小
    pub fn plugin_data_size(&self, plugin_id: &str) -> usize {
        let data = self.data.lock().unwrap();
        data.get(plugin_id).map(|d| d.len()).unwrap_or(0)
    }
}

/// 测试断言辅助
pub struct TestAssertions;

impl TestAssertions {
    /// 断言插件状态
    pub fn assert_plugin_status(plugin: &dyn Plugin, expected: PluginStatus) {
        assert_eq!(plugin.status(), expected, "Plugin status mismatch");
    }
    
    /// 断言插件配置存在
    pub fn assert_plugin_has_config(plugin: &dyn Plugin) {
        assert!(plugin.get_config().is_some(), "Plugin should have config");
    }
    
    /// 断言消息内容
    pub fn assert_message_content(message: &PluginMessage, expected: &str) {
        let content = message.payload_string().unwrap();
        assert_eq!(content, expected, "Message content mismatch");
    }
    
    /// 断言消息主题
    pub fn assert_message_topic(message: &PluginMessage, expected: &str) {
        assert_eq!(message.topic, expected, "Message topic mismatch");
    }
    
    /// 断言消息优先级
    pub fn assert_message_priority(message: &PluginMessage, expected: MessagePriority) {
        assert_eq!(message.priority, expected, "Message priority mismatch");
    }
    
    /// 断言消息未过期
    pub fn assert_message_not_expired(message: &PluginMessage) {
        assert!(!message.is_expired(), "Message should not be expired");
    }
    
    /// 断言消息已过期
    pub fn assert_message_expired(message: &PluginMessage) {
        assert!(message.is_expired(), "Message should be expired");
    }
    
    /// 断言配置值
    pub fn assert_config_value<T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        config: &PluginConfig,
        key: &str,
        expected: T,
    ) {
        let extractor = crate::utils::config::ConfigExtractor::new(config.data.clone());
        let value: T = extractor.get_number(key).unwrap_or_else(|_| {
            panic!("Config key '{}' not found or invalid", key);
        });
        assert_eq!(value, expected, "Config value mismatch for key '{}'", key);
    }
    
    /// 断言存储包含键
    pub fn assert_storage_contains_key(storage: &MockStorage, plugin_id: &str, key: &str) {
        let keys = storage.list_keys(plugin_id);
        assert!(keys.contains(&key.to_string()), "Storage should contain key '{}'", key);
    }
    
    /// 断言存储不包含键
    pub fn assert_storage_not_contains_key(storage: &MockStorage, plugin_id: &str, key: &str) {
        let keys = storage.list_keys(plugin_id);
        assert!(!keys.contains(&key.to_string()), "Storage should not contain key '{}'", key);
    }
    
    /// 断言存储值
    pub fn assert_storage_value<T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        storage: &MockStorage,
        plugin_id: &str,
        key: &str,
        expected: T,
    ) {
        let value: T = storage.get(plugin_id, key).unwrap()
            .unwrap_or_else(|| panic!("Storage key '{}' not found", key));
        assert_eq!(value, expected, "Storage value mismatch for key '{}'", key);
    }
}

/// 测试计时器
pub struct TestTimer {
    start: std::time::Instant,
    name: String,
}

impl TestTimer {
    /// 创建新的测试计时器
    pub fn new(name: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            name: name.to_string(),
        }
    }
    
    /// 获取经过的时间
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
    
    /// 断言执行时间在指定范围内
    pub fn assert_elapsed_within(&self, min: std::time::Duration, max: std::time::Duration) {
        let elapsed = self.elapsed();
        assert!(
            elapsed >= min && elapsed <= max,
            "Timer '{}' elapsed {:?}, expected between {:?} and {:?}",
            self.name, elapsed, min, max
        );
    }
    
    /// 断言执行时间小于指定值
    pub fn assert_elapsed_less_than(&self, max: std::time::Duration) {
        let elapsed = self.elapsed();
        assert!(
            elapsed < max,
            "Timer '{}' elapsed {:?}, expected less than {:?}",
            self.name, elapsed, max
        );
    }
}

/// 测试宏
#[macro_export]
macro_rules! assert_plugin_error {
    ($result:expr, $error_type:path) => {
        match $result {
            Err(e) => {
                assert!(matches!(e, $error_type(_)), "Expected {} error, got: {:?}", stringify!($error_type), e);
            }
            Ok(_) => panic!("Expected error, got Ok"),
        }
    };
}

#[macro_export]
macro_rules! assert_plugin_ok {
    ($result:expr) => {
        match $result {
            Ok(_) => {}
            Err(e) => panic!("Expected Ok, got error: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! test_plugin_lifecycle {
    ($plugin:expr, $config:expr) => {
        // 测试初始化
        assert_eq!($plugin.status(), $crate::plugin::PluginStatus::Uninitialized);
        assert_plugin_ok!($plugin.initialize($config.clone()));
        assert_eq!($plugin.status(), $crate::plugin::PluginStatus::Running);
        assert!($plugin.get_config().is_some());
        
        // 测试关闭
        assert_plugin_ok!($plugin.shutdown());
        assert_eq!($plugin.status(), $crate::plugin::PluginStatus::Shutdown);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_plugin() {
        let mut plugin = MockPlugin::new("test");
        let config = TestConfigBuilder::new()
            .add_string("key", "value")
            .build();
        
        assert_eq!(plugin.status(), PluginStatus::Uninitialized);
        plugin.initialize(config).unwrap();
        assert_eq!(plugin.status(), PluginStatus::Running);
        
        let message = TestMessageBuilder::simple("sender", "test", "hello");
        plugin.handle_message(message.clone()).unwrap();
        
        assert_eq!(plugin.received_messages().len(), 1);
        assert_eq!(plugin.received_messages()[0].payload_string().unwrap(), "hello");
    }
    
    #[test]
    fn test_mock_storage() {
        let storage = MockStorage::new();
        
        storage.store("plugin1", "key1", &"value1").unwrap();
        storage.store("plugin1", "key2", &42).unwrap();
        storage.store("plugin2", "key1", &true).unwrap();
        
        let value: String = storage.get("plugin1", "key1").unwrap().unwrap();
        assert_eq!(value, "value1");
        
        let value: i32 = storage.get("plugin1", "key2").unwrap().unwrap();
        assert_eq!(value, 42);
        
        let keys = storage.list_keys("plugin1");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
    
    #[test]
    fn test_test_config_builder() {
        let config = TestConfigBuilder::new()
            .add_string("name", "test")
            .add_number("port", 8080)
            .add_bool("enabled", true)
            .add_array("tags", &["tag1", "tag2"])
            .build();
        
        assert!(config.enabled);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.data.len(), 4);
    }
    
    #[test]
    fn test_test_message_builder() {
        let message = TestMessageBuilder::simple("from", "to", "content");
        assert_eq!(message.from, "from");
        assert_eq!(message.to, "to");
        assert_eq!(message.payload_string().unwrap(), "content");
        
        let message = TestMessageBuilder::high_priority("from", "to", "urgent");
        assert_eq!(message.priority, MessagePriority::High);
        
        let message = TestMessageBuilder::expired("from", "to", "expired");
        assert!(message.is_expired());
    }
}