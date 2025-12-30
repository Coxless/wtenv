# wtenv - Git Worktree環境マネージャー

> **Warning**
> このツールは開発中であり、安定版ではありません。使用する際は慎重に行ってください。

高速でユーザーフレンドリーなgit worktree管理CLIツール。

## 機能

- ブランチ管理を含む簡単なworktree作成
- 環境ファイルの自動コピー（設定ベース）
- post-createコマンドの実行
- 対話モード（引数なしで実行可能）
- プログレス表示とカラー出力
- 詳細/サイレント出力モード

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

### `wtenv create [BRANCH] [PATH]`

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

## グローバルオプション

| オプション | 説明 |
|-----------|------|
| `-v, --verbose` | 詳細出力を有効化 |
| `-q, --quiet` | エラー以外の出力を抑制 |
| `-h, --help` | ヘルプを表示 |
| `-V, --version` | バージョンを表示 |

## ライセンス

MIT
