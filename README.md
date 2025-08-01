# Minimal Kernel - 最小化内核

> 基于插件架构的本地数据处理平台

## 🎯 项目简介

Minimal Kernel 是一个基于插件架构的本地数据处理平台，专注于个人健康数据管理。采用最小化内核设计理念，通过 WebAssembly 插件系统实现无限扩展性，让用户完全掌控自己的数据。

**项目愿景**：为注重隐私的个人用户打造一个安全、可扩展、本地优先的数字健康管理平台。

## 📸 系统架构概览

### 整体架构
```
┌──────────────────────────────────────────────────────────┐
│                    用户界面层 (Tauri)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  仪表盘组件  │  │  健康卡片   │  │  数据图表   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
├─────────┴─────────────────┴─────────────────┴────────────┤
│                    Tauri IPC 通信层                       │
│                    (双向消息传递)                          │
├───────────────────────────────────────────────────────────┤
│                  内核桥接层 (KernelBridge)                │
│              (消息路由、状态管理、事件分发)                │
├───────────────────────────────────────────────────────────┤
│                   插件运行时 (Extism)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ 健康监测插件 │  │ 数据分析插件 │  │ AI 建议插件 │     │
│  │   (WASM)    │  │   (WASM)    │  │   (WASM)    │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
├─────────┴─────────────────┴─────────────────┴────────────┤
│                    最小化内核 (Rust)                      │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │  消息总线   │  │  存储系统  │  │  身份管理  │        │
│  │(Pub/Sub)   │  │ (SQLite)   │  │  (Alloy)   │        │
│  └────────────┘  └────────────┘  └────────────┘        │
└──────────────────────────────────────────────────────────┘
```

### 🔄 信息流与交互模型

#### 1. 前端到后端的数据流
```
用户操作 → UI组件 → Tauri Command → KernelBridge → MessageBus → WASM Plugin
                                                           ↓
                                                      数据处理
                                                           ↓
UI更新 ← Tauri Event ← KernelBridge ← MessageBus ← 处理结果
```

#### 2. 插件间通信流
```
Plugin A                    MessageBus                    Plugin B
   │                            │                            │
   ├──publish("topic", data)──→│                            │
   │                            ├──route message──────────→│
   │                            │                            ├─process
   │                            │←───publish response────────┤
   │←───receive response────────┤                            │
```

#### 3. 数据存储流
```
Plugin → Storage Request → Host Function → SQLite → Encrypted Data
                                              ↓
Plugin ← Storage Response ← Host Function ← Query Result
```

## 🎯 目标用户体验

### 最终交互愿景
1. **一键安装**：下载安装包，双击即可使用
2. **可视化配置**：拖拽式仪表盘编辑，所见即所得
3. **插件商店**：浏览、安装、管理插件，类似 VS Code 扩展
4. **健康洞察**：AI 驱动的健康建议和趋势分析
5. **数据导出**：支持多种格式导出，与医疗系统兼容

### 核心使用场景
```
用户场景示例：
1. 早晨打开应用 → 查看睡眠质量分析 → 获得今日健康建议
2. 连接智能设备 → 自动同步数据 → 实时监测健康指标
3. 月度健康报告 → 趋势分析 → 导出给医生参考
4. 安装新插件 → 扩展监测维度 → 个性化健康管理
```

## 🏗️ 核心组件功能详解

### 1. 前端层（Tauri + Web）
- **仪表盘系统**：可自定义的卡片式布局
- **实时更新**：WebSocket 推送数据变化
- **响应式设计**：适配桌面和平板
- **主题系统**：深色/浅色模式切换
- **国际化**：多语言支持

### 2. 内核桥接层（KernelBridge）
- **消息路由**：前端请求分发到对应插件
- **状态管理**：维护应用全局状态
- **事件聚合**：收集插件事件并推送前端
- **权限控制**：验证前端请求的合法性
- **缓存层**：优化频繁请求的性能

### 3. 插件系统（WebAssembly）
- **安全隔离**：每个插件独立沙箱运行
- **资源限制**：CPU/内存使用量控制
- **标准接口**：统一的插件生命周期
- **热更新**：无需重启即可更新插件
- **依赖管理**：自动处理插件依赖关系

### 4. 数据存储（SQLite）
- **加密存储**：敏感数据端到端加密
- **版本控制**：数据模式自动迁移
- **备份恢复**：定期自动备份
- **数据隔离**：插件数据互相隔离
- **查询优化**：索引和查询计划优化

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

## 🗺️ 详细开发路线图

### 📅 第一阶段：安装与首次体验优化（第1-3天）
**目标**：让用户能够轻松安装并快速上手

#### Day 1: 打包优化
- [ ] 配置 macOS 代码签名和公证
- [ ] 设置 Windows 安装程序（WiX）
- [ ] 配置 Linux 打包（AppImage/deb）
- [ ] 创建自动更新机制

#### Day 2: 首次启动体验
- [ ] 设计欢迎界面 UI
- [ ] 实现引导教程（交互式）
- [ ] 创建示例数据生成器
- [ ] 添加快速配置向导

#### Day 3: 安装程序完善
- [ ] 添加安装选项（路径、快捷方式等）
- [ ] 实现卸载清理逻辑
- [ ] 创建便携版本
- [ ] 编写安装故障排除指南

### 📅 第二阶段：插件开发生态（第4-7天）
**目标**：让开发者能在5分钟内创建第一个插件

#### Day 4-5: CLI 工具开发
```bash
# 目标命令
minimal-kernel create-plugin my-health-monitor
minimal-kernel dev --hot-reload
minimal-kernel build --release
minimal-kernel publish
```

#### Day 6: 开发体验优化
- [ ] VS Code 扩展（语法高亮、自动完成）
- [ ] 插件模板集合（健康、金融、生产力等）
- [ ] 实时日志和调试工具
- [ ] 性能分析工具

