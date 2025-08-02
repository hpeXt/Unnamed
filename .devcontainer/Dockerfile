FROM rust:1.79-bookworm

# 安装基础工具
RUN apt-get update && apt-get install -y \
    # 构建依赖
    build-essential \
    pkg-config \
    libssl-dev \
    # Tauri依赖
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # 开发工具
    git \
    curl \
    wget \
    vim \
    tmux \
    htop \
    jq \
    ripgrep \
    fd-find \
    # SQLite工具
    sqlite3 \
    libsqlite3-dev \
    # Node.js（前端构建）
    nodejs \
    npm \
    # Python（某些工具需要）
    python3 \
    python3-pip \
    # 调试工具
    gdb \
    lldb \
    valgrind \
    && rm -rf /var/lib/apt/lists/*

# 安装Rust工具
RUN rustup component add rustfmt clippy rust-analyzer
RUN rustup target add wasm32-unknown-unknown

# 安装cargo工具
RUN cargo install \
    cargo-watch \
    cargo-edit \
    cargo-audit \
    cargo-outdated \
    cargo-expand \
    cargo-chef \
    sccache \
    sqlx-cli \
    tauri-cli

# 安装act（本地CI测试）
RUN curl https://raw.githubusercontent.com/nektos/act/master/install.sh | bash

# 设置sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/workspace/.sccache

# 创建非root用户
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid $USER_GID vscode \
    && useradd --uid $USER_UID --gid $USER_GID -m vscode \
    && echo vscode ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/vscode \
    && chmod 0440 /etc/sudoers.d/vscode

# 设置工作目录
WORKDIR /workspace

# 切换到vscode用户
USER vscode

# 配置git（避免安全警告）
RUN git config --global --add safe.directory /workspace

# 安装oh-my-zsh（可选，提升开发体验）
RUN sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended

# 设置默认shell
ENV SHELL=/usr/bin/zsh