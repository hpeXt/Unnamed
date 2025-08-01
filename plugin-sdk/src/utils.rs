//! 插件开发工具函数
//!
//! 提供插件开发中常用的工具函数和辅助功能

use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 时间相关工具
pub mod time {
    use super::*;
    use extism_pdk::*;
    
    // 声明主机函数
    #[host_fn]
    extern "ExtismHost" {
        fn get_timestamp_host() -> String;
        fn get_timestamp_millis_host() -> String;
    }
    
    /// 获取当前时间戳（毫秒）
    pub fn now_millis() -> u64 {
        unsafe {
            match get_timestamp_millis_host() {
                Ok(timestamp_str) => timestamp_str.parse::<u64>().unwrap_or(0),
                Err(_) => 0,
            }
        }
    }
    
    /// 获取当前时间戳（秒）
    pub fn now_secs() -> u64 {
        unsafe {
            match get_timestamp_host() {
                Ok(timestamp_str) => timestamp_str.parse::<u64>().unwrap_or(0),
                Err(_) => 0,
            }
        }
    }
    
    /// 格式化时间戳（简单实现）
    pub fn format_timestamp(timestamp: u64) -> String {
        // 在插件环境中，我们只返回简单的格式
        format!("T{}", timestamp)
    }
    
    /// 计算时间差（毫秒）
    pub fn time_diff_millis(start: u64, end: u64) -> u64 {
        end.saturating_sub(start)
    }
    
    /// 简单的性能计时器
    pub struct Timer {
        start: Instant,
        name: String,
    }
    
    impl Timer {
        pub fn new(name: &str) -> Self {
            Self {
                start: Instant::now(),
                name: name.to_string(),
            }
        }
        
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
        
        pub fn elapsed_millis(&self) -> u64 {
            self.start.elapsed().as_millis() as u64
        }
    }
    
    impl Drop for Timer {
        fn drop(&mut self) {
            crate::log_debug!("Timer '{}' elapsed: {:?}", self.name, self.elapsed());
        }
    }
}

/// 字符串处理工具
pub mod string {
    use super::*;
    
    /// 截断字符串到指定长度
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
    
    /// 清理字符串（移除控制字符）
    pub fn sanitize(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect()
    }
    
    /// 检查字符串是否为空或只包含空白字符
    pub fn is_empty_or_whitespace(s: &str) -> bool {
        s.trim().is_empty()
    }
    
    /// 生成随机字符串
    pub fn random_string(len: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();
        
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
        let mut result = String::new();
        let mut seed = hash;
        
        for _ in 0..len {
            let idx = (seed % chars.len() as u64) as usize;
            result.push(chars[idx]);
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        }
        
        result
    }
}

/// 数据转换工具
pub mod convert {
    use super::*;
    
    /// 安全地将 JSON 值转换为字符串
    pub fn json_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }
    
    /// 安全地将字符串转换为 JSON 值
    pub fn string_to_json(s: &str) -> serde_json::Value {
        serde_json::from_str(s).unwrap_or_else(|_| serde_json::Value::String(s.to_string()))
    }
    
    /// 将字节数组转换为十六进制字符串
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
    
    /// 将十六进制字符串转换为字节数组
    pub fn hex_to_bytes(hex: &str) -> PluginResult<Vec<u8>> {
        if hex.len() % 2 != 0 {
            return Err(PluginError::Generic("Invalid hex string length".to_string()));
        }
        
        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            let byte_str = &hex[i..i+2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|e| PluginError::Generic(format!("Invalid hex string: {}", e)))?;
            bytes.push(byte);
        }
        
        Ok(bytes)
    }
}

/// 配置处理工具
pub mod config {
    use super::*;
    
    /// 配置值提取器
    pub struct ConfigExtractor {
        data: HashMap<String, serde_json::Value>,
    }
    
    impl ConfigExtractor {
        pub fn new(data: HashMap<String, serde_json::Value>) -> Self {
            Self { data }
        }
        
        /// 获取字符串配置值
        pub fn get_string(&self, key: &str) -> PluginResult<String> {
            match self.data.get(key) {
                Some(serde_json::Value::String(s)) => Ok(s.clone()),
                Some(v) => Ok(convert::json_to_string(v)),
                None => Err(PluginError::Configuration(format!("Missing config key: {}", key))),
            }
        }
        
        /// 获取可选字符串配置值
        pub fn get_string_opt(&self, key: &str) -> Option<String> {
            self.get_string(key).ok()
        }
        
        /// 获取带默认值的字符串配置值
        pub fn get_string_or(&self, key: &str, default: &str) -> String {
            self.get_string_opt(key).unwrap_or_else(|| default.to_string())
        }
        