#### Day 7: 文档和教程
- [ ] 视频教程：10分钟创建健康监测插件
- [ ] API 参考文档（自动生成）
- [ ] 最佳实践指南
- [ ] 常见问题解答

### 📅 第三阶段：核心系统强化（第8-12天）
**目标**：构建稳定、安全、高性能的插件运行环境

#### Day 8-9: 插件管理系统
- [ ] 依赖解析算法（拓扑排序）
- [ ] 版本兼容性检查
- [ ] 插件沙箱资源配额
- [ ] 自动更新机制

#### Day 10: 权限系统
```typescript
// 权限示例
{
  "permissions": {
    "storage": ["read", "write"],
    "network": ["fetch"],
    "system": ["notifications"],
    "plugins": ["message:health-data"]
  }
}
```

#### Day 11-12: 插件市场
- [ ] 市场 UI 设计（类似 VS Code）
- [ ] 搜索和筛选功能
- [ ] 评分和评论系统
- [ ] 一键安装流程

### 📅 第四阶段：用户界面革新（第13-16天）
**目标**：打造直观、美观、高效的用户界面

#### Day 13-14: 布局系统
- [ ] 拖拽组件库（React DnD）
- [ ] 网格布局系统（CSS Grid）
- [ ] 布局模板保存/加载
- [ ] 响应式断点设计

#### Day 15: 主题和个性化
- [ ] 主题引擎（CSS 变量）
- [ ] 自定义主题编辑器
- [ ] 动画和过渡效果
- [ ] 无障碍功能（ARIA）

#### Day 16: 交互优化
- [ ] 全局快捷键系统
- [ ] 命令面板（Cmd+K）
- [ ] 上下文菜单
- [ ] 撤销/重做系统

### 📅 第五阶段：健康数据功能（第17-19天）
**目标**：实现核心健康数据管理功能

#### Day 17: 数据模型
```typescript
// FHIR 兼容的数据模型
interface HealthRecord {
  resourceType: "Observation" | "Condition" | "Medication";
  subject: PatientReference;
  effectiveDateTime: string;
  valueQuantity?: Quantity;
  interpretation?: CodeableConcept[];
}
```

#### Day 18: 可视化组件
- [ ] 心率变异性图表
- [ ] 睡眠阶段分析
- [ ] 血压趋势图
- [ ] 活动热力图

#### Day 19: 智能分析
- [ ] 异常检测算法
- [ ] 趋势预测（移动平均）
- [ ] 健康评分计算
- [ ] 个性化建议生成

### 📅 第六阶段：质量保证与发布（第20-21天）
**目标**：确保软件质量，顺利发布 v0.1

#### Day 20: 测试冲刺
- [ ] 自动化 UI 测试（Playwright）
- [ ] 性能基准测试
- [ ] 内存泄漏检测
- [ ] 安全审计

#### Day 21: 发布准备
- [ ] 生成发布说明
- [ ] 更新官网
- [ ] 准备演示视频
- [ ] 发布到 GitHub Releases

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

## 🔧 技术实现细节

### 消息传递机制
```rust
// 异步消息发布
kernel.publish("health.heartrate", json!({
    "value": 72,
    "timestamp": "2025-08-01T10:30:00Z",
    "device": "apple-watch"
})).await?;

// 订阅消息
kernel.subscribe("health.*", |msg| {
    println!("收到健康数据: {:?}", msg);
}).await?;
```

### 插件通信接口
```rust
// 插件间直接通信
let response = plugin.call("analyze-health-data", json!({
    "type": "heartrate",
    "period": "last-7-days"
})).await?;

// 批量数据处理
let batch_result = plugin.process_batch(vec![
    HealthData::HeartRate(72),
    HealthData::BloodPressure(120, 80),
    HealthData::Steps(8500),
]).await?;
```

### 前端集成示例
```typescript
// Tauri 命令调用
const healthData = await invoke('get_health_summary', {
    userId: currentUser.id,
    dateRange: 'last-30-days'
});

// 实时数据订阅
listen('health-update', (event) => {
    updateDashboard(event.payload);
});

// 插件管理
const plugins = await invoke('list_plugins');
await invoke('install_plugin', { 
    name: 'sleep-analyzer',
    version: '1.2.0' 
});
```

### 数据安全实现
```rust
// 端到端加密存储
let encrypted_data = crypto::encrypt(
    &health_record,
    &user_key
)?;
storage.insert("health_records", encrypted_data)?;

// 权限验证
#[tauri::command]
#[require_permission("health:read")]
async fn get_health_data(user_id: String) -> Result<HealthData> {
    // 只有授权用户可以访问
}
```

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

## 🔗 快速链接

### 开发资源
- [插件开发指南](https://github.com/hpeXt/Unnamed/wiki/Plugin-Development)
- [API 文档](https://github.com/hpeXt/Unnamed/wiki/API-Reference)
- [架构设计文档](https://github.com/hpeXt/Unnamed/wiki/Architecture)

### 社区
- [GitHub Issues](https://github.com/hpeXt/Unnamed/issues) - 报告问题和功能建议
- [Discussions](https://github.com/hpeXt/Unnamed/discussions) - 社区讨论
- [Roadmap](https://github.com/hpeXt/Unnamed/projects/1) - 项目进度跟踪

### 相关项目
- [Extism](https://extism.org/) - WebAssembly 插件框架
- [Tauri](https://tauri.app/) - 桌面应用框架
- [FHIR](https://www.hl7.org/fhir/) - 健康数据标准

---

**Minimal Kernel** - 为个人健康数据管理而生，由开源社区驱动

*"简单优于复杂，可用优于完美，本地优于云端"* ✨