# 插件模板

这是最小化内核的标准插件模板，展示了如何正确使用主机函数和开发插件。

## 重要：主机函数命名规范

**关键原则：插件必须遵循主机系统定义的接口。**

在我们的系统中，所有主机函数都使用 `_host` 后缀：
- ✅ 正确：`log_message_host`
- ❌ 错误：`log_message`

## 快速开始

### 1. 复制模板

```bash
cp -r plugins/template plugins/my-plugin
cd plugins/my-plugin
```

### 2. 修改 Cargo.toml

更新包名和描述：

```toml
[package]
name = "my-plugin"
description = "我的自定义插件"
```

### 3. 修改插件代码

编辑 `src/lib.rs`，更新：
- 插件名称
- 插件 ID
- 业务逻辑

### 4. 构建插件

```bash
cargo build --target wasm32-unknown-unknown --release
```

## 主机函数使用方式

### 方式一：使用 SDK 提供的包装函数（推荐）

```rust
// SDK 会自动调用正确的主机函数
log_info!("这是一条日志");
host::storage::store(&plugin_id, "key", &value)?;
```

### 方式二：直接调用主机函数

```rust
// 必须使用带 _host 后缀的函数名
unsafe {
    log_message_host("info", "直接调用主机函数")?;
    store_data_host(&plugin_id, &key, &value)?;
}
```

## 可用的主机函数

| 功能类别 | 函数名 | 说明 |
|---------|--------|------|
| 日志 | `log_message_host` | 记录日志消息 |
| 存储 | `store_data_host` | 存储数据 |
| 存储 | `get_data_host` | 获取数据 |
| 存储 | `delete_data_host` | 删除数据 |
| 存储 | `list_keys_host` | 列出所有键 |
| 消息 | `send_message_host` | 发送消息 |
| 消息 | `subscribe_topic_host` | 订阅主题 |
| 消息 | `unsubscribe_topic_host` | 取消订阅 |
| 消息 | `publish_message_host` | 发布消息 |

## 测试插件

### 单元测试

```bash
cargo test
```

### 集成测试

1. 构建插件
2. 将 WASM 文件复制到内核的插件目录
3. 运行内核并观察日志

## 调试技巧

1. **启用调试日志**：在配置中设置 `debug_enabled: true`
2. **检查函数名**：确保使用了正确的主机函数名（带 _host 后缀）
3. **查看内核日志**：主机函数错误会在内核日志中显示
4. **测试主机函数**：使用 `test_host_functions` 命令测试所有主机函数

## 最佳实践

1. **优先使用 SDK**：SDK 提供了更好的类型安全和错误处理
2. **处理错误**：总是检查主机函数的返回值
3. **记录日志**：在关键操作前后记录日志
4. **保存状态**：在关闭时保存重要状态
5. **验证配置**：在初始化时验证配置的有效性

## 常见错误

### 错误：`undefined symbol: log_message`

**原因**：使用了错误的函数名
**解决**：改为 `log_message_host`

### 错误：主机函数调用失败

**原因**：返回值格式不正确
**解决**：检查响应是否为 JSON 格式的 `HostResponse`

## 参考资源

- [插件开发指南](../../docs/plugin-development-guide.md)
- [Extism 开发指南](../../docs/extism-dev-guide.md)
- [Plugin SDK 文档](../../plugin-sdk/README.md)