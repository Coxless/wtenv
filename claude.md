# wtenv - Worktree Environment Manager

高速で依存関係のないgit worktree管理CLIツール。設定ファイルベースで環境ファイルを自動コピーし、開発環境を瞬時にセットアップする。

## 環境情報

### 利用言語
- Rust 2021 edition
- 最小サポートバージョン: 1.92.0

### 利用ライブラリ
- clap 4.4系（CLI パーサー、derive機能使用）
- serde 1.0系（シリアライゼーション）
- serde_yaml 0.9系（YAML パース）
- toml 0.8系（TOML パース）
- glob 0.3系（ファイルパターンマッチング）
- colored 2.1系（カラー出力）
- anyhow 1.0系（エラーハンドリング）
- dialoguer 0.11系（対話モード）
- indicatif 0.17系（プログレス表示）

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
| `/src/main.rs` | エントリーポイント。CLIパーサー定義とサブコマンドのルーティング。clapのderive APIを使用してCLI構造を定義 | - |
| `/src/config.rs` | 設定ファイルの検索・読み込み・初期化。YAMLとTOML両対応。デフォルト設定の提供 | 構造体は`Config`で終わる |
| `/src/worktree.rs` | Git worktree操作のラッパー。`std::process::Command`でgitコマンドを実行 | 関数は動詞始まり（`create_`, `list_`, `remove_`） |
| `/src/copy.rs` | ファイルコピー機能。globパターンマッチング、除外フィルター、ディレクトリ再帰作成 | 関数は動詞始まり |
| `/src/commands.rs` | post-createコマンドの実行。進捗表示とエラーハンドリング | 関数は動詞始まり |
| `/src/interactive.rs` | 対話モード。dialoguerを使用したユーザー入力受付 | 関数は`prompt_`で始まる |

### モジュール分割基準
- **worktree.rs**: Git操作に関する全ての機能（作成・一覧・削除・情報取得）
- **copy.rs**: ファイルシステム操作に関する全ての機能（コピー・パターンマッチ・除外）
- **commands.rs**: 外部コマンド実行に関する全ての機能（実行・進捗・エラー処理）
- **config.rs**: 設定に関する全ての機能（検索・読込・初期化・バリデーション）
- **interactive.rs**: ユーザー対話に関する全ての機能（入力・確認・選択）

## コーディングルール

### 基本方針
- `unwrap()`は使わない。すべて`?`演算子で伝播するか、適切にハンドリング
- `panic!`は使わない。すべて`Result`型で返す
- エラーは`anyhow::Result`で統一。コンテキスト情報を`.context()`で追加
- すべてのpublic関数にドキュメントコメントを記述

### エラーメッセージ
- ユーザー向けメッセージは日本語・英語両対応（英語優先）
- 絵文字を使ってわかりやすく（❌ エラー、✅ 成功、⚠️  警告、📋 情報）
- エラー時は原因と解決策を併記

例:
```rust
anyhow::bail!(
    "❌ Not in main worktree\n\n\
     Please run this command from your main worktree directory.\n\
     Current: {}\n\
     Main: {}",
    current_path.display(),
    main_path.display()
);
```

### 命名規則
- 関数: スネークケース `create_worktree`
- 構造体: パスカルケース `WorktreeConfig`
- 定数: アッパースネークケース `DEFAULT_CONFIG_NAMES`
- モジュール: スネークケース `worktree`

### Git操作
- すべてのgitコマンドは`std::process::Command`で実行
- 標準出力・標準エラーの両方をキャプチャ
- 終了コードを必ずチェック
- タイムアウトは設定しない（ユーザーがCtrl+Cで中断可能）

例:
```rust
let output = Command::new("git")
    .args(["worktree", "add", path, branch])
    .output()
    .context("Failed to execute git command")?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("Git command failed: {}", stderr);
}
```

### ファイルシステム操作
- パスは`PathBuf`と`&Path`を適切に使い分け
- ディレクトリ作成は`create_dir_all`で再帰的に
- ファイルコピーで個別エラーが起きても処理を続行（警告表示）
- シンボリックリンクは通常のファイルとしてコピー

### 設定ファイル
- 検索順: `.worktree.yml` → `.worktree.yaml` → `.worktree.toml` → `worktree.config.yml` → `worktree.config.toml`
- 見つからない場合はデフォルト設定を使用（エラーにしない）
- バージョンフィールドは必須。現在は`version: 1`のみサポート
- 不明なフィールドは無視（将来の拡張性のため）

### カラー出力
- 成功: `colored::green`
- エラー: `colored::red`
- 警告: `colored::yellow`
- 情報: `colored::blue`
- パス: `colored::cyan`
- 詳細: `colored::bright_black`（グレー）

### プログレス表示
- 時間がかかる処理（1秒以上）は必ずスピナーを表示
- スピナーのメッセージは現在の処理内容を明記
- 完了時は✓マークと所要時間を表示

### テスト
- 単体テストは各モジュールの`tests`サブモジュールに配置
- 統合テストは`tests/`ディレクトリに配置
- モックは使わず、実際のファイルシステムで動作検証（テスト用一時ディレクトリ使用）

## 設定ファイル仕様

### サポート形式
YAMLをサポート。

### 必須フィールド
- `version`: 設定ファイルバージョン（現在は1のみ）

### オプションフィールド
- `copy`: コピーするファイルパターンのリスト（glob対応）
- `exclude`: 除外パターンのリスト
- `postCreate`: worktree作成後に実行するコマンドのリスト

### YAML例
```yaml
version: 1

copy:
  - .env
  - .env.local
  - apps/*/.env
  - packages/*/.env.local

exclude:
  - .env.production
  - .env.test

postCreate:
  - command: pnpm install
    description: "Installing dependencies..."
  - command: pnpm build
    description: "Building packages..."
    optional: true
```

## CLI仕様

### サブコマンド構成
```
wtenv
├── create [branch] [path]  # worktree作成
├── list                     # worktree一覧
├── remove <path>            # worktree削除
├── init                     # 設定ファイル初期化
└── config                   # 設定表示
```

### グローバルオプション
- `-h, --help`: ヘルプ表示
- `-V, --version`: バージョン表示

### createオプション
- `-v, --verbose`: 詳細出力
- `--no-copy`: ファイルコピーをスキップ
- `--no-post-create`: post-createコマンドをスキップ
- `-c, --config <PATH>`: 設定ファイルパス指定

### removeオプション
- `-f, --force`: 強制削除

### initオプション
- `-f, --force`: 既存設定を上書き

## パフォーマンス目標

- 起動時間: < 50ms
- worktree作成（10ファイルコピー）: < 500ms
- メモリ使用量: < 10MB
- バイナリサイズ: < 5MB（strip後）

## セキュリティ考慮事項

- `.env*`ファイルが誤ってgitに含まれないよう、デフォルトの`.gitignore`例を提供
- ユーザー入力（パス、ブランチ名）のバリデーション実施
- コマンドインジェクション対策（gitコマンドの引数は配列で渡す）
- シンボリックリンク攻撃対策（リンク先の検証）

## 配布戦略

### プライマリ: GitHub Releases
- タグプッシュで自動ビルド（GitHub Actions）
- macOS（Intel/ARM）、Linux（x64）、Windows（x64）のバイナリ提供
- バイナリ名: `wtenv-{version}-{os}-{arch}`
