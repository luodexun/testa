#!/bin/bash
# Tauri ARM64 Linux Docker 构建脚本
# 在项目根目录执行: ./build/build.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="${1:-ness-arm64}"
OUTPUT_DIR="${2:-$PROJECT_ROOT/build/dist}"

echo "📦 构建 Tauri ARM64 Linux 镜像: $IMAGE_NAME"
echo "   项目目录: $PROJECT_ROOT"
echo "   输出目录: $OUTPUT_DIR"
echo ""

# 构建 Docker 镜像
docker build --platform linux/arm64 \
  -f "$SCRIPT_DIR/Dockerfile" \
  -t "$IMAGE_NAME" \
  "$PROJECT_ROOT"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 运行容器并复制产物
# 产物位于 target/release/bundle/deb/ 和 target/release/bundle/appimage/
echo ""
echo "🔨 在容器中执行构建..."
docker run --rm --platform linux/arm64 \
  -e NODE_OPTIONS="--max-old-space-size=8192" \
  -v "$OUTPUT_DIR:/output" \
  "$IMAGE_NAME" \
  sh -c '
    . /root/.cargo/env 2>/dev/null || . $HOME/.cargo/env
    cd /app && cargo tauri build --bundles deb,appimage
    cp -r target/release/bundle/deb/*.deb /output/ 2>/dev/null || true
    cp -r target/release/bundle/appimage/*.AppImage /output/ 2>/dev/null || true
  '

echo ""
echo "✅ 构建完成！产物已输出到: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR" 2>/dev/null || echo "   (未找到产物文件)"
