# Minimal Kernel - 最小化内核

> 基于插件架构的本地数据处理平台

## 🎯 项目简介

Minimal Kernel 是一个实验性的本地数据处理平台，采用最小化内核架构，通过 WebAssembly 插件系统实现功能扩展。项目探索了如何构建一个安全、可扩展、本地优先的数据处理框架。

### ✨ 核心特性
- 🔌 **完整插件系统**: 基于 Extism WebAssembly，支持多语言插件
- 🏠 **本地优先**: 数据完全本地存储，离线可用
- 🔒 **零信任架构**: Alloy 身份管理，端到端加密
- ⚡ **高性能运行时**: Tokio 异步，零拷贝消息传递
- 🧩 **无限可组合**: 插件间自由通信，动态加载
- 🛠️ **开发者友好**: 丰富 SDK，完整工具链

## 🚀 快速开始

### 🛠️ 开发环境要求
- Rust 1.75+ 
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

### 📥 获取代码
```bash
# 克隆仓库
git clone https://github.com/hpeXt/Unnamed.git
cd Unnamed

# 构建项目
cargo build --release

# 构建插件（示例）
cd plugins/hello
cargo build --target wasm32-unknown-unknown --release
```

### 🔧 运行与测试
```bash
# 运行内核
cargo run

# 使用调试日志
RUST_LOG=debug cargo run

# 运行测试
cargo test

# 运行特定测试
cargo test --test integration_test
```

### 🔐 身份管理配置

Minimal Kernel 支持多种身份管理方式，适应不同使用场景：

#### 配置选项
```toml
[identity]
# 是否使用系统 keyring（默认: true）
use_keyring = true

# keyring 访问超时时间（秒）- 给用户足够时间输入系统密码
keyring_timeout_secs = 30

# 私钥文件路径（当 use_keyring = false 时使用）
# private_key_file = "~/.minimal-kernel/identity.key"

# 是否允许从环境变量加载私钥
allow_env_key = true
```

#### 使用场景

**1. 本地开发（默认）**
```bash
# 使用系统 keyring，首次运行需要输入系统密码
cargo run
```

**2. CI/CD 环境**
```bash
# 方式一：使用环境变量
export MINIMAL_KERNEL_PRIVATE_KEY="your-private-key-hex"
cargo run

# 方式二：禁用 keyring，使用文件
# 在 kernel.toml 中设置:
# use_keyring = false
# private_key_file = "/path/to/identity.key"
```

**3. 自动化测试**
```bash
# 创建测试配置
cat > test.toml << EOF
[identity]
use_keyring = false
allow_env_key = true
EOF

# 运行测试
MINIMAL_KERNEL_PRIVATE_KEY="test-key" cargo run -- --config test.toml
```

#### 安全建议
- 生产环境推荐使用 keyring 或加密文件存储
- 避免在代码中硬编码私钥
- 定期轮换密钥
- 使用环境变量时确保 CI/CD 系统的安全性

## 📁 项目架构

### 🏗️ 项目结构
```
Unnamed/
├── src/                         # 内核核心
│   ├── main.rs                  # 程序入口
│   ├── config.rs                # 配置系统
│   ├── kernel/                  # 内核模块
│   │   ├── plugin_loader.rs     # 插件加载器
│   │   ├── message_bus.rs       # 消息总线
│   │   └── host_functions.rs    # 主机函数
│   ├── storage/                 # 存储层
│   └── identity/                # 身份管理
├── plugin-sdk/                  # 插件开发 SDK
│   └── src/
│       ├── plugin.rs            # 插件接口
│       ├── error.rs             # 错误处理
│       ├── message.rs           # 消息通信
│       ├── macros.rs            # 开发宏
│       └── utils.rs             # 工具函数
├── plugins/                     # 插件示例
│   ├── hello/                   # 简单示例
│   ├── echo/                    # 回声插件
│   └── storage-test/            # 存储测试
├── src-tauri/                   # Tauri 前端
│   └── src/
│       ├── main.rs              # Tauri 主程序
│       ├── bridge.rs            # 内核桥接
│       └── container.rs         # 容器管理
├── tests/                       # 测试文件
└── migrations/                  # 数据库迁移
```

