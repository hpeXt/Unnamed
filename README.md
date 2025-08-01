# Minimal Kernel - 最小化内核

> 长寿极客的数字孪生平台 - **MVP 已完成！**

## 🎯 项目简介

Minimal Kernel 是一个本地优先的数据聚合与分析平台，专为长寿极客设计。采用最小化内核架构，通过插件系统实现无限扩展性。现已集成 Tauri 仪表盘系统，打造健康数据管理的完整解决方案。

**🏆 当前状态：健康数据仪表盘系统集成完成，支持自定义可视化组件**

### ✨ 核心特性
- 🔌 **完整插件系统**: 基于 Extism WebAssembly，支持多语言插件
- 🏠 **本地优先**: 数据完全本地存储，离线可用
- 🔒 **零信任架构**: Alloy 身份管理，端到端加密
- ⚡ **高性能运行时**: Tokio 异步，零拷贝消息传递
- 🧩 **无限可组合**: 插件间自由通信，动态加载
- 🛠️ **开发者友好**: 丰富 SDK，完整工具链

## 🚀 快速开始

### 📦 下载安装（推荐）
直接下载预编译的应用程序：
- 🍎 **macOS**: [下载 .dmg](https://github.com/minimal-kernel/minimal-kernel/releases)
- 🪟 **Windows**: [下载 .msi](https://github.com/minimal-kernel/minimal-kernel/releases)
- 🐧 **Linux**: [下载 .deb/.AppImage](https://github.com/minimal-kernel/minimal-kernel/releases)

详细安装说明请参考 [📖 安装指南](INSTALL.md)

### 🛠️ 从源码构建
需要先安装开发环境：
- Rust 1.75+ 
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

```bash
# 克隆仓库
git clone https://github.com/minimal-kernel/minimal-kernel
cd minimal-kernel

# 快速开始（自动安装依赖和演示功能）
./scripts/quick-start.sh

# 构建桌面应用
./build-app.sh

# 构建示例插件
./build-plugins.sh
```

### 🔧 开发模式
```bash
# 实时监听文件变化
cargo watch -x run

# 运行不同日志级别
cargo run -- --log-level debug
cargo run -- --log-level trace

# 使用环境变量控制日志
RUST_LOG=debug cargo run
RUST_LOG=minimal_kernel::kernel=trace cargo run

# 运行测试
cargo test

# 运行集成测试
cargo test --test integration_test

# 运行日志示例
cargo run --example logging_examples

# CLI 功能演示
./scripts/demo-cli.sh
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

### 🏗️ 完整项目结构
```
minimal-kernel/
├── 🟢 src/                      # 内核核心 (100% 完成)
│   ├── main.rs                  # 程序入口
│   ├── config.rs                # 配置系统
│   ├── kernel/                  # 内核模块
│   │   ├── mod.rs               # 统一内核接口
│   │   ├── plugin_loader.rs     # 插件加载器
│   │   ├── message_bus.rs       # 消息总线
│   │   └── host_functions.rs    # 主机函数
│   ├── storage/                 # 存储层
│   │   └── mod.rs               # SQLite 集成
│   └── identity/                # 身份管理
│       └── mod.rs               # Alloy 以太坊身份
├── 🟢 plugin-sdk/               # 插件开发 SDK (100% 完成)
│   ├── src/
│   │   ├── plugin.rs            # 插件核心接口
│   │   ├── error.rs             # 错误处理系统
│   │   ├── message.rs           # 消息通信
│   │   ├── host.rs              # 主机函数封装
│   │   ├── macros.rs            # 开发辅助宏
│   │   ├── utils.rs             # 工具函数库
│   │   └── testing.rs           # 测试工具
│   └── README.md                # SDK 文档
├── 🟢 plugins/                  # 插件生态 (示例完成)
│   ├── data-collector/          # 数据收集插件
│   ├── analyzer/                # 数据分析插件
│   ├── hello/                   # 简单示例
│   ├── ui-system-monitor/       # 系统监控 UI 插件
│   └── ...                     # 更多插件
├── 🟢 src-tauri/                # Tauri 前端应用 (新增)
│   ├── src/
│   │   ├── main.rs              # Tauri 主程序
│   │   ├── bridge.rs            # 内核桥接层
│   │   ├── container.rs         # UI 容器管理
│   │   └── system_monitor.rs    # 系统监控
│   └── Cargo.toml               # Tauri 依赖配置
├── 🟢 tests/                    # 测试体系 (90% 完成)
│   ├── integration_test.rs      # 集成测试
│   ├── e2e_message_test.rs      # 端到端消息测试
│   └── identity_test.rs         # 身份管理测试
├── 🟢 docs/                     # 文档体系 (95% 完成)
│   ├── plugin-development-guide.md  # 插件开发指南
│   ├── extism-dev-guide.md      # Extism 使用指南
│   └── alloy-usage-guide.md     # Alloy 使用指南
└── 🟢 migrations/               # 数据库迁移
    ├── 20240717000001_plugin_data.sql
    └── 20240717000002_plugin_metadata.sql
```

### 🔧 技术栈详情
```yaml
运行时:
  - 插件引擎: Extism (WebAssembly)
  - 异步运行时: Tokio
  - 数据库: SQLite + sqlx
  - 身份管理: Alloy (以太坊兼容)
  - 密钥存储: keyring-rs
  - 桌面框架: Tauri v2

开发语言:
  - 内核: Rust (100% 类型安全)
  - 插件: 任何支持 Extism PDK 的语言
    └── Rust, Go, JavaScript, Python, C++, Zig...
  - UI 插件: 任何 Web 技术栈
    └── HTML, JavaScript, React, Vue, Svelte...

架构模式:
  - 消息传递: Actor 模型 + 发布订阅
  - 数据流: 事件驱动 + 流处理
  - 安全模型: 零信任 + 沙箱隔离
  - UI 架构: 插件化多容器 + 消息桥接
```

## 🎯 功能演示

### ✅ 已实现功能

#### 🔌 插件系统
```bash
# 插件自动发现和加载
✅ 扫描 plugins/ 目录
✅ WASM 插件安全执行
✅ 动态生命周期管理
✅ 热重载支持
```

#### 📨 消息通信
```bash
# 插件间通信
✅ 主题订阅/发布
✅ 点对点消息传递
✅ 消息优先级和过期
✅ 异步非阻塞处理
```

#### 💾 数据存储
```bash
# 本地数据管理
✅ SQLite 键值存储
✅ 插件数据隔离
✅ 事务安全
✅ 自动备份
```

#### 🔐 身份管理
```bash
# 以太坊兼容身份
✅ 私钥安全存储
✅ 数字签名验证
✅ 地址生成管理
✅ keyring 集成
✅ 多种存储方式（keyring/文件/环境变量）
✅ 灵活配置选项
```

#### 🖥️ Tauri 仪表盘系统（新增）
```bash
# 原生桌面应用体验
✅ Tauri v2 框架集成
✅ 多 WebView 容器管理
✅ 插件化 UI 架构
✅ 内核消息桥接（KernelBridge）
✅ 动态 UI 插件加载
✅ 实时数据可视化
✅ 支持任何 Web 技术栈（HTML/JS/React/Vue）
```

运行 Tauri 仪表盘：
```bash
# 开发模式
cd src-tauri
npm install
npm run tauri dev

# 构建生产版本
npm run tauri build
```

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

## 📊 项目里程碑

### ✅ Phase 1: 核心架构 (已完成)
- [x] WebAssembly 插件引擎集成
- [x] 异步消息总线系统
- [x] SQLite 存储抽象层
- [x] 以太坊身份管理
- [x] 配置系统设计

### ✅ Phase 2: 插件生态 (已完成)
- [x] 完整插件 SDK 开发
- [x] 开发辅助宏系统
- [x] 测试工具和框架
- [x] 示例插件实现
- [x] 详尽开发文档

### 🔄 Phase 3: 增强功能 (进行中)
- [ ] 高级插件示例
- [ ] 性能优化和监控
- [ ] 管理界面开发
- [ ] 插件市场设计

### 🎯 Phase 4: 分布式扩展 (规划中)
- [ ] P2P 网络集成
- [ ] 分布式存储同步
- [ ] 跨节点插件通信
- [ ] 去中心化身份

## 📚 学习资源

### 🎓 快速入门
1. [插件开发指南](docs/plugin-development-guide.md) - 从零创建插件
2. [SDK API 文档](plugin-sdk/README.md) - 完整 API 参考
3. [示例插件](plugins/) - 实际代码示例

### 🔧 技术深入
1. [Extism 使用指南](docs/extism-dev-guide.md) - WebAssembly 插件开发
2. [Alloy 使用指南](docs/alloy-usage-guide.md) - 以太坊身份管理
3. [测试指南](TEST_GUIDE.md) - 测试最佳实践

### 🏗️ 架构设计
1. [CLAUDE.md](CLAUDE.md) - 项目设计哲学
2. [技术决策文档](docs/) - 架构选择说明

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

## 🤝 贡献指南

我们欢迎各种形式的贡献！

### 🔧 代码贡献
- 新插件开发
- 核心功能增强
- 性能优化
- Bug 修复

### 📚 文档贡献
- 改进文档
- 添加示例
- 翻译工作

### 💡 想法贡献
- 功能建议
- 架构改进
- 用户体验优化

### 🧪 测试贡献
- 增加测试覆盖
- 性能基准测试
- 兼容性测试

**开始贡献**: 查看 [Issues](https://github.com/long-life-geek/minimal-kernel/issues) 寻找适合的任务

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🎉 致谢

感谢以下开源项目和社区：
- [Extism](https://extism.org/) - WebAssembly 插件框架
- [Alloy](https://alloy.rs/) - 以太坊开发工具
- [Tokio](https://tokio.rs/) - 异步运行时
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包

---

**🚀 为长寿极客的数字孪生时代而构建 - Minimal Kernel Team**

*"简单优于复杂，可用优于完美，本地优于云端"* ✨