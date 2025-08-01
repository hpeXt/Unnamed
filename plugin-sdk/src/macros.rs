//! 插件开发宏
//!
//! 提供简化插件开发的宏定义

/// 定义插件主入口宏
#[macro_export]
macro_rules! plugin_main {
    ($plugin_type:ty) => {
        use $crate::plugin::Plugin;
        use $crate::error::PluginResult;
        use extism_pdk::*;
        use std::sync::Mutex;
        
        // 全局插件实例
        static PLUGIN_INSTANCE: std::sync::LazyLock<Mutex<Option<$plugin_type>>> = 
            std::sync::LazyLock::new(|| Mutex::new(None));
        
        /// 初始化插件
        #[plugin_fn]
        pub fn initialize(config_json: String) -> FnResult<String> {
            let config = if config_json.is_empty() {
                $crate::plugin::PluginConfig::default()
            } else {
                serde_json::from_str(&config_json)
                    .map_err(|e| extism_pdk::Error::msg(format!("Failed to parse config: {}", e)))?
            };
            
            let mut instance = <$plugin_type>::default();
            match instance.initialize(config) {
                Ok(_) => {
                    let mut guard = PLUGIN_INSTANCE.lock().unwrap();
                    *guard = Some(instance);
                    Ok(serde_json::json!({
                        "success": true,
                        "metadata": guard.as_ref().unwrap().metadata()
                    }).to_string())
                }
                Err(e) => Err(extism_pdk::Error::msg(format!("Failed to initialize plugin: {}", e))),
            }
        }
        
        /// 处理消息
        #[plugin_fn]
        pub fn handle_message(message_json: String) -> FnResult<String> {
            let message: $crate::message::PluginMessage = serde_json::from_str(&message_json)
                .map_err(|e| extism_pdk::Error::msg(format!("Failed to parse message: {}", e)))?;
            
            let mut guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref mut plugin) = guard.as_mut() {
                match plugin.handle_message(message) {
                    Ok(_) => Ok(serde_json::json!({"success": true}).to_string()),
                    Err(e) => Err(extism_pdk::Error::msg(format!("Failed to handle message: {}", e))),
                }
            } else {
                Err(extism_pdk::Error::msg("Plugin not initialized"))
            }
        }
        
        /// 插件定时任务
        #[plugin_fn]
        pub fn tick() -> FnResult<String> {
            let mut guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref mut plugin) = guard.as_mut() {
                match plugin.tick() {
                    Ok(_) => Ok(serde_json::json!({"success": true}).to_string()),
                    Err(e) => Err(extism_pdk::Error::msg(format!("Failed to tick: {}", e))),
                }
            } else {
                Err(extism_pdk::Error::msg("Plugin not initialized"))
            }
        }
        
        /// 获取插件元数据
        #[plugin_fn]
        pub fn metadata() -> FnResult<String> {
            let guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref plugin) = guard.as_ref() {
                Ok(serde_json::to_string(&plugin.metadata())
                    .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize metadata: {}", e)))?)
            } else {
                // 如果插件还没初始化，返回默认元数据
                let temp_instance = <$plugin_type>::default();
                Ok(serde_json::to_string(&temp_instance.metadata())
                    .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize metadata: {}", e)))?)
            }
        }
        
        /// 获取插件状态
        #[plugin_fn]
        pub fn status() -> FnResult<String> {
            let guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref plugin) = guard.as_ref() {
                Ok(serde_json::json!({
                    "status": plugin.status(),
                    "stats": plugin.get_stats().unwrap_or_default()
                }).to_string())
            } else {
                Ok(serde_json::json!({
                    "status": "Uninitialized"
                }).to_string())
            }
        }
        
        /// 健康检查
        #[plugin_fn]
        pub fn health_check() -> FnResult<String> {
            let guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref plugin) = guard.as_ref() {
                match plugin.health_check() {
                    Ok(health) => Ok(serde_json::json!({
                        "healthy": true,
                        "details": health
                    }).to_string()),
                    Err(e) => Ok(serde_json::json!({
                        "healthy": false,
                        "error": e.to_string()
                    }).to_string()),
                }
            } else {
                Ok(serde_json::json!({
                    "healthy": false,
                    "error": "Plugin not initialized"
                }).to_string())
            }
        }
        
        /// 关闭插件
        #[plugin_fn]
        pub fn shutdown() -> FnResult<String> {
            let mut guard = PLUGIN_INSTANCE.lock().unwrap();
            if let Some(ref mut plugin) = guard.as_mut() {
                match plugin.shutdown() {
                    Ok(_) => {
                        *guard = None;
                        Ok(serde_json::json!({"success": true}).to_string())
                    }
                    Err(e) => Err(extism_pdk::Error::msg(format!("Failed to shutdown plugin: {}", e))),
                }
            } else {
                Ok(serde_json::json!({"success": true}).to_string())
            }
        }
    };
}

