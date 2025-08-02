#!/bin/bash
# DevContainer后创建脚本

set -e

echo "🚀 设置开发环境..."

# 创建必要的目录
mkdir -p /workspace/data
mkdir -p /workspace/.cargo
mkdir -p /workspace/.rustup
mkdir -p /workspace/.sccache

# 配置Cargo镜像（中国用户）
if [ "$USE_CN_MIRROR" = "true" ]; then
    cat > /workspace/.cargo/config.toml << EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

[net]
git-fetch-with-cli = true
EOF
    echo "✅ 已配置Cargo中国镜像"
fi

# 安装前端依赖
if [ -f "frontend/package.json" ]; then
    echo "📦 安装前端依赖..."
    cd frontend
    npm install
    cd ..
fi

# 初始化数据库
if [ ! -f "/workspace/data/data.db" ]; then
    echo "🗄️ 初始化数据库..."
    sqlx database create
    sqlx migrate run
fi

# 预编译依赖（加速后续构建）
echo "🔨 预编译依赖..."
cargo build --all-features || true

# 构建插件
echo "🔌 构建插件..."
for plugin in plugins/*/; do
    if [ -f "$plugin/Cargo.toml" ]; then
        echo "  构建插件: $(basename $plugin)"
        (cd "$plugin" && cargo build --target wasm32-unknown-unknown --release) || true
    fi
done

# 显示环境信息
echo ""
echo "✅ 开发环境准备完成！"
echo ""
echo "环境信息："
echo "  Rust: $(rustc --version)"
echo "  Cargo: $(cargo --version)"
echo "  Node: $(node --version)"
echo "  npm: $(npm --version)"
echo ""
echo "可用命令："
echo "  cargo run              - 运行主程序"
echo "  cargo watch -x run     - 监视模式运行"
echo "  cargo test            - 运行测试"
echo "  ./scripts/test-ci-local.sh - 本地运行CI"
echo ""
echo "Happy coding! 🎉"