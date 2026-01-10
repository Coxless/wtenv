# ccmon - Claude Code Monitor

Claude Code の並列開発タスク進捗をリアルタイムで表示する CLI ツール。複数の Claude Code セッションを同時に監視し、TUI でステータスを確認できる。

## 環境情報

### 利用言語
- Rust 2021 edition
- 最小サポートバージョン: 1.91.0

### 利用ライブラリ
- clap 4.4系（CLI パーサー、derive機能使用）
- serde 1.0系（シリアライゼーション）
- serde_json 1.0系（JSON パース、タスク進捗読み込み用）
- colored 2.1系（カラー出力）
- anyhow 1.0系（エラーハンドリング）
- indicatif 0.17系（プログレス表示）
- chrono 0.4系（日時処理、serde機能有効）
- ratatui 0.29系（TUI構築）
- crossterm 0.28系（ターミナル制御）
- notify-rust 4.11系（デスクトップ通知）
- dirs 5.0系（ホームディレクトリ解決）

### ビルド設定
```toml
[profile.release]
opt-level = "z"      # サイズ最適化
lto = true           # リンク時最適化
codegen-units = 1    # 最適化優先
strip = true         # シンボル削除
```

## ディレクトリ構成

| Path | 用途 | 命名規則 |
|------|------|----------|
| `/src/main.rs` | エントリーポイント。CLIパーサー定義とサブコマンドのルーティング | - |
| `/src/config.rs` | Claude Code hooks テンプレートの作成 | - |
| `/src/output.rs` | 出力フォーマット機能。プログレスバー作成 | - |
| `/src/errors.rs` | エラーフォーマット機能 | - |
| `/src/commands/mod.rs` | コマンドモジュールの re-export | - |
| `/src/commands/claude_task.rs` | Claude Code タスク進捗追跡。データ構造とJSONL読み込み | - |
| `/src/commands/ui.rs` | インタラクティブTUI。Claude Code タスク進捗表示 | - |
| `/src/commands/notify.rs` | デスクトップ通知コマンド | - |

## コーディングルール

### 基本方針
- `unwrap()`は使わない。すべて`?`演算子で伝播するか、適切にハンドリング
- `panic!`は使わない。すべて`Result`型で返す
- エラーは`anyhow::Result`で統一。コンテキスト情報を`.context()`で追加
- すべてのpublic関数にドキュメントコメントを記述

### エラーメッセージ
- ユーザー向けメッセージは英語
- 絵文字を使ってわかりやすく（❌ エラー、✅ 成功、⚠️  警告、📋 情報）
- エラー時は原因と解決策を併記

### 命名規則
- 関数: スネークケース `create_hooks`
- 構造体: パスカルケース `TaskManager`
- 定数: アッパースネークケース `CLAUDE_SETTINGS_TEMPLATE`
- モジュール: スネークケース `claude_task`

### カラー出力
- 成功: `colored::green`
- エラー: `colored::red`
- 警告: `colored::yellow`
- 情報: `colored::blue`
- パス: `colored::cyan`
- 詳細: `colored::bright_black`（グレー）

## CLI仕様

### サブコマンド構成
```
ccmon
├── init                # Claude Code hooks を初期化
│   └── --force         # 既存設定を上書き
├── ui                  # インタラクティブTUI（タスク進捗表示）
└── notify <command>    # コマンド実行とデスクトップ通知
    ├── -d, --dir       # 作業ディレクトリ
    ├── --notify-success # 成功時に通知（デフォルト: true）
    └── --notify-error   # エラー時に通知（デフォルト: true）
```

### グローバルオプション
- `-h, --help`: ヘルプ表示
- `-V, --version`: バージョン表示
- `-v, --verbose`: 詳細出力モード
- `-q, --quiet`: サイレントモード（エラー以外の出力を抑制）

## Claude Code 連携

### hooks 初期化

`ccmon init` コマンドで、Claude Code の hooks ファイルを自動生成する。

#### 作成されるファイル

1. **`.claude/settings.json`** - Claude Code hooks 設定
   - SessionStart hook: セッション開始時に開発コンテキストを表示
   - Stop hook: タスク完了時に git の状態をチェック

2. **`.claude/hooks/session-init.sh`** - SessionStart hook スクリプト
   - 現在のブランチと worktree 情報を表示
   - 最近のコミット履歴を表示
   - 未コミット/ステージング済み/未追跡ファイルの状態を表示

3. **`.claude/hooks/track-progress.py`** - タスク進捗追跡 hook (Python)
   - Claude Code のタスク進捗を `~/.claude/task-progress/<session_id>.jsonl` に記録
   - `ccmon ui` コマンドでリアルタイム進捗表示が可能
   - SessionStart, PostToolUse, Stop, SessionEnd イベントを追跡

4. **`~/.claude/stop-hook-git-check.sh`** - グローバル Stop hook
   - 未コミットの変更がないかチェック
   - 未追跡ファイルがないかチェック
   - リモートにプッシュされていないコミットがないかチェック
   - 問題がある場合は exit 2 で Claude に通知

#### hooks の有効化

```bash
# プロジェクトレベルで有効化
ccmon init

# グローバルで有効化（全プロジェクトに適用）
cp .claude/settings.json ~/.claude/settings.json
```

### Claude Code タスク進捗表示

`ccmon ui` コマンドで、Claude Code のタスク進捗をリアルタイムで確認できる:

- アクティブな Claude Code セッション一覧
- 各セッションの現在の状態（in_progress, stop, session_ended, error）
- 最後のアクティビティ（ツール実行、ファイル編集など）
- タイムスタンプと経過時間

#### TUI キーバインド

| キー | 動作 |
|------|------|
| `j` / `↓` | 次のタスクへ移動 |
| `k` / `↑` | 前のタスクへ移動 |
| `r` | タスクリストを更新 |
| `q` / `Esc` | 終了 |

### デスクトップ通知

`ccmon notify <command>` で、コマンド実行完了時にデスクトップ通知を送信:

```bash
# ビルド完了を通知
ccmon notify "cargo build --release"

# テスト完了を通知
ccmon notify "cargo test"

# 成功時のみ通知
ccmon notify --notify-error=false "npm run lint"
```

## パフォーマンス目標

- 起動時間: < 50ms
- TUI更新: < 100ms
- メモリ使用量: < 10MB
- バイナリサイズ: < 5MB（strip後）

## 配布戦略

### プライマリ: GitHub Releases
- タグプッシュで自動ビルド（GitHub Actions）
- macOS（Intel/ARM）、Linux（x64）、Windows（x64）のバイナリ提供
- バイナリ名: `ccmon-{version}-{os}-{arch}`
