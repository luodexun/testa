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
               checkout([
                                  $class: 'GitSCM',
                                  branches: [[name: '*/main']],
                                  doGenerateSubmoduleConfigurations: false,
                                  extensions: [
                                      [$class: 'SubmoduleOption',
                                       disableSubmodules: false,
                                       parentCredentials: true,      // 关键：允许子模块使用主仓库凭据
                                       recursiveSubmodules: true,    // 递归拉取子模块
                                       reference: '',
                                       trackingSubmodules: false,
                                       shallow: false,
                                       depth: 0,
                                       timeout: 10]
                                  ],
                                  userRemoteConfigs: [[
                                      url: scm.userRemoteConfigs[0].url,  // 使用 SCM 配置的 URL
                                      credentialsId: 'github-token'       // 明确指定凭据
                                  ]]
                              ])
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
