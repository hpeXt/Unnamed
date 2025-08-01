//! 标准插件模板
//! 
//! 这是一个展示如何正确使用主机函数的插件模板。
//! 请仔细阅读注释，了解每个部分的作用。

use plugin_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 重要：主机函数声明
// ============================================================================
// 如果你需要直接调用主机函数（不使用 SDK 包装器），必须使用正确的函数名。
// 在我们的系统中，所有主机函数都使用 "_host" 后缀。
#[host_fn]
extern "ExtismHost" {
    // 日志函数
    fn log_message_host(level: &str, message: &str) -> String;
    
    // 存储函数
    fn store_data_host(plugin_id: &str, key: &str, value: &str) -> String;
    fn get_data_host(plugin_id: &str, key: &str) -> String;
    fn delete_data_host(plugin_id: &str, key: &str) -> String;
    fn list_keys_host(plugin_id: &str) -> String;
    
    // 消息函数
    fn send_message_host(from: &str, to: &str, payload: &str) -> String;
    fn subscribe_topic_host(plugin_id: &str, topic: &str) -> String;
    fn unsubscribe_topic_host(plugin_id: &str, topic: &str) -> String;
    fn publish_message_host(plugin_id: &str, topic: &str, payload: &str) -> String;
}

// ============================================================================
// 插件配置结构
// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateConfig {
    /// 处理间隔（毫秒）
    interval_ms: u64,
    /// 是否启用调试日志
    debug_enabled: bool,
    /// 自定义设置
    custom_settings: HashMap<String, String>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000,
            debug_enabled: false,
            custom_settings: HashMap::new(),
        }
    }
}

// ============================================================================
// 插件主结构
// ============================================================================
/// 模板插件 - 展示最佳实践
pub struct TemplatePlugin {
    /// 插件配置
    config: TemplateConfig,
    /// 插件状态
    status: PluginStatus,
    /// 处理计数器
    process_count: u64,
    /// 插件 ID（用于主机函数调用）
    plugin_id: String,
}

impl TemplatePlugin {
    pub fn new() -> Self {
        Self {
            config: TemplateConfig::default(),
            status: PluginStatus::Uninitialized,
            process_count: 0,
            plugin_id: "template-plugin".to_string(),
        }
    }
    
    /// 演示如何使用 SDK 提供的便利函数（推荐方式）
    fn use_sdk_functions(&self) -> PluginResult<()> {
        // 使用 SDK 的日志宏
        log_info!("使用 SDK 日志宏记录信息");
        
        // 使用 SDK 的存储函数
        let data = serde_json::json!({
            "count": self.process_count,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // SDK 内部会调用正确的主机函数
        host::storage::store(&self.plugin_id, "last_process", &data)?;
        
        Ok(())
    }
    
    /// 演示如何直接调用主机函数（当需要更多控制时）
    fn use_direct_host_functions(&self) -> PluginResult<()> {
        // 直接调用主机函数 - 注意函数名带 _host 后缀！
        unsafe {
            // 记录日志
            let log_result = log_message_host("info", "直接调用主机函数")?;
            self.handle_host_response(&log_result)?;
            
            // 存储数据
            let data = serde_json::json!({
                "direct_call": true,
                "count": self.process_count,
            });
            let store_result = store_data_host(
                &self.plugin_id,
                "direct_data",
                &data.to_string()
            )?;
            self.handle_host_response(&store_result)?;
        }
        
        Ok(())
    }
    
    /// 处理主机函数返回的响应
    fn handle_host_response(&self, response: &str) -> PluginResult<()> {
        #[derive(Deserialize)]
        struct HostResponse {
            success: bool,
            error: Option<String>,
        }
        
        let resp: HostResponse = serde_json::from_str(response)
            .map_err(|e| PluginError::HostFunction(format!("解析响应失败: {}", e)))?;
        
        if !resp.success {
            return Err(PluginError::HostFunction(
                resp.error.unwrap_or_else(|| "未知错误".to_string())
            ));
        }
        
        Ok(())
    }
}

impl Default for TemplatePlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 插件元数据
// ============================================================================
plugin_info!(
    name: "template-plugin",
    version: "0.1.0",
    description: "标准插件模板 - 展示正确的主机函数使用方式",
    author: "Your Name",
    tags: ["template", "example", "best-practices"]
);

// ============================================================================
// Plugin Trait 实现
// ============================================================================
impl Plugin for TemplatePlugin {
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        log_info!("模板插件正在初始化...");
        
        // 解析配置
        if let Some(interval) = config.get_number("interval_ms") {
            self.config.interval_ms = interval as u64;
        }
        
        if let Some(debug) = config.get_bool("debug_enabled") {
            self.config.debug_enabled = debug;
        }
        
        // 订阅主题（使用 SDK 宏）
        subscribe_topics!(&self.plugin_id, "template.command", "template.data")?;
        
        // 或者直接调用主机函数
        // unsafe {
        //     let result = subscribe_topic_host(&self.plugin_id, "template.command")?;
        //     self.handle_host_response(&result)?;
        // }
        
        self.status = PluginStatus::Running;
        log_info!("模板插件初始化完成！");
        
        Ok(())
    }
    
    fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()> {
        if self.config.debug_enabled {
            log_debug!("收到消息: {:?}", message);
        }
        
        match message.topic.as_str() {
            "template.command" => {
                let command = message.payload_string()?;
                self.handle_command(&command)?;
            }
            "template.data" => {
                self.process_count += 1;
                
                // 演示两种调用方式
                if self.process_count % 2 == 0 {
                    self.use_sdk_functions()?;
                } else {
                    self.use_direct_host_functions()?;
                }
            }
            _ => {
                log_warn!("收到未知主题的消息: {}", message.topic);
            }
        }
        
        Ok(())
    }
    
    fn tick(&mut self) -> PluginResult<()> {
        // 定期执行的任务
        if self.process_count > 0 && self.process_count % 10 == 0 {
            log_info!("已处理 {} 条消息", self.process_count);
            
            // 发布状态更新
            let status = serde_json::json!({
                "plugin": self.plugin_id,
                "count": self.process_count,
                "status": "healthy"
            });
            
            // 使用 SDK 函数
            host::messaging::publish(&self.plugin_id, "template.status", &status)?;
        }
        
        Ok(())
    }
    
    fn health_check(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        let mut health = HashMap::new();
        
        health.insert("status".to_string(), json!("healthy"));
        health.insert("process_count".to_string(), json!(self.process_count));
        health.insert("config".to_string(), json!(self.config));
        
        Ok(health)
    }
    
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        stats.insert("process_count".to_string(), json!(self.process_count));
        stats.insert("interval_ms".to_string(), json!(self.config.interval_ms));
        stats.insert("debug_enabled".to_string(), json!(self.config.debug_enabled));
        
        Ok(stats)
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        log_info!("模板插件正在关闭...");
        
        // 保存最终状态
        let final_state = serde_json::json!({
            "final_count": self.process_count,
            "shutdown_time": chrono::Utc::now().to_rfc3339(),
        });
        
        host::storage::store(&self.plugin_id, "final_state", &final_state)?;
        
        self.status = PluginStatus::Shutdown;
        log_info!("模板插件已关闭");
        
        Ok(())
    }
}

