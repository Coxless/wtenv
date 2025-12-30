# 基本的な使い方

wtenvの基本的な使い方を説明します。

## セットアップ

### 1. 設定ファイルの初期化

リポジトリのルートディレクトリで以下のコマンドを実行します:

```bash
wtenv init
```

これにより `.worktree.yml` が作成されます:

```yaml
version: 1

copy:
  - .env
  - .env.local

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "Installing dependencies..."
```

### 2. 設定ファイルの編集

プロジェクトに合わせて設定を編集します:

```yaml
version: 1

copy:
  - .env              # メインの環境変数ファイル
  - .env.local        # ローカル設定
  - config/local.json # ローカル設定ファイル

exclude:
  - .env.production   # 本番用はコピーしない
  - .env.staging      # ステージング用もコピーしない

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."
  - command: npm run build
    description: "ビルド中..."
    optional: true    # 失敗しても続行
```

## Worktree操作

### 新しいworktreeを作成

```bash
# 基本的な作成（パスは自動で ../feature-auth になる）
wtenv create feature-auth

# パスを指定して作成
wtenv create feature-auth ../my-feature

# ファイルコピーをスキップ
wtenv create feature-auth --no-copy

# post-createコマンドをスキップ
wtenv create feature-auth --no-post-create

# 対話モード（引数なし）
wtenv create
```

### worktree一覧を表示

```bash
wtenv list
```

出力例:
```
📁 /home/user/project (main)           [main] abc1234
📁 /home/user/feature-auth             [feature-auth] def5678
📁 /home/user/bugfix-login             [bugfix-login] ghi9012
```

### worktreeを削除

```bash
# 確認ダイアログ付きで削除
wtenv remove ../feature-auth

# 強制削除（確認なし）
wtenv remove ../feature-auth --force
```

### 現在の設定を確認

```bash
wtenv config
```

## 典型的なワークフロー

### 1. 新機能の開発開始

```bash
# メインブランチにいることを確認
git checkout main
git pull

# 新しいworktreeを作成
wtenv create feature-user-profile

# 作成されたworktreeに移動
cd ../feature-user-profile

# 開発開始
code .
```

### 2. バグ修正の並行作業

```bash
# 別のworktreeでバグ修正
wtenv create hotfix-login-bug

# 移動して作業
cd ../hotfix-login-bug

# 修正、コミット、プッシュ
git add .
git commit -m "fix: ログインバグを修正"
git push origin hotfix-login-bug
```

### 3. 作業完了後のクリーンアップ

```bash
# メインに戻る
cd /path/to/main-worktree

# マージ完了したworktreeを削除
wtenv remove ../feature-user-profile
wtenv remove ../hotfix-login-bug
```

## オプションフラグ

| フラグ | 説明 |
|--------|------|
| `--verbose`, `-v` | 詳細な出力を表示 |
| `--quiet`, `-q` | エラー以外の出力を抑制 |
| `--no-copy` | ファイルコピーをスキップ |
| `--no-post-create` | post-createコマンドをスキップ |
| `--force`, `-f` | 確認なしで実行 |
| `--config`, `-c` | 設定ファイルを指定 |

## ヒント

### 1. エイリアスの設定

頻繁に使うコマンドにはエイリアスを設定すると便利です:

```bash
# ~/.bashrc または ~/.zshrc
alias wc='wtenv create'
alias wl='wtenv list'
alias wr='wtenv remove'
```

### 2. ブランチ命名規則

worktreeのディレクトリ名はブランチ名から自動生成されるため、
ブランチ名にはファイルシステムで有効な文字を使用してください:

```bash
# 良い例
wtenv create feature-user-auth
wtenv create fix-123

# 避けるべき例（スラッシュを含む）
wtenv create feature/user-auth  # パスが ../feature/user-auth になる
```

### 3. 設定ファイルの共有

`.worktree.yml` はリポジトリにコミットして、
チーム全員で同じ設定を共有することをお勧めします。
