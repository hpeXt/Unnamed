# CI/CD 工作流说明

## 概述

本项目采用分阶段CI策略，针对不同场景优化构建速度和反馈时间。

## 工作流文件

### 1. `ci-pr.yml` - Pull Request CI
- **触发条件**: Pull Request
- **运行时间**: 2-3分钟
- **检查内容**:
  - 代码格式化
  - Clippy警告（仅记录，不失败）
  - Linux单平台测试
- **目的**: 快速反馈，不阻塞开发

### 2. `ci.yml` - 主分支CI
- **触发条件**: 推送到main/master分支
- **运行时间**: 5-8分钟
- **阶段**:
  1. 代码质量检查（格式化、Clippy）
  2. 全平台核心测试（Linux、Windows、macOS）
  3. 插件构建
  4. 安全审计（可选）
- **目的**: 确保主分支质量

### 3. `release.yml` - 发布构建
- **触发条件**: 创建版本标签（v*.*.*）或手动触发
- **运行时间**: 15-20分钟
- **内容**:
  - 预检查
  - 全平台发布构建
  - 自动创建GitHub Release
  - 包含更新说明
- **目的**: 生成可分发的安装包

### 4. `build-test.yml` - 构建测试
- **触发条件**: 手动或特定分支
- **内容**: 测试各平台打包流程
- **目的**: 验证打包配置

## 优化策略

### 1. 缓存优化
- 分离缓存：registry、bin、target分别缓存
- 使用swatinem/rust-cache优化Rust缓存
- Node.js依赖使用npm ci和缓存

### 2. 并行化
- 代码质量检查与测试并行
- 不同平台测试并行
- 插件构建独立进行

### 3. 快速失败
- fail-fast: true - 一个平台失败立即停止其他
- 分阶段执行 - 早期阶段失败不运行后续

### 4. 资源优化
- PR只测试Linux平台
- 使用最新的GitHub Actions
- 合理的超时设置

## 本地测试

使用提供的脚本进行本地测试：

```bash
# 测试CI配置
./scripts/test-ci-local.sh

# 监控CI性能
./scripts/ci-performance-monitor.sh
```

## 自定义缓存Action

项目包含自定义缓存action：`.github/actions/cache-cargo/`

使用方法：
```yaml
- uses: ./.github/actions/cache-cargo
  with:
    cache-name: 'my-cache'
    workspaces: '. -> target'
```

## 性能指标

目标运行时间：
- PR CI: < 3分钟
- 主分支CI: < 8分钟
- 发布构建: < 20分钟

## 维护建议

1. 定期清理缓存（每月一次）
2. 更新GitHub Actions版本
3. 监控CI性能趋势
4. 根据项目增长调整策略

## 故障排查

### SQLite测试失败
- 检查CI环境变量设置
- 确认临时目录权限
- 查看测试代码中的TempDir使用

### 缓存未命中
- 检查Cargo.lock是否频繁变化
- 验证缓存key设置
- 查看缓存大小限制

### 构建超时
- 检查是否有死循环测试
- 优化编译配置
- 考虑使用更快的runner