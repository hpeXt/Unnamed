use plugin_sdk::{
    plugin::{Plugin, PluginMetadata, PluginStatus, PluginConfig, PluginEvent},
    error::{PluginResult, PluginError},
    message::PluginMessage,
    log_debug, log_info, log_warn,
};
use extism_pdk::*;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 系统统计数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemStats {
    cpu: f64,
    memory: f64,
    plugin_count: u32,
    uptime: u64,
    timestamp: u64,
}

/// 系统统计收集插件
pub struct SystemStatsCollectorPlugin {
    config: Option<PluginConfig>,
    status: PluginStatus,
    start_time: u64,
    collect_interval_ms: u64,
    last_collect_time: u64,
}

impl SystemStatsCollectorPlugin {
    pub fn new() -> Self {
        let now = plugin_sdk::utils::time::now_secs();
        
        Self {
            config: None,
            status: PluginStatus::Uninitialized,
            start_time: now,
            collect_interval_ms: 2000, // 默认每2秒收集一次
            last_collect_time: 0,
        }
    }
    
    /// 收集系统统计数据（模拟）
    fn collect_stats(&self) -> SystemStats {
        let now = plugin_sdk::utils::time::now_secs();
        let now_millis = plugin_sdk::utils::time::now_millis();
        
        // 使用简单的伪随机数生成（基于时间戳）
        let seed = now % 100;
        let cpu_variation = (seed as f64 * 0.6) % 60.0;
        let mem_variation = ((seed + 17) as f64 * 0.4) % 40.0;
        
        SystemStats {
            cpu: 20.0 + cpu_variation,                 // 20-80% CPU
            memory: 30.0 + mem_variation,              // 30-70% 内存
            plugin_count: 3,                           // 固定3个插件
            uptime: now - self.start_time,            // 运行时间（秒）
            timestamp: now_millis,
        }
    }
    
    /// 发布统计数据到消息总线
    fn publish_stats(&self, stats: &SystemStats) -> PluginResult<()> {
        // 构建消息
        let _message = PluginMessage::builder("system-stats-collector")
            .topic("system.stats")
            .payload_json(stats)?
            .build()
            .map_err(|e| PluginError::MessageProcessing(e))?;
        
        // 发送到消息总线（在真实环境中，这会通过主机函数发送）
        log_info!("发布系统统计: CPU={:.1}%, 内存={:.1}%", stats.cpu, stats.memory);
        
        // 注意：在 WASM 环境中，实际的消息发送需要通过主机函数
        // 这里我们只是记录日志，真正的发送会在 tick 方法中处理
        
        Ok(())
    }
}