        /// 获取数字配置值
        pub fn get_number<T>(&self, key: &str) -> PluginResult<T>
        where
            T: for<'de> Deserialize<'de>,
        {
            match self.data.get(key) {
                Some(v) => serde_json::from_value(v.clone())
                    .map_err(|e| PluginError::Configuration(format!("Invalid number config '{}': {}", key, e))),
                None => Err(PluginError::Configuration(format!("Missing config key: {}", key))),
            }
        }
        
        /// 获取布尔配置值
        pub fn get_bool(&self, key: &str) -> PluginResult<bool> {
            match self.data.get(key) {
                Some(serde_json::Value::Bool(b)) => Ok(*b),
                Some(serde_json::Value::String(s)) => {
                    match s.to_lowercase().as_str() {
                        "true" | "yes" | "on" | "1" => Ok(true),
                        "false" | "no" | "off" | "0" => Ok(false),
                        _ => Err(PluginError::Configuration(format!("Invalid boolean config '{}': {}", key, s))),
                    }
                }
                Some(v) => Err(PluginError::Configuration(format!("Invalid boolean config '{}': {}", key, v))),
                None => Err(PluginError::Configuration(format!("Missing config key: {}", key))),
            }
        }
        
        /// 获取带默认值的布尔配置值
        pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
            self.get_bool(key).unwrap_or(default)
        }
        
        /// 获取数组配置值
        pub fn get_array<T>(&self, key: &str) -> PluginResult<Vec<T>>
        where
            T: for<'de> Deserialize<'de>,
        {
            match self.data.get(key) {
                Some(serde_json::Value::Array(arr)) => {
                    let mut result = Vec::new();
                    for item in arr {
                        let value = serde_json::from_value(item.clone())
                            .map_err(|e| PluginError::Configuration(format!("Invalid array item in '{}': {}", key, e)))?;
                        result.push(value);
                    }
                    Ok(result)
                }
                Some(v) => Err(PluginError::Configuration(format!("Config '{}' is not an array: {}", key, v))),
                None => Err(PluginError::Configuration(format!("Missing config key: {}", key))),
            }
        }
        
        /// 获取带默认值的数组配置值
        pub fn get_array_or<T>(&self, key: &str, default: Vec<T>) -> Vec<T>
        where
            T: for<'de> Deserialize<'de>,
        {
            self.get_array(key).unwrap_or(default)
        }
    }
}

/// 健康检查工具
pub mod health {
    use super::*;
    
    /// 健康检查结果
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthCheck {
        /// 是否健康
        pub healthy: bool,
        /// 检查时间
        pub timestamp: u64,
        /// 检查详情
        pub details: HashMap<String, serde_json::Value>,
    }
    
    impl HealthCheck {
        pub fn new() -> Self {
            Self {
                healthy: true,
                timestamp: time::now_millis(),
                details: HashMap::new(),
            }
        }
        
        /// 添加检查详情
        pub fn add_detail<T: Serialize>(mut self, key: &str, value: T) -> Self {
            self.details.insert(key.to_string(), serde_json::to_value(value).unwrap_or_default());
            self
        }
        
        /// 标记为不健康
        pub fn unhealthy(mut self) -> Self {
            self.healthy = false;
            self
        }
        
        /// 添加错误信息
        pub fn add_error(mut self, error: &str) -> Self {
            self.healthy = false;
            self.details.insert("error".to_string(), serde_json::Value::String(error.to_string()));
            self
        }
    }
    
    /// 健康检查构建器
    pub struct HealthCheckBuilder {
        checks: Vec<Box<dyn Fn() -> PluginResult<HealthCheck>>>,
    }
    
    impl HealthCheckBuilder {
        pub fn new() -> Self {
            Self {
                checks: Vec::new(),
            }
        }
        
        /// 添加检查
        pub fn add_check<F>(mut self, check: F) -> Self
        where
            F: Fn() -> PluginResult<HealthCheck> + 'static,
        {
            self.checks.push(Box::new(check));
            self
        }
        
        /// 运行所有检查
        pub fn run(self) -> HealthCheck {
            let mut overall = HealthCheck::new();
            let mut all_healthy = true;
            
            for (i, check) in self.checks.iter().enumerate() {
                match check() {
                    Ok(result) => {
                        overall.details.insert(format!("check_{}", i), serde_json::to_value(result.clone()).unwrap_or_default());
                        if !result.healthy {
                            all_healthy = false;
                        }
                    }
                    Err(e) => {
                        all_healthy = false;
                        overall.details.insert(format!("check_{}_error", i), serde_json::Value::String(e.to_string()));
                    }
                }
            }
            
            if !all_healthy {
                overall = overall.unhealthy();
            }
            
            overall
        }
    }
}

