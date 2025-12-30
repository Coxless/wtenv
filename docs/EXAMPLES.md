# wtenv - Usage Examples

このドキュメントでは、wtenvの実践的な使用例を紹介します。

## 基本的なワークフロー

### 1. プロジェクトのセットアップ

```bash
# Gitリポジトリに移動
cd ~/projects/myapp

# wtenv設定ファイルを初期化
wtenv init

# 設定ファイルを編集（オプション）
vim .worktree.yml
```

### 2. 新しい機能開発

```bash
# 新機能用のworktreeを作成
wtenv create feature-authentication

# 作成されたworktreeに移動
cd ../myapp-feature-authentication

# 開発作業...
# （wtenvのpostCreateコマンドで自動的に npm install などが実行される）

# 現在の状態を確認
wtenv status
```

### 3. 複数の機能を並列開発

```bash
# 複数のworktreeを作成
wtenv create feature-auth
wtenv create feature-payment
wtenv create bugfix-login

# すべてのworktreeの状態を確認
wtenv status
```

**出力例:**
```
┌─────────────────────────────────────────────────────────────┐
│ Worktrees Overview (4 active, 0 processes)                  │
├─────────────────────────────────────────────────────────────┤
│ 📁 main                           (main)                     │
│    Status: Clean                  No process                │
│    Last commit: 1 day ago                                   │
│                                                              │
│ 🔄 feature-auth                   main → feature-auth       │
│    Status: Modified (5 files)     No process                │
│    Modified: 5 files  |  Last commit: 10m ago               │
│                                                              │
│ 🔄 feature-payment                main → feature-payment    │
│    Status: Modified (3 files)     No process                │
│    Modified: 3 files  |  Last commit: 30m ago               │
│                                                              │
│ ✅ bugfix-login                   main → bugfix-login       │
│    Status: Ahead: 2 commits       No process                │
│    Last commit: 1h ago                                      │
├─────────────────────────────────────────────────────────────┤
│ 📊 Total: 4 worktrees  |  Modified: 8 files                │
└─────────────────────────────────────────────────────────────┘
```

## プロセス管理の活用

### テストを並列実行

各worktreeで異なるテストを実行し、進捗を監視できます。

```bash
# feature-authでE2Eテストを開始
cd ../myapp-feature-auth
pnpm test:e2e &

# feature-paymentでユニットテストを開始
cd ../myapp-feature-payment
pnpm test:unit &

# メインworktreeに戻る
cd ../myapp

# 実行中のプロセスを確認
wtenv ps
```

**出力例:**
```
Active Processes in Worktrees:

feature-auth (PID: 12345)
  Command: pnpm test:e2e
  Started: 2m 30s ago
  Working Dir: /home/user/projects/myapp-feature-auth
  Status: Running

feature-payment (PID: 12346)
  Command: pnpm test:unit
  Started: 1m 15s ago
  Working Dir: /home/user/projects/myapp-feature-payment
  Status: Running

Total: 2 processes
```

### プロセスの停止

```bash
# 特定のworktreeのプロセスを停止
wtenv kill feature-auth

# すべてのプロセスを停止
wtenv kill --all

# 特定のPIDを停止
wtenv kill 12345
```

## 複数Claude Codeセッションの管理（将来実装予定）

```bash
# 各worktreeでClaude Codeセッションを起動
cd ../myapp-feature-auth
code .

cd ../myapp-feature-payment
code .

# メインworktreeで状態を確認
cd ../myapp
wtenv status
```

## 環境変数の管理

### 設定ファイルの例

```yaml
# .worktree.yml
version: 1

copy:
  - .env
  - .env.local
  - .env.development

exclude:
  - .env.production
  - .env.staging

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."
  - command: npm run db:migrate
    description: "データベースをマイグレーション中..."
    optional: true
```

### worktreeごとに異なる環境変数

```bash
# feature-auth用の.envを編集
cd ../myapp-feature-auth
echo "API_PORT=3001" >> .env.local

# feature-payment用の.envを編集
cd ../myapp-feature-payment
echo "API_PORT=3002" >> .env.local

# 環境変数の違いを確認
wtenv diff-env feature-auth feature-payment
```

**出力例:**
```
🔍 feature-auth と feature-payment の環境変数の違い:

.env.local:
  API_PORT:
    - 3001
    + 3002
```

## クリーンアップ

### 不要なworktreeを削除

```bash
# 単一のworktreeを削除
wtenv remove ../myapp-feature-auth

# 強制削除（変更があっても削除）
wtenv remove ../myapp-feature-auth --force
```

### マージ済みブランチのクリーンアップ

