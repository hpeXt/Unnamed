# 自建Runner配置指南

## 为什么需要自建Runner？

1. **速度提升**: 预装依赖，避免重复下载
2. **成本控制**: 减少GitHub Actions使用分钟数
3. **更好的缓存**: 本地持久化缓存
4. **自定义环境**: 特殊硬件或软件需求

## 配置步骤

### 1. 准备服务器

推荐配置：
- CPU: 4核以上
- 内存: 8GB以上
- 存储: 100GB SSD
- 系统: Ubuntu 22.04 LTS

### 2. 安装Runner

```bash
# 创建runner用户
sudo useradd -m -s /bin/bash runner
sudo usermod -aG docker runner

# 切换到runner用户
sudo -u runner -i

# 下载runner
cd ~
mkdir actions-runner && cd actions-runner
curl -o actions-runner-linux-x64-2.317.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.317.0/actions-runner-linux-x64-2.317.0.tar.gz
tar xzf ./actions-runner-linux-x64-2.317.0.tar.gz

# 配置runner（需要从GitHub仓库设置获取token）
./config.sh --url https://github.com/YOUR_ORG/YOUR_REPO --token YOUR_TOKEN
```

### 3. 预装依赖

创建安装脚本 `setup-runner.sh`:

```bash
#!/bin/bash
set -e

# 更新系统
sudo apt-get update
sudo apt-get upgrade -y

# 安装构建依赖
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    curl \
    wget \
    git \
    jq

# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# 安装Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 安装Docker
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER

# 安装sccache
cargo install sccache

# 安装其他工具
cargo install cargo-watch cargo-edit sqlx-cli tauri-cli

# 配置sccache
mkdir -p ~/.config/sccache
cat > ~/.config/sccache/config.toml << EOF
[cache.disk]
dir = "/home/runner/.cache/sccache"
size = 10737418240  # 10 GB
EOF

echo "✅ Runner环境配置完成！"
```

### 4. 配置为系统服务

```bash
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

### 5. 在workflow中使用自建runner

```yaml
jobs:
  build:
    runs-on: [self-hosted, linux, x64]
    # 或使用标签
    # runs-on: [self-hosted, rust-builder]
```

## 优化配置

### 1. 持久化缓存

```yaml
# .github/workflows/ci.yml
- name: Setup cache directories
  run: |
    mkdir -p ~/.cargo/registry
    mkdir -p ~/.cargo/git
    mkdir -p ~/.sccache
    mkdir -p ~/node_modules_cache
```

### 2. 并发构建

```bash
# 在runner上配置
echo "export CARGO_BUILD_JOBS=4" >> ~/.bashrc
echo "export SCCACHE_CACHE_SIZE=20G" >> ~/.bashrc
```

### 3. 监控和维护

创建监控脚本 `monitor-runner.sh`:

```bash
#!/bin/bash

# 检查runner状态
if ! systemctl is-active --quiet actions.runner.*.service; then
    echo "❌ Runner服务未运行"
    sudo systemctl start actions.runner.*.service
fi

# 检查磁盘空间
DISK_USAGE=$(df -h /home/runner | awk 'NR==2 {print $5}' | sed 's/%//')
if [ $DISK_USAGE -gt 80 ]; then
    echo "⚠️ 磁盘使用率高: $DISK_USAGE%"
    # 清理缓存
    rm -rf /home/runner/.cache/sccache/*
    rm -rf /home/runner/actions-runner/_work/*/*
fi

# 检查内存
FREE_MEM=$(free -m | awk 'NR==2 {print $4}')
if [ $FREE_MEM -lt 1000 ]; then
    echo "⚠️ 可用内存低: ${FREE_MEM}MB"
fi
```

### 4. 安全考虑

1. **网络隔离**: Runner应该在独立的网络环境
2. **权限限制**: 使用专用用户运行，避免root
3. **定期更新**: 保持系统和依赖最新
4. **监控日志**: 定期检查异常活动

## 多Runner配置

为不同任务配置专用runner：

```yaml
# 标记runner
./config.sh --labels linux,rust,build-server

# 在workflow中使用
jobs:
  test:
    runs-on: [self-hosted, linux, test-server]
  
  build:
    runs-on: [self-hosted, linux, build-server]
```

## 故障排除

### Runner离线
```bash
sudo systemctl restart actions.runner.*.service
sudo journalctl -u actions.runner.*.service -f
```

### 清理工作目录
```bash
cd ~/actions-runner
./cleanup.sh
```

### 更新runner
```bash
./svc.sh stop
./svc.sh uninstall
# 下载新版本
./svc.sh install
./svc.sh start
```

## 成本效益分析

使用自建runner vs GitHub hosted runner:

| 项目 | GitHub Hosted | 自建Runner |
|-----|--------------|-----------|
| 设置成本 | 低 | 高 |
| 运行成本 | $0.008/分钟 | 服务器成本 |
| 速度 | 标准 | 快2-5倍 |
| 缓存 | 有限 | 无限 |
| 自定义 | 受限 | 完全控制 |

适合自建runner的场景：
- 每月CI时间超过3000分钟
- 需要特殊环境或依赖
- 对构建速度有高要求
- 需要本地资源访问