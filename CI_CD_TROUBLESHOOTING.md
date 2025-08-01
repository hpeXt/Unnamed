# CI/CD 故障排除指南

## 当前已解决的问题

### 1. SQLite INTEGER 类型映射问题
- **问题**: SQLx 默认将 SQLite INTEGER 映射到 Rust i64，但代码中使用了 i32
- **解决**: 统一所有整数字段为 i64，保持类型一致性
- **理由**: 符合"简单优于完美"的项目哲学
- **状态**: ✅ 已完成

### 2. SQLX_OFFLINE 模式配置
- **问题**: CI 环境无法连接数据库进行编译时查询验证
- **解决**: 添加 `SQLX_OFFLINE=true` 环境变量到所有 CI 配置
- **状态**: ✅ 已完成

### 3. Security Audit 优化
- **问题**: cargo-audit 安装缓慢导致超时
- **解决**: 使用二进制下载方式，添加 fallback 机制
- **状态**: ✅ 已完成

### 4. DateTime 类型转换问题 
- **问题**: SQLite TIMESTAMP 与 Rust DateTime<Utc> 类型不匹配
- **错误**: `the trait bound 'DateTime<Utc>: From<Option<NaiveDateTime>>' is not satisfied`
- **最终解决**: 移除 SQLx 编译时宏，改用运行时查询
- **实施细节**:
  1. 将所有 `sqlx::query_as!()` 宏替换为 `sqlx::query()`
  2. 手动映射每个字段
  3. 添加辅助函数 `naive_to_utc()` 处理时间转换
  4. 删除 `sqlx-data.json` 文件
- **状态**: ✅ 已完成

## 关键解决方案总结

**CI/CD 编译失败的根本原因**：SQLx 编译时宏要求在编译时连接数据库进行查询验证，但 CI 环境中没有数据库。

**最佳解决方案**：移除 SQLx 编译时宏，使用运行时查询。这个方案：
- 完全消除了编译时对数据库的依赖
- 保持了代码的灵活性和可维护性
- 符合项目"简单优于完美"的哲学
- 不需要维护额外的元数据文件

## 实施步骤记录

### 第一阶段：移除编译时宏（已完成）
1. **识别问题**：SQLite 的 TIMESTAMP 类型映射到 `Option<NaiveDateTime>`，而代码使用 `DateTime<Utc>`
2. **移除编译时宏**：将 `sqlx::query_as!()` 替换为 `sqlx::query()`
3. **手动映射字段**：使用 `row.get()` 方法获取每个字段
4. **处理时间转换**：添加 `naive_to_utc()` 辅助函数
5. **清理文件**：删除不再需要的 `sqlx-data.json`

### 第二阶段：清理CI配置（已完成）
1. **移除SQLx CLI安装步骤**：不再需要 `cargo install sqlx-cli`
2. **移除数据库初始化**：不再需要 `sqlx database create` 和 `sqlx migrate run`
3. **保留运行时环境变量**：保留 `DATABASE_URL: sqlite:data.db` 供运行时使用

## 未来建议

- 如果需要更强的类型安全，可以考虑使用 sea-orm 或 diesel
- 目前的解决方案简单有效，符合最小化内核的设计理念