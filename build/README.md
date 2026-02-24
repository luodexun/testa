# Tauri ARM64 Linux 构建

使用 Docker 在 ARM64 (aarch64) 架构下构建 Tauri 应用的 Linux 包（.deb 和 .AppImage）。

## 前置要求

- Docker（需支持多平台构建，即 `docker buildx` 或 Docker Desktop）
- 在 x86_64 机器上构建 ARM64 时，Docker 会通过 QEMU 模拟，构建速度较慢
- **Docker 内存**：若在容器内构建前端，建议 Docker Desktop 分配至少 8GB 内存

## 快速开始

### 方式一：Docker Compose + 卷挂载

```bash
# 1. 在项目根目录，先主机构建前端
cd frontend && NODE_OPTIONS=--max-old-space-size=8192 pnpm build:tauri && cd ..

# 2. 创建 tauri.linux.conf.json 跳过 Docker 内前端构建
echo '{"$schema":"https://schema.tauri.app/config/1","build":{"beforeBuildCommand":""}}' > tauri.linux.conf.json

# 3. 使用 Compose 运行（卷挂载项目目录和输出目录）
docker compose -f build/docker-compose.yml --profile build run --rm tauri-arm64

# 4. 构建完成后删除临时配置
rm -f tauri.linux.conf.json
```

### 方式二：分阶段构建脚本（推荐，避免 Docker 内存不足）

```bash
# 主机构建前端 + Docker 打包，适合 Docker 内存有限时使用
./build/build-with-host-frontend.sh
```

### 方式三：全在 Docker 内构建

```bash
# 在项目根目录执行
./build/build.sh
```

默认会：
- 构建 Docker 镜像 `ness-arm64`
- 在容器内编译前端和 Tauri 应用
- 将 `.deb` 和 `.AppImage` 输出到 `./dist-arm64/`

### 2. 自定义镜像名和输出目录

```bash
./build/build.sh <镜像名> <输出目录>
# 示例
./build/build.sh ness-arm64 ./my-output
```

### 3. 手动执行

```bash
# 构建镜像
docker build --platform linux/arm64 -f build/Dockerfile -t ness-arm64 .

# 运行构建（挂载输出目录）
mkdir -p dist-arm64
docker run --rm --platform linux/arm64 \
  -v "$(pwd)/dist-arm64:/output" \
  ness-arm64 \
  sh -c '. $HOME/.cargo/env && cd /app && cargo tauri build --bundles deb,appimage && \
    cp target/release/bundle/deb/*.deb /output/ 2>/dev/null; \
    cp target/release/bundle/appimage/*.AppImage /output/ 2>/dev/null'
```

## 配置文件说明

打包相关配置已统一在 **`tauri.conf.json`** 中：

- `build.beforeBuildCommand`：`cwd: "frontend"` 指定从 frontend 目录执行前端构建
- `tauri.bundle.targets`：`["dmg", "deb", "appimage"]` 表示：
  - macOS 构建 dmg
  - Linux 构建 deb 和 AppImage

- **`.dockerignore`**：位于项目根目录，用于排除 `node_modules`、`target` 等以加速构建

## 仅构建 .deb（跨平台编译场景）

若在 x86_64 上交叉编译 ARM64，AppImage 无法生成，可仅构建 deb：

```bash
docker run --rm --platform linux/arm64 -v "$(pwd)/dist-arm64:/output" ness-arm64 \
  sh -c '. $HOME/.cargo/env && cd /app && cargo tauri build --bundles deb && \
    cp target/release/bundle/deb/*.deb /output/'
```

## 注意事项

1. **构建时间**：在 x86_64 上模拟 ARM64 时，首次构建可能需 30 分钟以上
2. **内存**：建议至少 4GB 可用内存
3. **网络**：需能访问 npm 和 crates.io