```bash
# ドライラン（削除候補を表示）
wtenv clean --dry-run

# マージ済みworktreeのみ削除
wtenv clean --merged-only

# 30日以上更新されていないworktreeを削除
wtenv clean --stale-days 30

# 確認なしで強制削除
wtenv clean --force

# マージ済みかつ30日以上古いworktreeを削除
wtenv clean --merged-only --stale-days 30 --force
```

## トラブルシューティング

### プロセスが表示されない場合

プロセス情報は `.worktree/processes.json` に保存されます。手動で起動したプロセスは追跡されません。

```bash
# プロセス情報をクリーンアップ
wtenv ps  # 死んだプロセスは自動的に削除される
```

### 設定ファイルのバリデーション

```bash
# 設定ファイルの確認
wtenv config

# 詳細情報を表示
wtenv config --verbose
```

### worktreeリストの更新

```bash
# すべてのworktreeを表示
wtenv list

# 詳細情報を表示
wtenv list --verbose
```

## ベストプラクティス

### 1. プロジェクトごとに設定ファイルを作成

各プロジェクトのルートに `.worktree.yml` を配置し、必要なファイルコピーとpost-createコマンドを定義します。

### 2. 命名規則を統一

```bash
# Good: 目的が明確
wtenv create feature-user-authentication
wtenv create bugfix-login-error
wtenv create hotfix-security-patch

# Bad: 曖昧
wtenv create test
wtenv create temp
wtenv create new-stuff
```

### 3. 定期的に状態確認

```bash
# 毎日の作業開始時
wtenv status

# プロセスの確認
wtenv ps
```

### 4. 不要なプロセスを停止

```bash
# 作業終了時
wtenv kill --all
```

## 通知機能の活用

### 長時間実行コマンドの通知

時間がかかるコマンド（ビルド、テストなど）の完了を通知できます。

```bash
# ビルド完了時に通知
wtenv notify "npm run build"

# テスト完了時に通知
wtenv notify "pnpm test:e2e"

# 特定のworktreeでコマンドを実行して通知
wtenv notify --dir ../myapp-feature-auth "npm test"

# 成功時のみ通知
wtenv notify --notify-error false "npm run deploy"
```

## GitHub PRからworktree作成

GitHub PRから直接worktreeを作成できます（GitHub CLIが必要）。

```bash
# PR #123からworktreeを作成
wtenv pr 123

# カスタムパスを指定
wtenv pr 456 ~/projects/custom-path

# PRからworktreeを作成し、環境ファイルもコピーされ、post-createコマンドも実行される
wtenv pr 789
```

**出力例:**
```
🔍 Fetching PR #123...
✓ Found PR: Add user authentication feature
  Branch: feature-user-auth
  Owner: username
  State: OPEN

📥 Fetching PR branch...
🌲 Creating worktree at /home/user/projects/myapp-feature-user-auth...
✓ Worktree created: /home/user/projects/myapp-feature-user-auth

📋 Copying environment files...
✅ 3個のファイルをコピーしました

📦 post-createコマンドを実行中...

[1/2] Installing dependencies...
  ✓ Installing dependencies... (12.34s)

[2/2] Building packages...
  ✓ Building packages... (8.56s)

✨ post-createコマンドが完了しました

✨ PR #123 worktree is ready!
  cd /home/user/projects/myapp-feature-user-auth
```

## 高度な使用例

### CIパイプラインとの統合

```yaml
# .worktree.yml
postCreate:
  - command: npm install
    description: "Installing dependencies..."

  - command: npm run build
    description: "Building project..."

  - command: npm test
    description: "Running tests..."
    optional: true  # テスト失敗でもworktree作成は続行
```

### 複数データベースのテスト

```yaml
# .worktree.yml
postCreate:
  - command: |
      DB_NAME="test_$(basename $(pwd))"
      createdb $DB_NAME
      echo "DATABASE_URL=postgresql://localhost/$DB_NAME" >> .env.local
    description: "Creating test database..."
```

### 自動ポート割り当て

```yaml
# .worktree.yml
postCreate:
  - command: |
      PORT=$((3000 + $(git rev-list --count HEAD) % 100))
      echo "PORT=$PORT" >> .env.local
      echo "Assigned port: $PORT"
    description: "Assigning unique port..."
```

## まとめ

wtenvは、複数のworktreeを効率的に管理し、並列開発を支援するツールです。

主な利点：
- 🚀 複数の機能を同時に開発
- 📊 一目で全worktreeの状態を把握
- 🔧 プロセスの一元管理
- ⚡ 環境構築の自動化

詳細は[README.md](../README.md)を参照してください。
