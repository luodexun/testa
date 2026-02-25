pipeline {
    agent any

    tools {
            nodejs "18.14.2"  // 需要在 Jenkins 中配置 Node.js
            dockerTool 'local-docker'
        }

    environment {
        // 定义变量
        IMAGE_NAME = "ness-arm64-${env.BUILD_ID}"  // 每次构建使用不同的镜像名
        PROJECT_ROOT = "${WORKSPACE}"
        OUTPUT_DIR = "/Users/luodexun/Documents/deb"
    }

    stages {
        stage('Checkout') {
            steps {
                // 使用 Git 插件拉取主仓库
                checkout([
                    $class: 'GitSCM',
                    branches: [[name: '*/main']],  // 替换为你的分支
                    extensions: [
                        [
                            $class: 'SubmoduleOption',
                            parentCredentials: true,  // 使用父仓库凭据
                            recursiveSubmodules: true,  // 递归拉取子模块
                            reference: '',
                            trackingSubmodules: false,
                            disableSubmodules: false,
                            shallowClone: true,
                            timeout: null
                        ],
                        [$class: 'CloneOption', depth: 1, shallow: true, timeout: 60],
                        [$class: 'CleanBeforeCheckout']
                    ],
                    userRemoteConfigs: [[
                        url: 'https://github.com/luodexun/testa.git',  // 替换为你的仓库地址
                        credentialsId: 'github-token'  // 替换为你的凭据ID
                    ]]
                ])
            }
        }

        stage('准备环境') {
            steps {
                script {
                    // 创建输出目录
                    sh 'mkdir -p ${OUTPUT_DIR}'
                    echo "📦 开始构建 Tauri ARM64 Linux 应用"
                    echo "   项目目录: ${PROJECT_ROOT}"
                    echo "   输出目录: ${OUTPUT_DIR}"
                    echo "   镜像名称: ${IMAGE_NAME}"
                }
            }
        }

        stage('构建Docker镜像') {
            steps {
                script {
                    echo "🚀 构建 Docker 镜像..."
                    sh """
                        docker build --platform linux/arm64 \
                          -f ${PROJECT_ROOT}/build/Dockerfile \
                          -t ${IMAGE_NAME} \
                          ${PROJECT_ROOT}
                    """
                }
            }
        }

        stage('在容器中构建Tauri应用') {
            steps {
                script {
                    echo "🔨 在容器中执行 Tauri 构建..."
                    sh """
                        docker run --rm --platform linux/arm64 \
                          -e NODE_OPTIONS="--max-old-space-size=8192" \
                          -v ${OUTPUT_DIR}:/output \
                          ${IMAGE_NAME} \
                          sh -c '
                            . /root/.cargo/env 2>/dev/null || . \$HOME/.cargo/env
                            cd /app && cargo tauri build --bundles deb,appimage
                            cp -r target/release/bundle/deb/*.deb /output/ 2>/dev/null || true
                            cp -r target/release/bundle/appimage/*.AppImage /output/ 2>/dev/null || true
                          '
                    """
                }
            }
        }
    }

    post {
        always {
            // cleanWs()  // 清理工作空间
             script {
                echo "🧹 清理 Docker 资源..."
                // 清理本次构建创建的镜像
                sh "docker rmi -f ${IMAGE_NAME} 2>/dev/null || true"
                // 清理无用的 Docker 资源
                sh 'docker image prune -f 2>/dev/null || true'
            }
        }
    }
}