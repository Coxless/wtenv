# ccmon - Claude Code Monitor

Claude Code の並列開発セッションをリアルタイムで監視するツール。

## 機能

- **Claude Code タスクのリアルタイム監視** - 複数の Claude Code セッションを同時に追跡
- **インタラクティブ TUI** - タスク状態、経過時間、最後のアクティビティを表示
- **デスクトップ通知** - コマンド完了時に通知
- **自動更新** - UI が1秒ごとに自動更新
- **Claude Code hooks** - hooks による自動タスク進捗追跡

### タスクステータス表示

| ステータス | アイコン | 説明 |
|-----------|---------|------|
| **In Progress** | 🔵 | Claude がアクティブに作業中 |
| **Stop** | 🟡 | レスポンス完了、ユーザー待ち |
| **Session Ended** | ⚫ | セッション終了 |
| **Error** | 🔴 | タスクでエラー発生 |

## インストール

### 必要な環境

- **Rust** 1.91.0 以降（ソースからビルドする場合）
- **Python** 3.6 以降（Claude Code hooks 用）

### ソースから

```bash
git clone https://github.com/Coxless/ccmon.git
cd ccmon
cargo install --path .
```

### バイナリから

[Releases](https://github.com/Coxless/ccmon/releases) からダウンロードして PATH に配置。

## クイックスタート

```bash
# Claude Code hooks を初期化
ccmon init

# インタラクティブ TUI を起動
ccmon ui

# デスクトップ通知付きでコマンド実行
ccmon notify "cargo build"
```

## セットアップ

### hooks の初期化

インストール後、リポジトリで Claude Code hooks を初期化：

```bash
cd /path/to/your/repo
ccmon init
```

以下のファイルが作成されます：
- `.claude/settings.json` - Claude Code hook 設定
- `.claude/hooks/session-init.sh` - セッション開始 hook（git コンテキスト表示）
- `.claude/hooks/track-progress.py` - タスク進捗追跡 hook
- `~/.claude/stop-hook-git-check.sh` - グローバル stop hook（git 状態チェック）

### hooks をグローバルに有効化

すべてのプロジェクトで hooks を有効化：

```bash
cp .claude/settings.json ~/.claude/settings.json
```

### セットアップの確認

```bash
# リポジトリで Claude Code セッションを開始
# 別のターミナルで以下を実行：
ccmon ui

# アクティブな Claude セッションが表示されます！
```

## コマンド

### `ccmon init`

カレントディレクトリに Claude Code hooks を初期化。

```bash
ccmon init          # hooks を作成
ccmon init --force  # 既存の hooks を上書き
```

### `ccmon ui`

Claude Code タスク監視用のインタラクティブ TUI を起動。

```bash
ccmon ui
```

#### キーバインド

| キー | 操作 |
|------|------|
| `j` / `↓` | 次のタスクへ移動 |
| `k` / `↑` | 前のタスクへ移動 |
| `r` | 手動更新 |
| `q` / `Esc` | 終了 |

#### 表示情報

- セッション ID
- 作業ディレクトリ
- 現在のステータスと経過時間
- 最後のアクティビティ（使用ツール、編集ファイルなど）

### `ccmon notify <command>`

デスクトップ通知付きでコマンドを実行。

```bash
# ビルドを通知付きで実行
ccmon notify "cargo build --release"

# テストを通知付きで実行
ccmon notify "cargo test"

# 特定のディレクトリで実行
ccmon notify --dir ./project "npm test"

# 通知の制御
ccmon notify --notify-success=false "npm run lint"  # 成功時の通知なし
ccmon notify --notify-error=false "make check"      # エラー時の通知なし
```

## グローバルオプション

| オプション | 説明 |
|-----------|------|
| `-v, --verbose` | 詳細出力を有効化 |
| `-q, --quiet` | エラー以外の出力を抑制 |
| `-h, --help` | ヘルプを表示 |
| `-V, --version` | バージョンを表示 |

## 仕組み

1. `ccmon init` が hook スクリプトを作成。Claude Code が各ポイントで実行：
   - **SessionStart**: セッション開始を記録、git コンテキストを表示
   - **UserPromptSubmit**: ユーザープロンプト送信時に in_progress に設定
   - **PostToolUse**: ツール使用を追跡（Edit, Bash, Read など）
   - **Stop**: Claude がユーザー入力待ちの時を記録
   - **SessionEnd**: セッション完了を記録

2. hook イベントは `~/.claude/task-progress/<session_id>.jsonl` に書き込まれる

3. `ccmon ui` がこれらのファイルを読み取り、リアルタイムでステータスを表示

## ライセンス

MIT
