use plugin_sdk::prelude::*;
use std::collections::HashMap;

pub struct TestPlugin {
    config: Option<PluginConfig>,
    status: PluginStatus,
}

impl TestPlugin {
    pub fn new() -> Self {
        Self {
            config: None,
            status: PluginStatus::Uninitialized,
        }
    }
}

impl Default for TestPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TestPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            description: "Test plugin".to_string(),
            author: Some("Test".to_string()),
            dependencies: Vec::new(),
            tags: vec!["test".to_string()],
            config_schema: None,
        }
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        self.config = Some(config);
        self.status = PluginStatus::Running;
        Ok(())
    }
    
    fn tick(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn handle_message(&mut self, _message: PluginMessage) -> PluginResult<()> {
        Ok(())
    }
    
    fn health_check(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
    
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
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

// 使用宏生成插件入口点
plugin_main!(TestPlugin);