# 高度な設定例

wtenvの高度な設定パターンを紹介します。

## Globパターンの活用

### 複数のファイルをコピー

```yaml
version: 1

copy:
  # 特定のファイル
  - .env
  - .env.local

  # ディレクトリ内のすべてのファイル
  - config/*.json

  # 再帰的にマッチ
  - secrets/**/*.pem

  # 複数の拡張子
  - "*.local.{json,yaml}"
```

### 除外パターン

```yaml
version: 1

copy:
  - .env*
  - config/**/*

exclude:
  # 本番環境用ファイルを除外
  - .env.production
  - .env.staging

  # テスト用ファイルを除外
  - config/**/test-*.json

  # バックアップファイルを除外
  - "**/*.bak"
  - "**/*~"
```

## Post-Createコマンド

### 複数コマンドの実行

```yaml
version: 1

postCreate:
  # 依存関係のインストール
  - command: npm install
    description: "npm依存関係をインストール中..."

  # データベースのセットアップ
  - command: npm run db:migrate
    description: "データベースマイグレーション中..."
    optional: true

  # シードデータの投入
  - command: npm run db:seed
    description: "シードデータを投入中..."
    optional: true

  # ビルド
  - command: npm run build
    description: "ビルド中..."
```

### 条件付き実行（optional）

```yaml
postCreate:
  # 必須：これが失敗すると中断
  - command: npm install
    description: "依存関係をインストール中..."

  # オプション：失敗しても続行
  - command: npm run setup:optional
    description: "オプションセットアップ中..."
    optional: true

  # 必須：上記が成功しても失敗しても実行される
  - command: npm run validate
    description: "検証中..."
```

## 言語/フレームワーク別設定例

### Node.js / npm プロジェクト

```yaml
version: 1

copy:
  - .env
  - .env.local
  - .npmrc.local

exclude:
  - .env.production
  - .env.ci

postCreate:
  - command: npm install
    description: "npm install を実行中..."
  - command: npm run prepare
    description: "husky をセットアップ中..."
    optional: true
```

### Python / Poetry プロジェクト

```yaml
version: 1

copy:
  - .env
  - .env.local
  - config/local.yaml

exclude:
  - .env.production

postCreate:
  - command: poetry install
    description: "Poetry依存関係をインストール中..."
  - command: poetry run python manage.py migrate
    description: "データベースマイグレーション中..."
    optional: true
```

### Rust / Cargo プロジェクト

```yaml
version: 1

copy:
  - .env
  - config.local.toml

postCreate:
  - command: cargo build
    description: "ビルド中..."
  - command: cargo test --no-run
    description: "テストをコンパイル中..."
    optional: true
```

### Go プロジェクト

```yaml
version: 1

copy:
  - .env
  - config/local.yaml

postCreate:
  - command: go mod download
    description: "依存関係をダウンロード中..."
  - command: go build ./...
    description: "ビルド中..."
```

### Docker Compose プロジェクト

```yaml
version: 1

copy:
  - .env
  - docker-compose.override.yml

exclude:
  - .env.production

postCreate:
  - command: docker-compose pull
    description: "Dockerイメージをプル中..."
    optional: true
  - command: docker-compose build
    description: "Dockerイメージをビルド中..."
    optional: true
```

## 複雑な設定例

### フルスタックアプリケーション

```yaml
version: 1

copy:
  # フロントエンド
  - frontend/.env
  - frontend/.env.local

  # バックエンド
  - backend/.env
  - backend/.env.local
  - backend/config/local.json

  # 共通
  - .env
  - docker-compose.override.yml

exclude:
  - "**/.env.production"
  - "**/.env.staging"
  - "**/secrets/**"

postCreate:
  # フロントエンド
  - command: cd frontend && npm install
    description: "フロントエンド依存関係をインストール中..."

  # バックエンド
  - command: cd backend && npm install
    description: "バックエンド依存関係をインストール中..."

  # Docker
  - command: docker-compose up -d db redis
    description: "データベースを起動中..."
    optional: true

  # マイグレーション
  - command: cd backend && npm run db:migrate
    description: "データベースマイグレーション中..."
    optional: true
```

### マイクロサービス

```yaml
version: 1

copy:
  - .env
  - services/*/.env.local
  - shared/config/local.yaml

exclude:
  - "**/.env.production"
  - "**/credentials/**"

postCreate:
  # 依存関係を並列でインストール（make使用）
  - command: make install-all
    description: "すべての依存関係をインストール中..."

  # インフラ起動
  - command: docker-compose -f docker-compose.dev.yml up -d
    description: "開発インフラを起動中..."
    optional: true
```

## ヒント

### 1. 設定ファイルのテスト

新しい設定を試す前に、`--no-post-create` で動作確認できます:

```bash
# ファイルコピーのみテスト
wtenv create test-branch --no-post-create

# 確認後に削除
wtenv remove ../test-branch --force
```

### 2. 詳細ログの確認

問題が発生した場合は `--verbose` で詳細を確認:

```bash
wtenv create feature-x --verbose
```

### 3. 環境固有の設定

本番環境のシークレットは絶対にコピーしないよう注意:

```yaml
exclude:
  - .env.production
  - .env.prod
  - "**/production/**"
  - "**/secrets/**"
  - "**/*.pem"
  - "**/*.key"
```