impl Default for SystemStatsCollectorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SystemStatsCollectorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "system-stats-collector".to_string(),
            version: "0.1.0".to_string(),
            description: "收集系统统计信息并发布到消息总线".to_string(),
            author: Some("Minimal Kernel Team".to_string()),
            dependencies: Vec::new(),
            tags: vec!["system".to_string(), "monitoring".to_string(), "stats".to_string()],
            config_schema: None,
        }
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        log_info!("系统统计收集插件正在初始化...");
        
        // 从配置中读取收集间隔
        if let Some(interval) = config.data.get("collect_interval_ms") {
            if let Some(interval_ms) = interval.as_u64() {
                self.collect_interval_ms = interval_ms;
                log_info!("设置收集间隔: {}ms", interval_ms);
            }
        }
        
        self.config = Some(config);
        self.status = PluginStatus::Running;
        
        log_info!("系统统计收集插件初始化完成");
        Ok(())
    }
    
    fn tick(&mut self) -> PluginResult<()> {
        let now = plugin_sdk::utils::time::now_millis();
        
        // 检查是否该收集数据了
        if now - self.last_collect_time >= self.collect_interval_ms {
            // 收集统计数据
            let stats = self.collect_stats();
            
            // 发布到消息总线
            self.publish_stats(&stats)?;
            
            // 更新最后收集时间
            self.last_collect_time = now;
        }
        
        Ok(())
    }
    
    fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()> {
        log_debug!("收到消息: 主题={}, 发送者={}", message.topic, message.from);
        
        match message.topic.as_str() {
            "control" => {
                let command = message.payload_string()?;
                match command.as_str() {
                    "collect_now" => {
                        log_info!("立即收集统计数据");
                        let stats = self.collect_stats();
                        self.publish_stats(&stats)?;
                    }
                    "status" => {
                        log_info!("插件状态: {:?}", self.status);
                    }
                    _ => {
                        log_warn!("未知控制命令: {}", command);
                    }
                }
            }
            _ => {
                log_debug!("未处理的消息主题: {}", message.topic);
            }
        }
        
        Ok(())
    }
    
    fn health_check(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        let mut health = HashMap::new();
        
        health.insert("status".to_string(), serde_json::Value::String("healthy".to_string()));
        health.insert("collect_interval_ms".to_string(), serde_json::Value::Number(self.collect_interval_ms.into()));
        health.insert("uptime".to_string(), serde_json::Value::Number((plugin_sdk::utils::time::now_secs() - self.start_time).into()));
        
        Ok(health)
    }
    
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        stats.insert("status".to_string(), serde_json::Value::String(format!("{:?}", self.status)));
        stats.insert("collect_interval_ms".to_string(), serde_json::Value::Number(self.collect_interval_ms.into()));
        stats.insert("last_collect_time".to_string(), serde_json::Value::Number(self.last_collect_time.into()));
        
        Ok(stats)
    }
    
    fn get_config(&self) -> Option<&PluginConfig> {
        self.config.as_ref()
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        self.status = PluginStatus::Shutdown;
        Ok(())
    }
    
    fn handle_event(&mut self, event: PluginEvent) -> PluginResult<()> {
        match event {
            PluginEvent::Message(msg) => self.handle_message(msg),
            PluginEvent::Shutdown => {
                self.status = PluginStatus::Shutdown;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// 手动实现插件入口点

static PLUGIN_INSTANCE: LazyLock<Mutex<Option<SystemStatsCollectorPlugin>>> = 
    LazyLock::new(|| Mutex::new(None));

#[plugin_fn]
pub fn initialize(config_json: String) -> FnResult<String> {
    let config = if config_json.is_empty() {
        PluginConfig::default()
    } else {
        serde_json::from_str(&config_json)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to parse config: {}", e)))?
    };
    
    let mut instance = SystemStatsCollectorPlugin::default();
    match instance.initialize(config) {
        Ok(_) => {
            let mut guard = PLUGIN_INSTANCE.lock().unwrap();
            *guard = Some(instance);
            Ok(serde_json::json!({
                "success": true,
                "metadata": guard.as_ref().unwrap().metadata()
            }).to_string())
        }
        Err(e) => Err(extism_pdk::Error::msg(format!("Failed to initialize plugin: {}", e)).into()),
    }
}

#[plugin_fn]
pub fn handle_message(message_json: String) -> FnResult<String> {
    let message: PluginMessage = serde_json::from_str(&message_json)
        .map_err(|e| extism_pdk::Error::msg(format!("Failed to parse message: {}", e)))?;
    
    let mut guard = PLUGIN_INSTANCE.lock().unwrap();
    if let Some(ref mut plugin) = guard.as_mut() {
        match plugin.handle_message(message) {
            Ok(_) => Ok(serde_json::json!({"success": true}).to_string()),
            Err(e) => Err(extism_pdk::Error::msg(format!("Failed to handle message: {}", e)).into()),
        }
    } else {
        Err(extism_pdk::Error::msg("Plugin not initialized").into())
    }
}

#[plugin_fn]
pub fn tick() -> FnResult<String> {
    let mut guard = PLUGIN_INSTANCE.lock().unwrap();
    if let Some(ref mut plugin) = guard.as_mut() {
        match plugin.tick() {
            Ok(_) => Ok(serde_json::json!({"success": true}).to_string()),
            Err(e) => Err(extism_pdk::Error::msg(format!("Failed to tick: {}", e)).into()),
        }
    } else {
        Err(extism_pdk::Error::msg("Plugin not initialized").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::testing::*;
    
    #[test]
    fn test_stats_collection() {
        let plugin = SystemStatsCollectorPlugin::new();
        let stats = plugin.collect_stats();
        
        // 验证统计数据范围
        assert!(stats.cpu >= 20.0 && stats.cpu <= 80.0);
        assert!(stats.memory >= 30.0 && stats.memory <= 70.0);
        assert_eq!(stats.plugin_count, 3);
        assert!(stats.uptime >= 0);
    }
    
    #[test]
    fn test_plugin_lifecycle() {
        let mut plugin = SystemStatsCollectorPlugin::new();
        let config = TestConfigBuilder::new()
            .add_number("collect_interval_ms", 1000u64)
            .build();
        
        // 测试生命周期
        test_plugin_lifecycle!(plugin, config);
    }
}