# 插件 SDK (Plugin SDK)

最小化内核的插件开发工具包，提供插件开发所需的所有基础设施和工具。

## 特性

- 🔌 **完整的插件接口**: 统一的 Plugin trait 和生命周期管理
- 🛠️ **开发辅助宏**: 简化插件开发的宏系统
- 📨 **消息通信**: 插件间通信和消息处理机制
- 💾 **存储抽象**: 简化的数据存储接口
- 🔍 **错误处理**: 统一的错误类型和处理机制
- 🧪 **测试工具**: 完整的测试辅助和模拟工具
- 📊 **工具函数**: 常用的工具函数和辅助功能

## 快速开始

### 1. 创建新插件

在 `plugins/` 目录下创建新的插件项目：

```bash
mkdir plugins/my-plugin
cd plugins/my-plugin
```

创建 `Cargo.toml`:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-sdk = { path = "../../plugin-sdk" }
extism-pdk = "1.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. 编写插件代码

创建 `src/lib.rs`:

```rust
use plugin_sdk::prelude::*;

pub struct MyPlugin {
    config: Option<PluginConfig>,
    status: PluginStatus,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            config: None,
            status: PluginStatus::Uninitialized,
        }
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// 定义插件元数据
plugin_info!(
    name: "my-plugin",
    version: "0.1.0",
    description: "我的第一个插件",
    author: "Your Name",
    tags: ["example", "demo"]
);

impl Plugin for MyPlugin {
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        log_info!("初始化我的插件");
        self.config = Some(config);
        self.status = PluginStatus::Running;
        Ok(())
    }
    
    fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()> {
        log_info!("收到消息: {}", message.topic);
        // 处理消息逻辑
        Ok(())
    }
    
    fn tick(&mut self) -> PluginResult<()> {
        // 定时任务逻辑
        Ok(())
    }
}

// 生成插件入口点
plugin_main!(MyPlugin);
```

### 3. 编译插件

```bash
cargo build --target wasm32-unknown-unknown --release
```

## 核心概念

### Plugin Trait

所有插件都必须实现 `Plugin` trait：

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn status(&self) -> PluginStatus;
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()>;
    fn handle_event(&mut self, event: PluginEvent) -> PluginResult<()>;
    
    // 可选实现的方法
    fn tick(&mut self) -> PluginResult<()> { Ok(()) }
    fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()>;
    fn get_config(&self) -> Option<&PluginConfig>;
    fn shutdown(&mut self) -> PluginResult<()>;
    fn health_check(&self) -> PluginResult<HashMap<String, serde_json::Value>>;
    fn get_stats(&self) -> PluginResult<HashMap<String, serde_json::Value>>;
}
```

### 插件生命周期

1. **Uninitialized** → **Initialize** → **Running**
2. **Running** → **Tick** (定期调用)
3. **Running** → **Handle Message** (收到消息时)
4. **Running** → **Shutdown** → **Shutdown**

### 消息通信

插件间通过消息进行通信：

```rust
// 发送消息
let message = PluginMessage::builder("my-plugin")
    .to("target-plugin")
    .topic("data")
    .payload_json(&data)?
    .build()?;

// 处理消息
fn handle_message(&mut self, message: PluginMessage) -> PluginResult<()> {
    match message.topic.as_str() {
        "data" => {
            let data: MyData = message.payload_json()?;
            // 处理数据
        }
        _ => {}
    }
    Ok(())
}
```

### 数据存储

使用简化的存储接口：

```rust
// 存储数据
store_data!("my-plugin", "key", &value)?;

// 获取数据
let value: MyType = get_data!("my-plugin", "key", MyType)?
    .unwrap_or_default();
```

### 配置管理

从配置中提取值：

```rust
fn get_my_config(&self) -> MyConfig {
    if let Some(config) = &self.config {
        let extractor = plugin_sdk::utils::config::ConfigExtractor::new(config.data.clone());
        
        MyConfig {
            interval: extractor.get_number("interval").unwrap_or(5000u64),
            enabled: extractor.get_bool_or("enabled", true),
            targets: extractor.get_array("targets").unwrap_or_default(),
        }
    } else {
        MyConfig::default()
    }
}
```

## 开发辅助宏

### plugin_main!

自动生成插件入口点：

```rust
plugin_main!(MyPlugin);
```

### plugin_info!

定义插件元数据：

```rust
plugin_info!(
    name: "my-plugin",
    version: "1.0.0",
    description: "插件描述",
    author: "作者名",
    dependencies: ["dep1", "dep2"],
    tags: ["tag1", "tag2"]
);
```

### 日志宏

```rust
log_error!("错误信息: {}", error);
log_warn!("警告信息");
log_info!("信息");
log_debug!("调试信息: {}", data);
log_trace!("跟踪信息");
```

### 存储宏

```rust
store_data!("plugin-id", "key", &value)?;
let value = get_data!("plugin-id", "key", Type)?;
```

### 消息订阅宏

```rust
subscribe_topics!("plugin-id", "topic1", "topic2", "topic3")?;
```

### 错误处理宏

```rust
// 创建错误
let error = plugin_error!(Configuration, "配置错误: {}", msg);

