# モノレポでの使用例

モノレポ（Monorepo）環境でwtenvを活用する方法を説明します。

## 典型的なモノレポ構造

```
my-monorepo/
├── .worktree.yml
├── .env
├── package.json
├── pnpm-workspace.yaml
├── packages/
│   ├── web/
│   │   ├── .env.local
│   │   └── package.json
│   ├── api/
│   │   ├── .env.local
│   │   └── package.json
│   └── shared/
│       └── package.json
└── apps/
    ├── admin/
    │   ├── .env.local
    │   └── package.json
    └── mobile/
        └── package.json
```

## 基本設定

### pnpm / npm workspaces

```yaml
version: 1

copy:
  # ルートの環境変数
  - .env
  - .env.local

  # 各パッケージの環境変数
  - packages/*/.env.local
  - apps/*/.env.local

exclude:
  - "**/.env.production"
  - "**/.env.staging"

postCreate:
  # pnpmでワークスペース全体をインストール
  - command: pnpm install
    description: "pnpm install を実行中..."

  # 共通パッケージをビルド
  - command: pnpm -F @myorg/shared build
    description: "共有パッケージをビルド中..."
    optional: true
```

### Turborepo

```yaml
version: 1

copy:
  - .env
  - .env.local
  - apps/*/.env.local
  - packages/*/.env.local

exclude:
  - "**/.env.production"

postCreate:
  - command: pnpm install
    description: "依存関係をインストール中..."

  - command: pnpm turbo build --filter=@myorg/shared
    description: "共有パッケージをビルド中..."
    optional: true
```

### Nx

```yaml
version: 1

copy:
  - .env
  - .env.local
  - apps/*/.env.local
  - libs/*/.env.local

exclude:
  - "**/.env.production"
  - "**/.env.ci"

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."

  - command: npx nx run-many --target=build --projects=shared-*
    description: "共有ライブラリをビルド中..."
    optional: true
```

### Lerna

```yaml
version: 1

copy:
  - .env
  - .env.local
  - packages/*/.env.local

exclude:
  - "**/.env.production"

postCreate:
  - command: npm install
    description: "ルート依存関係をインストール中..."

  - command: npx lerna bootstrap
    description: "Lernaブートストラップ中..."

  - command: npx lerna run build --scope=@myorg/shared
    description: "共有パッケージをビルド中..."
    optional: true
```

## 複雑なモノレポ設定

### フロントエンド + バックエンド + インフラ

```yaml
version: 1

copy:
  # 共通
  - .env
  - .env.local
  - docker-compose.override.yml

  # フロントエンドアプリ
  - apps/web/.env.local
  - apps/admin/.env.local
  - apps/mobile/.env.local

  # バックエンドサービス
  - services/api/.env.local
  - services/auth/.env.local
  - services/worker/.env.local

  # 共有パッケージ
  - packages/config/.env.local

exclude:
  - "**/.env.production"
  - "**/.env.staging"
  - "**/secrets/**"
  - "**/*.pem"

postCreate:
  # 依存関係
  - command: pnpm install
    description: "pnpm install を実行中..."

  # Docker（データベース等）
  - command: docker-compose up -d postgres redis
    description: "データベースを起動中..."
    optional: true

  # マイグレーション
  - command: pnpm -F @myorg/api db:migrate
    description: "データベースマイグレーション中..."
    optional: true

  # 共有パッケージのビルド
  - command: pnpm turbo build --filter=@myorg/shared --filter=@myorg/ui
    description: "共有パッケージをビルド中..."
    optional: true
```

### Go + TypeScript ハイブリッド

```yaml
version: 1

copy:
  - .env
  - .env.local

  # Go サービス
  - services/*/config/local.yaml

  # TypeScript アプリ
  - apps/*/.env.local

exclude:
  - "**/config/production.yaml"
  - "**/.env.production"

postCreate:
  # Go 依存関係
  - command: cd services && go mod download
    description: "Go依存関係をダウンロード中..."

  # TypeScript 依存関係
  - command: pnpm install
    description: "pnpm install を実行中..."

  # ビルド
  - command: make build-shared
    description: "共有コードをビルド中..."
    optional: true
```

## ワークフロー例

### 特定のパッケージで作業

```bash
# 機能ブランチを作成
wtenv create feature-new-ui

# worktreeに移動
cd ../feature-new-ui

# 特定のパッケージのみ開発サーバー起動
pnpm -F @myorg/web dev

# または Turborepo で
pnpm turbo dev --filter=@myorg/web
```

### 複数パッケージにまたがる変更

```bash
# ブランチ作成
wtenv create feature-api-refactor

cd ../feature-api-refactor

# 関連パッケージをウォッチモードでビルド
pnpm turbo build --filter=@myorg/shared --filter=@myorg/api-client --watch
```

### 緊急バグ修正

```bash
# 現在の作業を中断せずに別worktreeで修正
wtenv create hotfix-auth-bug

cd ../hotfix-auth-bug

# 修正してプッシュ
git add .
git commit -m "fix: 認証バグを修正"
git push origin hotfix-auth-bug

# 元の作業に戻る
cd ../feature-new-ui

# 完了後にクリーンアップ
wtenv remove ../hotfix-auth-bug
```

## ヒント

### 1. パッケージ固有の設定

各パッケージに `.env.local.example` を用意しておくと便利です:

```bash
# 初回セットアップ時にコピー
cp packages/api/.env.local.example packages/api/.env.local
```

### 2. 大規模モノレポでのパフォーマンス

多数のファイルをコピーする場合、`--verbose` で進捗を確認:

```bash
wtenv create feature-x --verbose
```

### 3. 部分的なセットアップ

全パッケージの依存関係が不要な場合:

```bash
# post-createをスキップして手動で必要なものだけインストール
wtenv create feature-web --no-post-create

cd ../feature-web
pnpm install --filter=@myorg/web...
```

### 4. Docker環境との連携

worktreeごとに別のDockerネットワークを使用:

```yaml
postCreate:
  - command: docker-compose -p wtenv-$(basename $(pwd)) up -d
    description: "Docker環境を起動中..."
    optional: true
```