// ============================================================================
// 辅助方法
// ============================================================================
impl TemplatePlugin {
    fn handle_command(&mut self, command: &str) -> PluginResult<()> {
        match command {
            "reset" => {
                self.process_count = 0;
                log_info!("计数器已重置");
            }
            "status" => {
                log_info!("当前状态: 处理数={}, 状态={:?}", 
                    self.process_count, self.status);
            }
            "test_host_functions" => {
                // 测试所有主机函数
                self.test_all_host_functions()?;
            }
            _ => {
                log_warn!("未知命令: {}", command);
            }
        }
        
        Ok(())
    }
    
    /// 测试所有主机函数是否正常工作
    fn test_all_host_functions(&self) -> PluginResult<()> {
        log_info!("开始测试主机函数...");
        
        // 测试日志
        unsafe {
            log_message_host("info", "测试日志函数")?;
        }
        
        // 测试存储
        let test_data = json!({"test": true});
        host::storage::store(&self.plugin_id, "test_key", &test_data)?;
        let retrieved = host::storage::get::<serde_json::Value>(&self.plugin_id, "test_key")?;
        assert!(retrieved.is_some(), "存储测试失败");
        
        // 测试消息发送
        let msg = PluginMessage::builder(&self.plugin_id)
            .to("test-receiver")
            .payload_string("测试消息")
            .build()
            .map_err(|e| PluginError::MessageProcessing(e))?;
        host::messaging::send(&msg)?;
        
        log_info!("主机函数测试完成！");
        Ok(())
    }
}

// ============================================================================
// 插件入口点
// ============================================================================
// 生成标准的插件入口函数
plugin_main!(TemplatePlugin);

// ============================================================================
// 测试模块
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::testing::*;
    
    #[test]
    fn test_plugin_lifecycle() {
        let mut plugin = TemplatePlugin::new();
        let config = TestConfigBuilder::new()
            .add_number("interval_ms", 1000u64)
            .add_bool("debug_enabled", true)
            .build();
        
        // 测试初始化
        assert_plugin_ok!(plugin.initialize(config));
        assert_eq!(plugin.status, PluginStatus::Running);
        assert_eq!(plugin.config.interval_ms, 1000);
        
        // 测试消息处理
        let msg = TestMessageBuilder::simple("test", "template-plugin", "status")
            .with_topic("template.command");
        assert_plugin_ok!(plugin.handle_message(msg));
        
        // 测试关闭
        assert_plugin_ok!(plugin.shutdown());
        assert_eq!(plugin.status, PluginStatus::Shutdown);
    }
}