// 条件检查
ensure!(condition, plugin_error!(Validation, "验证失败"));

// 尝试执行，失败时记录日志
try_or_log!(some_operation(), "操作失败");
```

## 工具函数

### 时间工具

```rust
use plugin_sdk::utils::time;

let now = time::now_millis();
let formatted = time::format_timestamp(timestamp);
let timer = time::Timer::new("操作名称");
// 当 timer 被 drop 时自动记录耗时
```

### 字符串工具

```rust
use plugin_sdk::utils::string;

let truncated = string::truncate("long string", 10);
let sanitized = string::sanitize(user_input);
let random = string::random_string(16);
```

### 数据转换工具

```rust
use plugin_sdk::utils::convert;

let hex = convert::bytes_to_hex(&bytes);
let bytes = convert::hex_to_bytes(&hex)?;
let json_str = convert::json_to_string(&json_value);
```

### 健康检查工具

```rust
use plugin_sdk::utils::health;

let health = health::HealthCheck::new()
    .add_detail("version", "1.0.0")
    .add_detail("uptime", uptime_seconds);

// 或使用构建器
let health = health::HealthCheckBuilder::new()
    .add_check(|| {
        // 检查逻辑
        Ok(health::HealthCheck::new().add_detail("status", "ok"))
    })
    .run();
```

## 测试

### 使用测试工具

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::testing::*;
    
    #[test]
    fn test_my_plugin() {
        let mut plugin = MyPlugin::new();
        let config = TestConfigBuilder::new()
            .add_string("key", "value")
            .add_number("port", 8080)
            .build();
        
        // 测试完整生命周期
        test_plugin_lifecycle!(plugin, config);
    }
    
    #[test]
    fn test_message_handling() {
        let mut plugin = MyPlugin::new();
        let config = TestConfigBuilder::new().build();
        plugin.initialize(config).unwrap();
        
        let message = TestMessageBuilder::simple("sender", "my-plugin", "test data");
        assert_plugin_ok!(plugin.handle_message(message));
    }
    
    #[test]
    fn test_with_mock_storage() {
        let storage = MockStorage::new();
        storage.store("my-plugin", "key", &"value").unwrap();
        
        TestAssertions::assert_storage_contains_key(&storage, "my-plugin", "key");
        TestAssertions::assert_storage_value(&storage, "my-plugin", "key", "value".to_string());
    }
}
```

### 测试断言

```rust
// 插件状态断言
TestAssertions::assert_plugin_status(&plugin, PluginStatus::Running);
TestAssertions::assert_plugin_has_config(&plugin);

// 消息断言
TestAssertions::assert_message_content(&message, "expected content");
TestAssertions::assert_message_topic(&message, "expected topic");
TestAssertions::assert_message_priority(&message, MessagePriority::High);

// 配置断言
TestAssertions::assert_config_value(&config, "key", expected_value);

// 存储断言
TestAssertions::assert_storage_contains_key(&storage, "plugin", "key");
TestAssertions::assert_storage_value(&storage, "plugin", "key", expected_value);
```

### 测试宏

```rust
// 断言错误类型
assert_plugin_error!(result, PluginError::Configuration);

// 断言成功
assert_plugin_ok!(result);

// 测试插件生命周期
test_plugin_lifecycle!(plugin, config);
```

## 示例插件

查看 `plugins/` 目录下的示例插件：

- **data-collector**: 数据收集插件，展示定时任务和数据存储
- **analyzer**: 数据分析插件，展示消息处理和数据分析

## 最佳实践

1. **错误处理**: 使用 SDK 提供的错误类型，添加适当的上下文信息
2. **日志记录**: 合理使用不同级别的日志，便于调试和监控
3. **配置管理**: 提供合理的默认值，验证配置的有效性
4. **资源清理**: 在 `shutdown` 方法中正确清理资源
5. **测试覆盖**: 为核心功能编写测试，使用 SDK 提供的测试工具
6. **文档注释**: 为公共接口添加文档注释

## API 参考

### 模块结构

- `plugin_sdk::plugin` - 插件核心接口
- `plugin_sdk::error` - 错误处理
- `plugin_sdk::message` - 消息通信
- `plugin_sdk::host` - 主机函数包装
- `plugin_sdk::utils` - 工具函数
- `plugin_sdk::testing` - 测试辅助 (仅测试时可用)
- `plugin_sdk::prelude` - 常用导入

### 重要类型

- `Plugin` - 插件 trait
- `PluginMetadata` - 插件元数据
- `PluginConfig` - 插件配置
- `PluginStatus` - 插件状态
- `PluginMessage` - 插件消息
- `PluginError` - 插件错误
- `PluginResult<T>` - 插件结果类型

## 许可证

本项目采用 MIT 许可证。