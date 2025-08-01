# Minimal Kernel - 最小化内核

> 基于插件架构的本地数据处理平台

## 🎯 项目简介

Minimal Kernel 是一个基于插件架构的本地数据处理平台，专注于个人健康数据管理。采用最小化内核设计理念，通过 WebAssembly 插件系统实现无限扩展性，让用户完全掌控自己的数据。

**项目愿景**：为注重隐私的个人用户打造一个安全、可扩展、本地优先的数字健康管理平台。

## 📸 快速概览

```
┌─────────────────────────────────────────┐
│            用户界面 (Tauri)              │
├─────────────────────────────────────────┤
│         插件层 (WebAssembly)            │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │健康监测│ │数据分析│ │ 可视化 │  │
│  └────┬────┘ └────┬────┘ └────┬────┘  │
├───────┴────────────┴────────────┴───────┤
│           最小化内核 (Rust)             │
│  消息总线 │ 存储系统 │ 身份管理       │
└─────────────────────────────────────────┘
```

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
- Node.js 18+ (用于 Tauri)
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

### 📥 获取代码
```bash
# 克隆仓库
git clone https://github.com/hpeXt/Unnamed.git
cd Unnamed

# 构建核心项目
cargo build --release

# 构建插件（示例）
cd plugins/hello
cargo build --target wasm32-unknown-unknown --release
cd ../..

# 运行 Tauri 应用（可选）
cd src-tauri
cargo tauri dev
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

### 当前版本：v0.01（技术预览）
核心架构已经实现：
- ✅ WebAssembly 插件引擎（Extism）
- ✅ 异步消息系统（Tokio + 发布订阅）
- ✅ 本地存储层（SQLite + sqlx）
- ✅ 身份管理系统（Alloy 以太坊兼容）
- ✅ 插件 SDK（完整的开发工具包）
- ✅ Tauri 桌面应用框架
- ✅ 基础 UI 容器管理

### 目标版本：v0.1（首个公开版本）
**预计发布时间**：2025年8月22日（3周开发周期）

## 🗺️ 开发路线图

### 📅 第一阶段：安装体验（第1-3天）
- [ ] 创建首次启动引导界面
- [ ] 完善 Windows 打包（.exe/.msi）
- [ ] 完善 Linux 打包（.deb/.AppImage）  
- [ ] 优化 macOS 打包体验
- [ ] 创建安装后的欢迎界面
- [ ] 实现基础配置向导

### 📅 第二阶段：插件开发工具（第4-7天）
- [ ] 实现 `minimal-kernel create-plugin` 命令行工具
- [ ] 创建插件项目模板生成器
- [ ] 编写插件开发快速入门教程
- [ ] 实现插件热重载机制
- [ ] 创建插件调试控制台
- [ ] 发布插件 SDK 到 crates.io

### 📅 第三阶段：核心功能增强（第8-12天）
- [ ] 实现插件依赖管理系统
- [ ] 添加插件权限控制
- [ ] 创建插件市场原型
- [ ] 实现数据导入/导出功能
- [ ] 添加插件性能监控
- [ ] 优化插件加载速度

### 📅 第四阶段：UI/UX 优化（第13-16天）
- [ ] 实现拖拽式布局编辑器
- [ ] 创建主题系统（深色/浅色）
- [ ] 添加键盘快捷键支持
- [ ] 实现响应式布局
- [ ] 创建设置界面
- [ ] 优化整体视觉设计

### 📅 第五阶段：健康数据基础（第17-19天）
- [ ] 定义健康数据模型（FHIR 兼容）
- [ ] 创建基础健康监测组件
- [ ] 实现数据可视化图表
- [ ] 添加数据趋势分析
- [ ] 创建健康数据仪表盘模板

### 📅 第六阶段：测试与发布（第20-21天）
- [ ] 完整的端到端测试
- [ ] 跨平台兼容性测试
- [ ] 性能优化和内存泄漏检查
- [ ] 编写用户文档
- [ ] 准备发布材料
- [ ] 发布 v0.1 版本

## 🎯 版本对比

| 特性 | v0.01（当前） | v0.1（目标） |
|------|--------------|-------------|
| **安装方式** | 源码构建 | 一键安装包 |
| **平台支持** | macOS 测试 | macOS/Windows/Linux |
| **插件开发** | 手动创建 | 脚手架工具 |
| **插件示例** | 3个 | 10+ 个 |
| **UI 组件** | 基础容器 | 15+ 种组件 |
| **文档完整度** | 60% | 95% |
| **用户体验** | 开发者向 | 用户友好 |

## 🚀 未来展望（v0.2+）

### v0.2 - 社区版本（2025年10月）
- 插件市场和发现机制
- 社区插件分享平台
- 高级数据分析功能
- AI 驱动的健康建议

### v1.0 - 稳定版本（2026年）
- P2P 网络集成
- 分布式数据同步
- 端到端加密通信
- 企业级功能支持

## 📚 开发资源

### 示例代码
查看 `plugins/` 目录下的示例插件：
- `hello/` - 最简单的插件示例
- `echo/` - 消息处理示例
- `storage-test/` - 存储功能示例

### 外部资源
- [Extism 官方文档](https://extism.org/) - WebAssembly 插件框架
- [Alloy 文档](https://alloy.rs/) - 以太坊开发工具

## 🌟 核心价值与设计哲学

### 🎯 **"一切皆插件"**
- 内核只包含最核心功能（消息传递、存储、身份管理）
- 所有业务逻辑以插件形式存在
- 插件可以无限组合和扩展

### 🏠 **本地优先**
- 数据完全本地存储和处理
- 无需联网即可完整工作
- 用户拥有100%数据主权

### 🔒 **零信任安全**
- WebAssembly 沙箱隔离执行
- 插件间严格权限控制
- 以太坊兼容的身份系统

### 🚀 **开发者友好**
- Rust 类型安全保证
- 丰富的开发工具和宏
- 完整的测试框架

## 💡 为什么选择这个架构？

### 技术选型理由
- **Rust**: 内存安全、高性能、零运行时开销
- **WebAssembly**: 安全沙箱、跨语言支持、接近原生性能
- **Tauri**: 原生应用体验、小体积、高安全性
- **SQLite**: 本地存储、事务支持、零配置
- **Tokio**: 高性能异步运行时、成熟稳定

### 架构优势
1. **模块化设计**: 核心功能与业务逻辑完全解耦
2. **安全隔离**: 每个插件运行在独立沙箱中
3. **易于扩展**: 新功能只需添加插件，无需修改核心
4. **跨平台**: 一次开发，全平台运行
5. **面向未来**: 为 P2P 和分布式扩展预留接口

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