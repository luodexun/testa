pipeline {
    agent any

    environment {
        NODE_ENV = 'production'
    }

    tools {
            nodejs "18.14.2"  // 需要在 Jenkins 中配置 Node.js
        }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
                withCredentials([usernamePassword(
                    credentialsId: 'luodexun'
                )]) {
                    sh '''
                      set -e

                      # 为需要鉴权的域配置 http extra header，供 submodule 使用
                      git submodule sync --recursive || true
                      git submodule update --init --recursive
                    '''
                }
            }
        }

        stage('Install dependencies') {
            steps {
                sh '''
                  set -e

                  cd frontend

                  if command -v corepack >/dev/null 2>&1; then
                    corepack enable || true
                  fi

                  if ! command -v pnpm >/dev/null 2>&1; then
                    npm install -g pnpm
                  fi

                  pnpm install
                '''
            }
        }

        stage('Build') {
            steps {
                sh '''
                  set -e
                  cd frontend
                  pnpm run build
                '''
            }
        }

        stage('Archive artifacts') {
            steps {
                dir('frontend') {
                    archiveArtifacts artifacts: 'dist/**/*', fingerprint: true, onlyIfSuccessful: true
                }
            }
        }
    }
}