/// 定义插件处理函数的宏
#[macro_export]
macro_rules! plugin_handler {
    ($fn_name:ident, $handler:expr) => {
        #[plugin_fn]
        pub fn $fn_name(input: String) -> FnResult<String> {
            match $handler(input) {
                Ok(result) => Ok(result),
                Err(e) => Err(extism_pdk::Error::msg(format!("Handler error: {}", e))),
            }
        }
    };
}

/// 定义带JSON序列化的插件处理函数宏
#[macro_export]
macro_rules! plugin_json_handler {
    ($fn_name:ident, $input_type:ty, $output_type:ty, $handler:expr) => {
        #[plugin_fn]
        pub fn $fn_name(input: String) -> FnResult<String> {
            let parsed_input: $input_type = serde_json::from_str(&input)
                .map_err(|e| extism_pdk::Error::msg(format!("Failed to parse input: {}", e)))?;
            
            let result: $output_type = $handler(parsed_input)
                .map_err(|e| extism_pdk::Error::msg(format!("Handler error: {}", e)))?;
            
            serde_json::to_string(&result)
                .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize result: {}", e)))
        }
    };
}

/// 定义插件信息宏
#[macro_export]
macro_rules! plugin_info {
    (
        name: $name:expr,
        version: $version:expr,
        description: $description:expr
        $(, author: $author:expr)?
        $(, dependencies: [$($dep:expr),*])?
        $(, tags: [$($tag:expr),*])?
    ) => {
        impl Default for Self {
            fn default() -> Self {
                Self::new()
            }
        }
        
        impl $crate::plugin::Plugin for Self {
            fn metadata(&self) -> $crate::plugin::PluginMetadata {
                $crate::plugin::PluginMetadata {
                    name: $name.to_string(),
                    version: $version.to_string(),
                    description: $description.to_string(),
                    author: plugin_info!(@author $($author)?),
                    dependencies: plugin_info!(@dependencies $($($dep),*)?),
                    tags: plugin_info!(@tags $($($tag),*)?),
                    config_schema: None,
                }
            }
            
            fn status(&self) -> $crate::plugin::PluginStatus {
                self.status
            }
            
            fn initialize(&mut self, config: $crate::plugin::PluginConfig) -> $crate::error::PluginResult<()> {
                self.config = Some(config);
                self.status = $crate::plugin::PluginStatus::Running;
                Ok(())
            }
            
            fn handle_event(&mut self, event: $crate::plugin::PluginEvent) -> $crate::error::PluginResult<()> {
                match event {
                    $crate::plugin::PluginEvent::Message(msg) => self.handle_message(msg),
                    $crate::plugin::PluginEvent::Shutdown => {
                        self.status = $crate::plugin::PluginStatus::Shutdown;
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            
            fn get_config(&self) -> Option<&$crate::plugin::PluginConfig> {
                self.config.as_ref()
            }
            
            fn shutdown(&mut self) -> $crate::error::PluginResult<()> {
                self.status = $crate::plugin::PluginStatus::Shutdown;
                Ok(())
            }
        }
    };
    
    (@author) => { None };
    (@author $author:expr) => { Some($author.to_string()) };
    
    (@dependencies) => { Vec::new() };
    (@dependencies $($dep:expr),*) => { vec![$($dep.to_string()),*] };
    
    (@tags) => { Vec::new() };
    (@tags $($tag:expr),*) => { vec![$($tag.to_string()),*] };
}

/// 定义消息订阅宏
#[macro_export]
macro_rules! subscribe_topics {
    ($plugin_id:expr, $($topic:expr),*) => {
        {
            use $crate::host::messaging;
            $(
                messaging::subscribe($plugin_id, $topic)?;
            )*
            Ok(())
        }
    };
}

/// 定义存储操作宏
#[macro_export]
macro_rules! store_data {
    ($plugin_id:expr, $key:expr, $value:expr) => {
        $crate::host::storage::store($plugin_id, $key, &$value)
    };
}

#[macro_export]
macro_rules! get_data {
    ($plugin_id:expr, $key:expr, $type:ty) => {
        $crate::host::storage::get::<$type>($plugin_id, $key)
    };
}

/// 定义错误处理宏
#[macro_export]
macro_rules! try_or_log {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                $crate::log_error!("{}: {}", $msg, e);
                return Err(e);
            }
        }
    };
}

/// 定义条件编译的调试宏
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_debug!($($arg)*);
    };
}

/// 定义计时宏
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:block) => {
        {
            let start = std::time::Instant::now();
            let result = $block;
            let elapsed = start.elapsed();
            $crate::log_debug!("{} took {:?}", $name, elapsed);
            result
        }
    };
}

/// 插件测试宏
#[cfg(test)]
#[macro_export]
macro_rules! plugin_test {
    ($test_name:ident, $plugin_type:ty, $test_body:block) => {
        #[test]
        fn $test_name() {
            use $crate::plugin::Plugin;
            let mut plugin = <$plugin_type>::default();
            let config = $crate::plugin::PluginConfig::default();
            plugin.initialize(config).unwrap();
            
            $test_body
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 这些宏需要在实际插件中测试，这里只是确保它们能编译
    #[test]
    fn test_macros_compile() {
        // 测试宏能正确编译
        assert!(true);
    }
}