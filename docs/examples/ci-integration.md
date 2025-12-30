# CI/CD での使用例

CI/CD環境でwtenvを活用する方法を説明します。

## GitHub Actions

### 基本的なワークフロー

```yaml
# .github/workflows/feature-test.yml
name: Feature Branch Test

on:
  push:
    branches:
      - 'feature-*'
      - 'fix-*'

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install wtenv
        run: |
          curl -L https://github.com/USERNAME/wtenv/releases/latest/download/wtenv-linux-x64 -o /usr/local/bin/wtenv
          chmod +x /usr/local/bin/wtenv

      - name: Setup environment
        run: |
          # CI用の環境変数を設定
          echo "DATABASE_URL=postgres://..." >> .env
          echo "API_KEY=${{ secrets.API_KEY }}" >> .env

      - name: Run tests
        run: |
          npm install
          npm test
```

### 複数worktreeでの並列テスト

```yaml
# .github/workflows/parallel-test.yml
name: Parallel Tests

on: [push, pull_request]

jobs:
  setup:
    runs-on: ubuntu-latest
    outputs:
      worktrees: ${{ steps.create.outputs.worktrees }}

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install wtenv
        run: |
          curl -L https://github.com/USERNAME/wtenv/releases/latest/download/wtenv-linux-x64 -o /usr/local/bin/wtenv
          chmod +x /usr/local/bin/wtenv

      - name: Create worktrees for testing
        id: create
        run: |
          # 異なるバージョンのNodeでテスト用のworktreeを作成
          wtenv create test-node18 ../test-node18 --no-post-create
          wtenv create test-node20 ../test-node20 --no-post-create
          echo "worktrees=['test-node18','test-node20']" >> $GITHUB_OUTPUT

  test:
    needs: setup
    runs-on: ubuntu-latest
    strategy:
      matrix:
        worktree: ${{ fromJson(needs.setup.outputs.worktrees) }}
        node: [18, 20]

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}

      - name: Run tests
        run: |
          cd ../${{ matrix.worktree }}
          npm install
          npm test
```

## GitLab CI

### 基本設定

```yaml
# .gitlab-ci.yml
stages:
  - setup
  - test
  - cleanup

variables:
  WTENV_VERSION: "0.1.0"

.install_wtenv: &install_wtenv
  before_script:
    - curl -L https://github.com/USERNAME/wtenv/releases/download/v${WTENV_VERSION}/wtenv-linux-x64 -o /usr/local/bin/wtenv
    - chmod +x /usr/local/bin/wtenv

setup:
  stage: setup
  <<: *install_wtenv
  script:
    - wtenv create test-branch ../test-branch --no-post-create --quiet
  artifacts:
    paths:
      - ../test-branch
    expire_in: 1 hour

test:
  stage: test
  needs: [setup]
  script:
    - cd ../test-branch
    - npm install
    - npm test

cleanup:
  stage: cleanup
  <<: *install_wtenv
  script:
    - wtenv remove ../test-branch --force
  when: always
```

## Jenkins

### Jenkinsfile

```groovy
// Jenkinsfile
pipeline {
    agent any

    environment {
        WTENV_PATH = "${WORKSPACE}/../wtenv-${BUILD_NUMBER}"
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    # wtenvをインストール
                    curl -L https://github.com/USERNAME/wtenv/releases/latest/download/wtenv-linux-x64 -o /tmp/wtenv
                    chmod +x /tmp/wtenv

                    # worktreeを作成
                    /tmp/wtenv create build-${BUILD_NUMBER} ${WTENV_PATH} --no-post-create
                '''
            }
        }

        stage('Build & Test') {
            steps {
                dir("${WTENV_PATH}") {
                    sh '''
                        npm install
                        npm run build
                        npm test
                    '''
                }
            }
        }

        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                dir("${WTENV_PATH}") {
                    sh 'npm run deploy'
                }
            }
        }
    }

    post {
        always {
            sh '''
                /tmp/wtenv remove ${WTENV_PATH} --force || true
            '''
        }
    }
}
```

## CircleCI

### config.yml

```yaml
# .circleci/config.yml
version: 2.1

executors:
  node-executor:
    docker:
      - image: cimg/node:20.0

commands:
  install-wtenv:
    steps:
      - run:
          name: Install wtenv
          command: |
            curl -L https://github.com/USERNAME/wtenv/releases/latest/download/wtenv-linux-x64 -o ~/bin/wtenv
            chmod +x ~/bin/wtenv

jobs:
  build:
    executor: node-executor
    steps:
      - checkout
      - install-wtenv

      - run:
          name: Create worktree
          command: |
            wtenv create ci-build ../ci-build --no-post-create --quiet

      - run:
          name: Install dependencies
          working_directory: ../ci-build
          command: npm install

      - run:
          name: Run tests
          working_directory: ../ci-build
          command: npm test

      - run:
          name: Cleanup
          command: wtenv remove ../ci-build --force
          when: always

workflows:
  version: 2
  build-and-test:
    jobs:
      - build
```

## CI環境でのベストプラクティス

### 1. キャッシュの活用

```yaml
# GitHub Actions での例
- name: Cache node modules
  uses: actions/cache@v3
  with:
    path: |
      node_modules
      ../feature-branch/node_modules
    key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
```

### 2. 環境変数の安全な管理

```yaml
# GitHub Actions
- name: Setup environment
  run: |
    # シークレットから環境変数を設定
    echo "API_KEY=${{ secrets.API_KEY }}" >> .env
    echo "DATABASE_URL=${{ secrets.DATABASE_URL }}" >> .env

    # worktreeにコピー（設定ファイルでは管理しない）
    cp .env ../feature-branch/.env
```

### 3. 並列実行時のリソース管理

```yaml
# 同時実行数を制限
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

### 4. クリーンアップの確実な実行

```yaml
# GitHub Actions
- name: Cleanup worktrees
  if: always()  # 成功/失敗に関わらず実行
  run: |
    wtenv remove ../test-branch --force || true
```

### 5. ログの保存

```yaml
- name: Create worktree with verbose logging
  run: |
    wtenv create test-branch ../test-branch --verbose 2>&1 | tee wtenv.log

- name: Upload logs
  if: failure()
  uses: actions/upload-artifact@v3
  with:
    name: wtenv-logs
    path: wtenv.log
```

## トラブルシューティング

### Git認証の問題

```yaml
# GitHub Actions でのSSH設定
- uses: webfactory/ssh-agent@v0.8.0
  with:
    ssh-private-key: ${{ secrets.SSH_PRIVATE_KEY }}
```

### ディスク容量の問題

```yaml
# 古いworktreeを削除
- name: Cleanup old worktrees
  run: |
    wtenv list | grep -E "ci-build|test-" | awk '{print $1}' | xargs -I {} wtenv remove {} --force || true
```

### 権限の問題

```yaml
# ファイル権限を修正
- name: Fix permissions
  run: |
    chmod -R 755 ../test-branch
```