### 🔧 技术栈
- **插件引擎**: Extism (WebAssembly)
- **异步运行时**: Tokio
- **数据库**: SQLite + sqlx
- **身份管理**: Alloy (以太坊兼容)
- **桌面框架**: Tauri v2
- **开发语言**: Rust (内核) + 多语言插件支持

## 🎯 核心功能

### ✅ 插件系统
- 基于 WebAssembly 的安全执行环境
- 支持多语言插件开发（Rust、Go、JavaScript 等）
- 插件自动发现和加载
- 生命周期管理

### ✅ 消息通信
- 异步消息总线
- 发布/订阅模式
- 插件间通信隔离

### ✅ 数据存储
- SQLite 本地存储
- 插件数据隔离
- 事务支持

### ✅ 身份管理
- 以太坊兼容的身份系统
- 多种密钥存储方式
- 数字签名支持

### 🚀 使用示例

#### 创建第一个插件
```rust
use plugin_sdk::prelude::*;

#[derive(Default)]
pub struct MyPlugin {
    config: Option<PluginConfig>,
    status: PluginStatus,
}

plugin_info!(
    name: "my-plugin",
    version: "1.0.0",
    description: "我的第一个插件"
);

impl Plugin for MyPlugin {
    fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        log_info!("插件初始化成功!");
        self.config = Some(config);
        self.status = PluginStatus::Running;
        Ok(())
    }
    
    fn handle_message(&mut self, msg: PluginMessage) -> PluginResult<()> {
        log_info!("收到消息: {}", msg.topic);
        Ok(())
    }
}

plugin_main!(MyPlugin);
```

#### 运行示例插件
```bash
# 编译示例插件
cd plugins/data-collector
cargo build --target wasm32-unknown-unknown --release

# 运行内核（自动加载插件）
cd ../../
cargo run
```

## 📊 项目状态

这是一个实验性项目，核心架构已经实现：
- ✅ WebAssembly 插件引擎
- ✅ 异步消息系统
- ✅ 本地存储层
- ✅ 身份管理系统
- ✅ 基础 SDK

目前正在探索更多可能性，欢迎贡献想法和代码。

## 📚 开发资源

### 示例代码
查看 `plugins/` 目录下的示例插件：
- `hello/` - 最简单的插件示例
- `echo/` - 消息处理示例
- `storage-test/` - 存储功能示例

### 外部资源
- [Extism 官方文档](https://extism.org/) - WebAssembly 插件框架
- [Alloy 文档](https://alloy.rs/) - 以太坊开发工具

## 🌟 核心价值

### 🎯 **"一切皆插件"**
- 内核只包含最核心功能
- 所有业务逻辑以插件形式存在
- 插件可以无限组合和扩展

### 🏠 **本地优先**
- 数据完全本地存储和处理
- 无需联网即可完整工作
- 用户拥有100%数据主权

### 🔒 **零信任安全**
- 插件沙箱隔离执行
- 端到端数据加密
- 以太坊级别身份验证

### 🚀 **极客友好**
- Rust 类型安全保证
- 丰富的开发工具链
- 完整的测试和文档

## 🤝 贡献

欢迎各种形式的贡献：
- 提交问题和建议
- 开发新插件
- 改进核心功能
- 完善文档

**开始贡献**: 查看 [Issues](https://github.com/hpeXt/Unnamed/issues)

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🎉 致谢

感谢以下开源项目：
- [Extism](https://extism.org/) - WebAssembly 插件框架
- [Alloy](https://alloy.rs/) - 以太坊开发工具
- [Tokio](https://tokio.rs/) - 异步运行时
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
- [Tauri](https://tauri.app/) - 桌面应用框架

---

*"简单优于复杂，可用优于完美，本地优于云端"*