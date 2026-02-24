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
                                   extensions: [
                                       [$class: 'SubmoduleOption',
                                        parentCredentials: true,
                                        recursiveSubmodules: true,
                                        reference: '',
                                        trackingSubmodules: false,
                                        disableSubmodules: false,
                                        timeout: 10,
                                        shallow: false,
                                        depth: 0]
                                   ],
                                   userRemoteConfigs: [[url: 'https://github.com/luodexun/testa.git']]
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
