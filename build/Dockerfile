# Tauri 1.0 ARM64 Linux 构建环境
# 用于在 Docker 中打包 Linux ARM64 (aarch64) 的 .deb 和 .AppImage

FROM --platform=linux/arm64 debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

# 安装系统依赖 (Tauri Linux 构建所需)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# 安装 Node.js 20 LTS
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# 安装 pnpm
RUN npm install -g pnpm

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
    && rustup default stable

# 安装 Tauri CLI
RUN . $HOME/.cargo/env && cargo install tauri-cli --version ^1

# 设置工作目录
WORKDIR /app

# 复制项目文件 (通过 .dockerignore 排除 node_modules, target 等)
COPY . .

# 安装前端依赖并构建
RUN cd frontend && pnpm install

# 构建命令 (可在运行时覆盖)
# 使用 --bundles deb 可单独构建 .deb (跨平台编译时推荐)
# 本镜像为 ARM64 原生，可同时生成 deb 和 AppImage
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always

# 默认构建命令: 先构建前端，再构建 Tauri (生成 deb 和 AppImage)
CMD cd /app/frontend && pnpm build:tauri && cd /app && cargo tauri build --bundles deb,appimage
