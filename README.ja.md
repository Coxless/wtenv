# wtenv - Git Worktree環境マネージャー

> **Warning**
> このツールは開発中であり、安定版ではありません。使用する際は慎重に行ってください。

高速でユーザーフレンドリーなgit worktree管理CLIツール。**並列開発のコントロールセンター**機能を搭載。

## 機能

### コアworktree管理機能
- ブランチ管理を含む簡単なworktree作成
- 環境ファイルの自動コピー（設定ベース）
- post-createコマンドの実行
- 対話モード（引数なしで実行可能）
- プログレス表示とカラー出力
- 詳細/サイレント出力モード

### **NEW: 並列開発コントロールセンター** 🚀
- **リアルタイムworktree状態監視** - すべてのworktreeの状態を一目で確認
- **プロセス管理** - 各worktreeで実行中のプロセスを追跡・管理
- **プロセス制御** - PID、worktree、または一括でプロセスを停止
- **永続的なプロセス追跡** - ターミナルセッションを超えてプロセス情報を保持

## インストール

### ソースから

```bash
git clone https://github.com/USERNAME/wtenv.git
cd wtenv
cargo install --path .
```

### バイナリから

[Releases](https://github.com/USERNAME/wtenv/releases)からダウンロードしてPATHに配置。

## クイックスタート

```bash
# 設定ファイル初期化
wtenv init

# worktree作成（対話モード）
wtenv create

# ブランチ名を指定してworktree作成
wtenv create feature-branch

# worktree一覧
wtenv list

# worktree削除
wtenv remove ../feature-branch
```

## 設定

リポジトリルートに`.worktree.yml`を作成:

```yaml
version: 1

copy:
  - .env
  - .env.local
  - config/*.local.json

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."
  - command: npm run build
    description: "プロジェクトをビルド中..."
    optional: true
```

### 設定オプション

| フィールド | 説明 |
|-----------|------|
| `version` | 設定ファイルバージョン（現在: 1） |
| `copy` | コピーするファイルのglobパターン |
| `exclude` | 除外するファイルのglobパターン |
| `postCreate` | worktree作成後に実行するコマンド |

### post-createコマンドオプション

| フィールド | 説明 |
|-----------|------|
| `command` | 実行するシェルコマンド |
| `description` | 実行中に表示される説明 |
| `optional` | trueの場合、失敗しても続行 |

## コマンド

### 監視・制御コマンド

#### `wtenv status`

すべてのworktreeの詳細な状態とプロセス情報を表示。

```bash
# worktree概要を表示
wtenv status

# 詳細モード（フルパスを表示）
wtenv status --verbose
```

**出力例:**
```
┌─────────────────────────────────────────────────────────────┐
│ Worktrees Overview (3 active, 2 processes)                  │
├─────────────────────────────────────────────────────────────┤
│ 🔄 feature-a                      main → feature-a          │
│    Status: Modified (3 files)     Process: pnpm test        │
│    Modified: 3 files  |  Last commit: 2h ago                │
│                                                              │
│ 🔨 feature-b                      main → feature-b          │
│    Status: Running                Process: pnpm build       │
│    Modified: 1 file   |  Last commit: 30m ago               │
│                                                              │
│ ✅ bugfix-123                     main → bugfix-123         │
│    Status: Clean                  No process                │
│    Last commit: 5m ago                                      │
├─────────────────────────────────────────────────────────────┤
│ 📊 Total: 3 worktrees  |  Modified: 4 files                │
└─────────────────────────────────────────────────────────────┘
```

#### `wtenv ps [FILTER]`

worktreeで実行中のすべてのプロセスを一覧表示。

```bash
# すべてのプロセスを表示
wtenv ps

# worktree/ブランチ名でフィルタ
wtenv ps feature-a
```

**出力例:**
```
Active Processes in Worktrees:

feature-a (PID: 12345)
  Command: pnpm test:e2e
  Started: 9m 12s ago
  Working Dir: /home/user/projects/myapp-feature-a
  Status: Running

Total: 1 process
```

#### `wtenv kill [OPTIONS]`

実行中のプロセスを停止。

```bash
# 特定のPIDを停止
wtenv kill 12345

# すべてのプロセスを停止
wtenv kill --all

# 特定のworktreeのプロセスを停止
wtenv kill feature-a
```

### Worktree管理コマンド

#### `wtenv create [BRANCH] [PATH]`

新しいworktreeを作成。

```bash
# 対話モード
wtenv create

# ブランチ指定（パスは../branch-nameがデフォルト）
wtenv create feature-auth

# ブランチとパスを指定
wtenv create feature-auth ~/projects/feature-auth

# ファイルコピーをスキップ
wtenv create feature-auth --no-copy

# post-createコマンドをスキップ
wtenv create feature-auth --no-post-create
```

### `wtenv list`

すべてのworktreeを一覧表示。

```bash
wtenv list

# 詳細モード（完全なコミットハッシュを表示）
wtenv list --verbose
```

### `wtenv remove <PATH>`

worktreeを削除。

```bash
# 対話的に確認
wtenv remove ../feature-branch

# 強制削除（確認なし）
wtenv remove ../feature-branch --force
```

### `wtenv init`

設定ファイルを初期化。

```bash
wtenv init

# 既存の設定を上書き
wtenv init --force
```

### `wtenv config`

現在の設定を表示。

```bash
wtenv config

# 詳細情報を表示
wtenv config --verbose
```

### `wtenv diff-env`

worktree間の環境変数の違いを表示。

```bash
# 2つのworktree間の環境変数を比較
wtenv diff-env feature-a feature-b

# すべてのworktreeの環境変数を比較
wtenv diff-env --all
```

**出力例:**
```
🔍 feature-a と feature-b の環境変数の違い:

.env:
  API_PORT:
    - 3001
    + 3002
  DATABASE_URL:
    - postgresql://localhost/auth_db
    + postgresql://localhost/payment_db

.env.local:
  DEBUG (feature-aのみ)
    - true
```

### `wtenv ui`

インタラクティブなTUIでworktreeを管理。

```bash
# TUIを起動
wtenv ui
```

**キー操作:**
- `↑/↓` または `j/k`: worktree選択
- `r`: 状態を更新
- `q` または `Esc`: 終了

**機能:**
- すべてのworktreeを一覧表示
- 選択したworktreeの詳細情報を表示
- 実行中プロセス数をリアルタイム表示
- キーボードナビゲーション

### `wtenv analyze`

worktreeの状態を分析し、ディスク使用量や依存関係の状態を表示。

```bash
# worktreeを分析
wtenv analyze

# 詳細情報を表示
wtenv analyze --detailed
```

**出力例:**
```
📊 Worktree Analysis

  feature-auth
    Disk: 12.45 MB
    Last update: 2 days ago
    Tags: node_modules, lockfile, build

  feature-payment
    Disk: 8.32 MB
    Last update: Yesterday
    Tags: node_modules, lockfile, merged

Summary
  Total worktrees: 3
  Total disk usage: 35.12 MB
  Merged branches: 1
  Stale (>30 days): 0
```

### `wtenv clean`

マージ済みまたは長期間更新されていないworktreeを削除。

```bash
# ドライラン（削除候補を表示）
wtenv clean --dry-run

# マージ済みブランチのみ削除
wtenv clean --merged-only

# 30日以上更新されていないworktreeを削除
wtenv clean --stale-days 30

# 確認なしで削除
wtenv clean --force
```

### `wtenv notify`

コマンドを実行し、完了時にデスクトップ通知を送信。

```bash
# ビルドコマンドを実行して通知
wtenv notify "npm run build"

# 指定ディレクトリでコマンドを実行
wtenv notify --dir ./worktrees/feature-a "npm test"

# 成功時のみ通知
wtenv notify --notify-error false "npm run deploy"
```

## グローバルオプション

| オプション | 説明 |
|-----------|------|
| `-v, --verbose` | 詳細出力を有効化 |
| `-q, --quiet` | エラー以外の出力を抑制 |
| `-h, --help` | ヘルプを表示 |
| `-V, --version` | バージョンを表示 |

## ライセンス

MIT
