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
- **Claude Code連携** 🤖 - すべてのworktreeでClaude Codeのタスク進捗をリアルタイム追跡
  - アクティブなAIコーディングセッションを監視
  - Claudeが応答待ちの時に通知
  - タスクの実行時間とステータスを一目で確認

## インストール

### 必要な環境

- **Rust** 1.91.0以降（ソースからビルドする場合）
- **Python** 3.6以降（Claude Code連携フック用）
- **Git** 2.17以降（worktreeサポート用）
- **GitHub CLI** (`gh`) - オプション、`wtenv pr`コマンドのみで必要

### ソースから

```bash
git clone https://github.com/USERNAME/wtenv.git
cd wtenv
cargo install --path .
```

### バイナリから

[Releases](https://github.com/USERNAME/wtenv/releases)からダウンロードしてPATHに配置。

## セットアップ

### 基本セットアップ

インストール後、リポジトリで設定ファイルを初期化します：

```bash
# gitリポジトリに移動
cd /path/to/your/repo

# wtenv設定を初期化
wtenv init
```

これにより`.worktree.yml`ファイルが作成され、ファイルのコピーやpost-createコマンドを設定できます。

### Claude Code フックセットアップ

リアルタイムタスク追跡のためにClaude Code連携を有効にする方法：

#### 1. フックファイルを生成

```bash
# フックと設定ファイルを生成
wtenv init --hooks
```

以下のファイルが作成されます：
- `.claude/settings.json` - Claude Code フック設定
- `.claude/hooks/session-init.sh` - セッション開始フック（git コンテキスト表示）
- `.claude/hooks/track-progress.py` - タスク進捗追跡フック（Python）
- `~/.claude/stop-hook-git-check.sh` - グローバル停止フック（git 状態チェック）

#### 2. Python インストールを確認

Python 3.6以降が利用可能か確認：

```bash
python3 --version
```

#### 3. フックスクリプトに実行権限を付与

```bash
chmod +x .claude/hooks/session-init.sh
chmod +x .claude/hooks/track-progress.py
chmod +x ~/.claude/stop-hook-git-check.sh
```

#### 4. Claude Code でフックを有効化

**オプションA: プロジェクトレベル（推奨）**

`wtenv init --hooks`を実行すると、このプロジェクトでフックが自動的に有効化されます。

**オプションB: グローバル（すべてのプロジェクト）**

すべてのプロジェクトでフックを有効化する場合：

```bash
# グローバルClaude Code設定にコピー
cp .claude/settings.json ~/.claude/settings.json
```

#### 5. 使用開始

設定完了後、Claude Codeは以下を実行します：
- ✅ セッション開始時に開発コンテキストを表示
- ✅ リアルタイムでタスク進捗を追跡
- ✅ 停止前にgit状態を確認
- ✅ `wtenv ui`でタスクを表示

**動作確認：**

```bash
# リポジトリでClaude Codeセッションを開始
# 別のターミナルで以下を実行：
wtenv ui

# アクティブなClaudeセッションが表示されます！
```

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

インタラクティブなTUIでworktreeを管理。Claude Codeタスクのリアルタイム監視機能付き。

```bash
# TUIを起動
wtenv ui
```

#### インターフェース概要

UIは3つのメインセクションに分かれています：

```
┌─────────────────────────────────────────────────────────────┐
│ Worktrees (3) | Processes (2) | Claude Tasks (1)            │ ← ヘッダー
├─────────────────────────────────────────────────────────────┤
│ > feature-auth     ✓ Clean        Process: npm test         │ ← Worktree一覧
│   bugfix-123       ⚠ Modified     No process                │   （左パネル）
│   feature-payment  🔄 Running     Process: pnpm build        │
├─────────────────────────────────────────────────────────────┤
│ Worktree Details: feature-auth                              │ ← 詳細パネル
│ Branch: main → feature-auth                                 │   （右パネル）
│ Path: /home/user/projects/myapp-feature-auth               │
│ Modified: 0 files | Staged: 0 files                         │
│ Last commit: 5m ago                                         │
│                                                              │
│ Active Processes: 1                                         │
│   PID 12345: npm test (Running for 9m 12s)                  │
│                                                              │
│ Claude Code Tasks:                                          │
│   🔵 feature-auth (In Progress) - 15m 30s                   │
│      Last: Edit(src/auth.rs) - 2s ago                       │
└─────────────────────────────────────────────────────────────┘
Press 'r' to refresh | 'q' to quit                            ← フッター
```

#### キーバインディング

| キー | 操作 |
|------|------|
| `↑/↓` | worktreeを移動（上/下） |
| `j/k` | Vimスタイルのナビゲーション（下/上） |
| `r` | **更新** - worktree、プロセス、Claudeタスクを再読み込み |
| `q` または `Esc` | UIを終了 |
| `Enter` | 選択したworktreeの詳細情報を表示 |

#### Worktreeステータスアイコン

| アイコン | ステータス | 説明 |
|---------|-----------|------|
| ✓ | **Clean** | 変更ファイルなし、すべてコミット済み |
| ⚠ | **Modified** | ファイルが変更されているがコミットされていない |
| 🔄 | **Running** | このworktreeでプロセスが実行中 |
| 📝 | **Staged** | 変更がステージング済み |
| 🔀 | **Ahead** | ローカルのコミットがリモートにプッシュされていない |
| 🔽 | **Behind** | リモートのコミットがローカルにプルされていない |

#### Claude Code タスクステータス

UIはすべてのworktreeでClaude Codeセッションのリアルタイムステータスを表示します：

| ステータス | アイコン | 説明 |
|-----------|---------|------|
| **In Progress** | 🔵 | Claudeが積極的にタスクを実行中 |
| **Stop** | 🟡 | Claudeが停止、ユーザー入力が必要な可能性あり |
| **Session Ended** | ⚫ | セッションが正常に終了 |
| **Error** | 🔴 | タスクでエラーが発生 |

**表示されるタスク情報：**
- Claudeが作業中のworktree/ブランチ名
- 現在のステータスと経過時間
- 最後のアクティビティ（例: "Edit(src/main.rs)"、"Bash(cargo build)"）
- 最後のアクティビティからの経過時間

**自動更新：**
UIは5秒ごとにClaudeタスクステータスを自動更新してリアルタイム情報を表示します。

#### プロセス情報

実行中プロセスがあるworktreeごとに：
- **PID**: プロセスID
- **Command**: 完全なコマンドライン
- **Duration**: プロセスの実行時間
- **Status**: Running、Stopped、Zombie

#### 使い方のヒント

1. **複数のWorktreeを監視**: すべての並列開発ブランチを一目で確認
2. **長時間実行プロセスの追跡**: ビルド、テスト、開発サーバーの監視
3. **Claude Code連携**: Claudeが何をしているか、いつ入力が必要かを把握
4. **クイック更新**: `r`キーでいつでも最新ステータスを取得
5. **キーボードフレンドリー**: すべての操作をキーボードで高速実行

#### Claude Code連携のセットアップ

UIでClaude Codeタスクを表示するには、まずフックをセットアップする必要があります：

```bash
# フックファイルを生成（未実施の場合）
wtenv init --hooks

# フックが実行可能か確認
chmod +x .claude/hooks/track-progress.py

# Claude Codeを使い始める
# タスクが自動的に `wtenv ui` に表示されます
```

詳細なフック設定手順は[セットアップ](#セットアップ)セクションを参照してください。

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

### `wtenv pr`

GitHub PRからworktreeを作成。GitHub CLI (`gh`)が必要です。

```bash
# PR #123からworktreeを作成
wtenv pr 123

# カスタムパスを指定
wtenv pr 456 /path/to/worktree
```

**機能:**
- GitHub CLIを使ってPR情報を自動取得
- リモートブランチを自動フェッチ
- worktreeを自動作成
- 環境ファイルの自動コピー
- post-createコマンドの自動実行

**必要条件:**
- GitHub CLI (`gh`) がインストールされていること
- GitHub CLIで認証済みであること (`gh auth login`)

## グローバルオプション

| オプション | 説明 |
|-----------|------|
| `-v, --verbose` | 詳細出力を有効化 |
| `-q, --quiet` | エラー以外の出力を抑制 |
| `-h, --help` | ヘルプを表示 |
| `-V, --version` | バージョンを表示 |

## ライセンス

MIT
