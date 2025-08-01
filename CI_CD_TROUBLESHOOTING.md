# CI/CD 故障排除指南

## 当前已解决的问题

### 1. SQLite INTEGER 类型映射问题
- **问题**: SQLx 默认将 SQLite INTEGER 映射到 Rust i64，但代码中使用了 i32
- **解决**: 统一所有整数字段为 i64，保持类型一致性
- **理由**: 符合"简单优于完美"的项目哲学

### 2. SQLX_OFFLINE 模式配置
- **问题**: CI 环境无法连接数据库进行编译时查询验证
- **解决**: 添加 `SQLX_OFFLINE=true` 环境变量到所有 CI 配置
- **状态**: ✅ 已完成

### 3. Security Audit 优化
- **问题**: cargo-audit 安装缓慢导致超时
- **解决**: 使用二进制下载方式，添加 fallback 机制
- **状态**: ✅ 已完成

## 待解决的问题

### 1. DateTime 类型转换问题
- **问题**: SQLite TIMESTAMP 与 Rust DateTime<Utc> 类型不匹配
- **错误**: `the trait bound 'DateTime<Utc>: From<Option<NaiveDateTime>>' is not satisfied`
- **建议解决方案**:
  1. 使用 SQLx 的类型注解：`created_at as "created_at: DateTime<Utc>"`
  2. 或者使用自定义类型转换
  3. 或者生成完整的 sqlx-data.json 文件

### 2. 生成正确的 SQLx 元数据
需要在本地环境执行：
```bash
# 确保数据库存在
./scripts/init-database.sh

# 生成元数据
cargo sqlx prepare

# 提交生成的文件
git add .sqlx/ sqlx-data.json
git commit -m "chore: 更新 SQLx 元数据文件"
```

## 临时解决方案

如果 CI 仍然失败，可以考虑：

1. **跳过部分测试**
   ```yaml
   - name: Run tests
     run: cargo test --all-features || true
     continue-on-error: true
   ```

2. **使用特性标志**
   ```toml
   [features]
   ci = []
   ```
   然后在 CI 中使用 `--no-default-features --features ci`

3. **分离存储层测试**
   将需要数据库的测试单独运行

## 长期改进建议

1. **考虑使用 sea-orm 或 diesel**
   - 更成熟的 ORM 解决方案
   - 更好的类型映射支持

2. **使用 Docker 进行 CI 测试**
   - 提供一致的数据库环境
   - 避免离线模式的复杂性

3. **简化数据模型**
   - 减少不必要的时间戳字段
   - 使用更简单的类型