/// 批处理工具
pub mod batch {
    use super::*;
    
    /// 批处理器
    pub struct BatchProcessor<T> {
        batch_size: usize,
        items: Vec<T>,
    }
    
    impl<T> BatchProcessor<T> {
        pub fn new(batch_size: usize) -> Self {
            Self {
                batch_size,
                items: Vec::new(),
            }
        }
        
        /// 添加项目
        pub fn add(&mut self, item: T) -> Option<Vec<T>> {
            self.items.push(item);
            if self.items.len() >= self.batch_size {
                Some(self.drain())
            } else {
                None
            }
        }
        
        /// 清空并返回所有项目
        pub fn drain(&mut self) -> Vec<T> {
            std::mem::take(&mut self.items)
        }
        
        /// 获取当前批次大小
        pub fn current_size(&self) -> usize {
            self.items.len()
        }
        
        /// 检查是否有待处理项目
        pub fn has_pending(&self) -> bool {
            !self.items.is_empty()
        }
    }
}

/// 重试工具
pub mod retry {
    use super::*;
    
    /// 重试策略
    pub enum RetryStrategy {
        /// 固定间隔
        Fixed(Duration),
        /// 指数退避
        Exponential { initial: Duration, max: Duration, multiplier: f64 },
    }
    
    /// 重试器
    pub struct Retrier {
        strategy: RetryStrategy,
        max_attempts: usize,
    }
    
    impl Retrier {
        pub fn new(strategy: RetryStrategy, max_attempts: usize) -> Self {
            Self {
                strategy,
                max_attempts,
            }
        }
        
        /// 执行重试
        pub fn retry<F, R, E>(&self, mut f: F) -> Result<R, E>
        where
            F: FnMut() -> Result<R, E>,
        {
            let mut attempts = 0;
            let mut delay = match &self.strategy {
                RetryStrategy::Fixed(d) => *d,
                RetryStrategy::Exponential { initial, .. } => *initial,
            };
            
            loop {
                attempts += 1;
                
                match f() {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        if attempts >= self.max_attempts {
                            return Err(e);
                        }
                        
                        // 简单的延迟模拟（实际应用中可能需要更复杂的延迟机制）
                        match &self.strategy {
                            RetryStrategy::Fixed(_) => {
                                // 固定延迟
                            }
                            RetryStrategy::Exponential { max, multiplier, .. } => {
                                delay = Duration::from_millis(
                                    (delay.as_millis() as f64 * multiplier) as u64
                                ).min(*max);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_utils() {
        assert_eq!(string::truncate("hello world", 5), "he...");
        assert_eq!(string::truncate("hi", 10), "hi");
        assert!(string::is_empty_or_whitespace("   "));
        assert!(!string::is_empty_or_whitespace("hello"));
    }
    
    #[test]
    fn test_convert_utils() {
        let json_val = serde_json::json!("test");
        assert_eq!(convert::json_to_string(&json_val), "test");
        
        let hex = convert::bytes_to_hex(b"hello");
        assert_eq!(hex, "68656c6c6f");
        
        let bytes = convert::hex_to_bytes(&hex).unwrap();
        assert_eq!(bytes, b"hello");
    }
    
    #[test]
    fn test_config_extractor() {
        let mut data = HashMap::new();
        data.insert("string_key".to_string(), serde_json::Value::String("test".to_string()));
        data.insert("bool_key".to_string(), serde_json::Value::Bool(true));
        data.insert("number_key".to_string(), serde_json::Value::Number(serde_json::Number::from(42)));
        
        let extractor = config::ConfigExtractor::new(data);
        
        assert_eq!(extractor.get_string("string_key").unwrap(), "test");
        assert_eq!(extractor.get_bool("bool_key").unwrap(), true);
        assert_eq!(extractor.get_number::<i32>("number_key").unwrap(), 42);
    }
    
    #[test]
    fn test_health_check() {
        let health = health::HealthCheck::new()
            .add_detail("version", "1.0.0")
            .add_detail("uptime", 3600);
        
        assert!(health.healthy);
        assert!(health.details.contains_key("version"));
    }
    
    #[test]
    fn test_batch_processor() {
        let mut processor = batch::BatchProcessor::new(3);
        
        assert!(processor.add(1).is_none());
        assert!(processor.add(2).is_none());
        let batch = processor.add(3).unwrap();
        
        assert_eq!(batch, vec![1, 2, 3]);
        assert_eq!(processor.current_size(), 0);
    }
}