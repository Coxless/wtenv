# wtenv - Rust実装プラン

## 概要

git worktree管理CLIツール「wtenv」の段階的実装プラン。
設定ファイルに基づいてworktree作成、環境ファイルコピー、post-createコマンド実行を自動化する。

## 決定事項

- **設定ファイル形式**: YAMLのみ（.worktree.yml, .worktree.yaml）
- **実装スコープ**: Phase 1-6（コア機能のみ、対話モードなし）
- **メッセージ言語**: 日本語のみ

---

## Phase 1: プロジェクト基盤（必須）

### Task 1.1: Cargo.toml作成

```toml
[package]
name = "wtenv"
version = "0.1.0"
edition = "2021"
rust-version = "1.92.0"
description = "Git worktree environment manager"
license = "MIT"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
glob = "0.3"
colored = "2.1"
anyhow = "1.0"
indicatif = "0.17"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

**注**: dialoguer（対話モード用）とtoml（TOML対応用）はスコープ外のため除外

### Task 1.2: src/main.rs - 基本構造

```rust
mod config;
mod worktree;
mod copy;
mod commands;

fn main() -> anyhow::Result<()> {
    // Phase 6で実装
    Ok(())
}
```

### Task 1.3: 空モジュール作成

- `src/config.rs` - 空のモジュール
- `src/worktree.rs` - 空のモジュール
- `src/copy.rs` - 空のモジュール
- `src/commands.rs` - 空のモジュール

**注**: interactive.rsは対話モード（Phase 7）のため今回のスコープ外

### 検証方法
```bash
cargo check  # コンパイルエラーなし
cargo build  # ビルド成功
```

---

## Phase 2: 設定ファイル管理（必須）

### Task 2.1: 型定義（config.rs）

```rust
/// 設定ファイル構造体
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub copy: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default, rename = "postCreate")]
    pub post_create: Vec<PostCreateCommand>,
}

#[derive(Debug, Deserialize)]
pub struct PostCreateCommand {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub optional: bool,
}
```

### Task 2.2: 設定ファイル検索

検索順序（YAMLのみ）:
1. `.worktree.yml`
2. `.worktree.yaml`

```rust
const CONFIG_FILE_NAMES: &[&str] = &[
    ".worktree.yml",
    ".worktree.yaml",
];

pub fn find_config_file(dir: &Path) -> Option<PathBuf>
pub fn load_config(path: &Path) -> Result<Config>
pub fn load_config_or_default(dir: &Path) -> Result<Config>
```

### Task 2.3: YAMLパーサー

`serde_yaml::from_str()`を使用してパース

### Task 2.4: 設定ファイル初期化（initコマンド用）

```rust
pub fn create_default_config(dir: &Path, force: bool) -> Result<PathBuf>
```

デフォルト設定テンプレート:
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

### 検証方法
```bash
# テスト用設定ファイル作成
echo 'version: 1' > .worktree.yml
cargo test config::tests
```

---

## Phase 3: Git操作（必須）

### Task 3.1: リポジトリ情報取得

```rust
/// Gitリポジトリのルートディレクトリを取得
pub fn get_repo_root() -> Result<PathBuf>

/// メインworktreeのパスを取得
pub fn get_main_worktree() -> Result<PathBuf>

/// 現在のディレクトリがメインworktreeかどうか
pub fn is_main_worktree() -> Result<bool>
```

### Task 3.2: ブランチ操作

```rust
/// ブランチが存在するか確認
pub fn branch_exists(branch: &str) -> Result<bool>

/// 現在のブランチ名を取得
pub fn get_current_branch() -> Result<String>
```

### Task 3.3: worktree作成

```rust
/// worktreeを作成
/// - 新規ブランチ: git worktree add -b <branch> <path>
/// - 既存ブランチ: git worktree add <path> <branch>
pub fn create_worktree(path: &Path, branch: &str) -> Result<()>
```

### Task 3.4: worktree一覧

```rust
#[derive(Debug)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_main: bool,
}

pub fn list_worktrees() -> Result<Vec<WorktreeInfo>>
```

### Task 3.5: worktree削除

```rust
/// worktreeを削除
/// force=trueの場合: git worktree remove --force
pub fn remove_worktree(path: &Path, force: bool) -> Result<()>
```

### 検証方法
```bash
# Gitリポジトリ内で実行
cargo test worktree::tests
```

---

## Phase 4: ファイル操作（必須）

### Task 4.1: globパターンマッチング

```rust
/// パターンにマッチするファイルを取得
pub fn expand_patterns(base_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>>
```

### Task 4.2: 除外フィルター

```rust
/// 除外パターンにマッチするファイルを除外
pub fn filter_excluded(files: Vec<PathBuf>, excludes: &[String]) -> Vec<PathBuf>
```

### Task 4.3: ファイルコピー

```rust
#[derive(Debug)]
pub struct CopyResult {
    pub copied: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,  // (path, error_message)
}

/// ファイルをコピー（個別エラーでも続行）
pub fn copy_files(
    files: &[PathBuf],
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<CopyResult>
```

- 親ディレクトリは`create_dir_all`で自動作成
- 個別ファイルのエラーは警告表示して続行
- シンボリックリンクは通常ファイルとしてコピー

### 検証方法
```bash
cargo test copy::tests
```

---

## Phase 5: コマンド実行（必須）

### Task 5.1: 外部コマンド実行

```rust
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

/// コマンドを実行
pub fn run_command(
    command: &str,
    working_dir: &Path,
    description: Option<&str>,
) -> Result<CommandResult>
```

### Task 5.2: プログレス表示

```rust
/// スピナー付きでコマンドを実行
pub fn run_with_spinner(
    command: &str,
    working_dir: &Path,
    description: &str,
) -> Result<CommandResult>
```

indicatifを使用:
- 実行中: スピナー + 説明文
- 成功: ✓ + 所要時間
- 失敗: ✗ + エラーメッセージ

### Task 5.3: post-createコマンド実行

```rust
/// 設定のpost-createコマンドを順次実行
pub fn run_post_create_commands(
    commands: &[PostCreateCommand],
    working_dir: &Path,
) -> Result<()>
```

- `optional: true`のコマンドは失敗しても続行
- `optional: false`のコマンドは失敗で中断

### 検証方法
```bash
cargo test commands::tests
```

---

## Phase 6: CLI実装（必須）

### Task 6.1: CLIパーサー定義（main.rs）

```rust
#[derive(Parser)]
#[command(name = "wtenv")]
#[command(about = "Git worktree environment manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create(CreateArgs),
    List,
    Remove(RemoveArgs),
    Init(InitArgs),
    Config,
}
```

### Task 6.2: createサブコマンド

```rust
#[derive(Args)]
struct CreateArgs {
    /// ブランチ名（必須）
    branch: String,
    /// worktreeパス（省略時: ../branch-name）
    path: Option<PathBuf>,
    #[arg(long)]
    no_copy: bool,
    #[arg(long)]
    no_post_create: bool,
    #[arg(short, long)]
    config: Option<PathBuf>,
}
```

処理フロー:
1. メインworktree確認
2. 設定ファイル読み込み
3. path未指定なら`../branch-name`をデフォルト値として使用
4. worktree作成
5. ファイルコピー（--no-copy指定時はスキップ）
6. post-createコマンド実行（--no-post-create指定時はスキップ）

**注**: `--verbose`オプションはPhase 8（スコープ外）のため省略

### Task 6.3: listサブコマンド

```
📁 /home/user/project (main)           [main] abc1234
📁 /home/user/feature-auth             [feature-auth] def5678
📁 /home/user/bugfix-login             [bugfix-login] ghi9012
```

### Task 6.4: removeサブコマンド

```rust
#[derive(Args)]
struct RemoveArgs {
    /// Worktree path to remove
    path: PathBuf,
    #[arg(short, long)]
    force: bool,
}
```

### Task 6.5: initサブコマンド

```rust
#[derive(Args)]
struct InitArgs {
    #[arg(short, long)]
    force: bool,
}
```

### Task 6.6: configサブコマンド

現在の設定を表示（YAML形式）

### 検証方法
```bash
cargo run -- --help
cargo run -- create --help
cargo run -- init
cargo run -- config
```

---

## Phase 7: 対話モード（推奨）

### Task 7.1: 依存関係追加

```toml
[dependencies]
dialoguer = "0.11"
```

### Task 7.2: interactive.rs作成

```rust
// src/interactive.rs
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use std::path::PathBuf;

/// ブランチ名を対話的に入力
pub fn prompt_branch_name() -> Result<String> {
    let branch: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("ブランチ名")
        .interact_text()?;

    if branch.trim().is_empty() {
        anyhow::bail!("ブランチ名を入力してください");
    }

    Ok(branch)
}

/// worktreeパスを対話的に入力
pub fn prompt_worktree_path(default: &str) -> Result<PathBuf> {
    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("worktreeパス")
        .default(default.to_string())
        .interact_text()?;

    Ok(PathBuf::from(path))
}

/// 削除確認
pub fn confirm_remove(path: &std::path::Path) -> Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("本当に削除しますか？: {}", path.display()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

/// 上書き確認
pub fn confirm_overwrite(path: &std::path::Path) -> Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("既存のファイルを上書きしますか？: {}", path.display()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}
```

### Task 7.3: createコマンドを対話モード対応に

```rust
// src/main.rs
#[derive(Args)]
struct CreateArgs {
    /// ブランチ名（省略時は対話モード）
    branch: Option<String>,
    /// worktreeパス（省略時は対話モード）
    path: Option<PathBuf>,
    #[arg(long)]
    no_copy: bool,
    #[arg(long)]
    no_post_create: bool,
    #[arg(short, long)]
    config: Option<PathBuf>,
}
```

処理フロー:
1. branch未指定なら`interactive::prompt_branch_name()`
2. path未指定なら`interactive::prompt_worktree_path()`でデフォルト値を提案
3. 既存パスがある場合は警告

### Task 7.4: removeコマンドを確認ダイアログ対応に

```rust
// --forceがない場合は確認ダイアログを表示
if !args.force {
    if !interactive::confirm_remove(&args.path)? {
        println!("キャンセルされました");
        return Ok(());
    }
}
```

### Task 7.5: initコマンドを上書き確認対応に

```rust
// --forceがない場合で既存ファイルがある場合は確認
if config_path.exists() && !args.force {
    if !interactive::confirm_overwrite(&config_path)? {
        println!("キャンセルされました");
        return Ok(());
    }
}
```

### 検証方法
```bash
# 引数なしで対話モード
cargo run -- create

# 削除時に確認ダイアログ
cargo run -- remove ../test-branch

# 設定ファイル上書き確認
cargo run -- init
```

---

## Phase 8: UX向上（推奨）

### Task 8.1: --verboseオプション追加

```rust
// すべてのサブコマンドに追加
#[derive(Args)]
struct CreateArgs {
    // ... 既存フィールド
    /// 詳細出力
    #[arg(short, long)]
    verbose: bool,
}
```

詳細モードで追加出力:
- 設定ファイルのパスと内容
- 各処理の詳細情報
- gitコマンドの完全な出力
- ファイルコピーの詳細（各ファイルのサイズ等）
- 処理時間の詳細

### Task 8.2: カラー出力の強化

```rust
// src/output.rs (新規作成)
use colored::*;

pub struct OutputStyle;

impl OutputStyle {
    pub fn success(msg: &str) -> ColoredString {
        format!("✓ {}", msg).green()
    }

    pub fn error(msg: &str) -> ColoredString {
        format!("✗ {}", msg).red()
    }

    pub fn warning(msg: &str) -> ColoredString {
        format!("⚠ {}", msg).yellow()
    }

    pub fn info(msg: &str) -> ColoredString {
        format!("ℹ {}", msg).blue()
    }

    pub fn path(path: &std::path::Path) -> ColoredString {
        path.display().to_string().cyan()
    }

    pub fn command(cmd: &str) -> ColoredString {
        cmd.bright_black()
    }

    pub fn header(msg: &str) -> ColoredString {
        msg.bold().blue()
    }
}
```

### Task 8.3: プログレス表示の改善

```rust
// indicatifのプログレスバーを追加
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn create_progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(msg.to_string());
    pb
}
```

ファイルコピー時に使用:
```rust
let pb = create_progress_bar(files.len() as u64, "ファイルをコピー中");
for file in files {
    // コピー処理
    pb.inc(1);
}
pb.finish_with_message("完了");
```

### Task 8.4: エラーメッセージテンプレート

```rust
// src/errors.rs (新規作成)
pub fn format_git_error(operation: &str, stderr: &str) -> String {
    format!(
        "❌ Git操作が失敗しました: {}\n\n\
         エラー内容:\n{}\n\n\
         ヒント:\n\
         - gitがインストールされているか確認してください\n\
         - Gitリポジトリ内で実行しているか確認してください\n\
         - 'git status' で状態を確認してください",
        operation,
        stderr.trim()
    )
}

pub fn format_file_error(operation: &str, path: &std::path::Path, error: &std::io::Error) -> String {
    format!(
        "❌ ファイル操作が失敗しました: {}\n\n\
         パス: {}\n\
         エラー: {}\n\n\
         ヒント:\n\
         - ファイル/ディレクトリが存在するか確認してください\n\
         - 書き込み権限があるか確認してください",
        operation,
        path.display(),
        error
    )
}
```

### Task 8.5: --quietオプション（サイレントモード）

```rust
#[arg(short, long)]
quiet: bool,
```

quietモードでは:
- エラー以外の出力を抑制
- プログレス表示なし
- 最終結果のみ出力

### 検証方法
```bash
cargo run -- create test-branch --verbose
cargo run -- create test-branch2 --quiet
cargo run -- list --verbose
```

---

## Phase 9: ドキュメント（オプション）

### Task 9.1: README.md（英語版）

```markdown
# wtenv - Git Worktree Environment Manager

Fast and dependency-free git worktree management CLI tool.

## Features

- 🌲 Easy worktree creation with branch management
- 📋 Automatic environment file copying (based on config)
- 📦 Post-create command execution
- ⚡ Fast startup (< 50ms)
- 🎨 Beautiful CLI with colors and progress indicators

## Installation

### From Binary
Download from [Releases](https://github.com/USERNAME/wtenv/releases)

### From Source
\`\`\`bash
cargo install --path .
\`\`\`

## Quick Start

\`\`\`bash
# Initialize config file
wtenv init

# Create worktree
wtenv create feature-branch

# List worktrees
wtenv list

# Remove worktree
wtenv remove ../feature-branch
\`\`\`

## Configuration

Create `.worktree.yml` in your repository root:

\`\`\`yaml
version: 1

copy:
  - .env
  - .env.local

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "Installing dependencies..."
\`\`\`

## License

MIT
```

### Task 9.2: README.ja.md（日本語版）

```markdown
# wtenv - Git Worktree環境マネージャー

高速で依存関係のないgit worktree管理CLIツール。

## 機能

- 🌲 ブランチ管理を含む簡単なworktree作成
- 📋 環境ファイルの自動コピー（設定ベース）
- 📦 post-createコマンドの実行
- ⚡ 高速起動（50ms未満）
- 🎨 カラーとプログレス表示による美しいCLI

## インストール

### バイナリから
[Releases](https://github.com/USERNAME/wtenv/releases)からダウンロード

### ソースから
\`\`\`bash
cargo install --path .
\`\`\`

## クイックスタート

\`\`\`bash
# 設定ファイル初期化
wtenv init

# worktree作成
wtenv create feature-branch

# worktree一覧
wtenv list

# worktree削除
wtenv remove ../feature-branch
\`\`\`

## 設定

リポジトリルートに`.worktree.yml`を作成:

\`\`\`yaml
version: 1

copy:
  - .env
  - .env.local

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."
\`\`\`

## ライセンス

MIT
```

### Task 9.3: INSTALL.md

インストール手順の詳細:
- バイナリインストール（各OS別）
- Cargoからのインストール
- ソースからのビルド
- シェル補完の設定
- トラブルシューティング

### Task 9.4: CHANGELOG.md

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-12-30

### Added
- Initial release
- Basic worktree operations (create, list, remove)
- Configuration file support (YAML)
- File copying with glob patterns
- Post-create command execution
- Colored output and progress indicators
```

### Task 9.5: LICENSE（MIT）

```
MIT License

Copyright (c) 2025 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction...
```

### Task 9.6: CONTRIBUTING.md

コントリビューションガイドライン:
- 開発環境のセットアップ
- コーディング規約
- プルリクエストのプロセス
- バグレポートの方法

### Task 9.7: docs/examples/

使用例のディレクトリ:
- `basic-usage.md` - 基本的な使い方
- `advanced-config.md` - 高度な設定例
- `monorepo.md` - モノレポでの使用例
- `ci-integration.md` - CI/CDでの使用例

---

## Phase 10: CI/CD・配布（オプション）

### Task 10.1: GitHub Actions - CI

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, 1.92.0]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --verbose

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Build
        run: cargo build --release

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
```

### Task 10.2: GitHub Actions - Release

`.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: wtenv
            asset_name: wtenv-linux-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact_name: wtenv
            asset_name: wtenv-linux-x64-musl
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: wtenv
            asset_name: wtenv-macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: wtenv
            asset_name: wtenv-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: wtenv.exe
            asset_name: wtenv-windows-x64.exe

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Strip binary (Unix)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/${{ matrix.artifact_name }}

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.asset_name }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Task 10.3: クロスコンパイル設定

`.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.aarch64-apple-darwin]
linker = "aarch64-apple-darwin-clang"
```

### Task 10.4: インストールスクリプト

`install.sh`:

```bash
#!/bin/bash
set -e

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case "$ARCH" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Download URL
VERSION="latest"
BINARY_NAME="wtenv-${OS}-${ARCH}"
DOWNLOAD_URL="https://github.com/USERNAME/wtenv/releases/latest/download/${BINARY_NAME}"

# Install location
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

# Download and install
echo "Downloading wtenv..."
curl -L "$DOWNLOAD_URL" -o "${INSTALL_DIR}/wtenv"
chmod +x "${INSTALL_DIR}/wtenv"

echo "wtenv installed to ${INSTALL_DIR}/wtenv"
echo "Make sure ${INSTALL_DIR} is in your PATH"
```

### Task 10.5: Cargo配布

`Cargo.toml`に追加:

```toml
[package]
# ... 既存設定
repository = "https://github.com/USERNAME/wtenv"
homepage = "https://github.com/USERNAME/wtenv"
documentation = "https://docs.rs/wtenv"
keywords = ["git", "worktree", "cli", "tool"]
categories = ["command-line-utilities", "development-tools"]
readme = "README.md"
```

crates.ioへの公開:
```bash
cargo login
cargo publish --dry-run
cargo publish
```

### Task 10.6: Homebrewフォーミュラ

`homebrew-wtenv/Formula/wtenv.rb`:

```ruby
class Wtenv < Formula
  desc "Git worktree environment manager"
  homepage "https://github.com/USERNAME/wtenv"
  url "https://github.com/USERNAME/wtenv/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/wtenv", "--version"
  end
end
```

### 検証方法

```bash
# ローカルでリリースビルドテスト
cargo build --release
ls -lh target/release/wtenv

# クロスコンパイルテスト
cargo build --release --target x86_64-unknown-linux-musl

# インストールスクリプトテスト
bash install.sh

# GitHub Actionsのローカルテスト（act使用）
act -j test
```

---

## 実装順序の理由

```
Phase 1-6（コア機能）:
config.rs (Phase 2)
    ↓ 依存
worktree.rs (Phase 3) ← 設定読み込みで使用
    ↓ 依存
copy.rs (Phase 4) ← worktree作成後に使用
    ↓ 依存
commands.rs (Phase 5) ← コピー後に使用
    ↓ 依存
main.rs (Phase 6) ← すべてを統合

Phase 7-10（拡張機能）:
interactive.rs (Phase 7) ← main.rsの対話化
    ↓ 拡張
output.rs (Phase 8) ← カラー出力強化
    ↓ 独立
docs/ (Phase 9) ← ドキュメント整備
    ↓ 独立
.github/ (Phase 10) ← CI/CD・配布
```

**Phase 1-6の順序理由:**
1. **config.rs を最初に**: すべての機能が設定を参照するため
2. **worktree.rs を次に**: コア機能であり、他の機能の前提条件
3. **copy.rs をその後**: worktree作成後に実行される
4. **commands.rs をその後**: コピー完了後に実行される
5. **main.rs で統合**: すべてのモジュールを組み合わせる

**Phase 7-10の順序理由:**
6. **interactive.rs (Phase 7)**: Phase 6完了後、既存CLIに対話性を追加
7. **output.rs (Phase 8)**: Phase 7と並行可能、出力の改善
8. **docs (Phase 9)**: 機能完成後にドキュメント作成が効率的
9. **CI/CD (Phase 10)**: コード・ドキュメント完成後に自動化を追加

---

## 想定される課題と対策

### 課題1: Windowsでのパス区切り文字

**対策:** `std::path::PathBuf`と`Path::join()`を一貫して使用。
文字列でのパス結合は行わない。

### 課題2: globパターンでディレクトリがマッチ

**対策:** `glob::glob()`の結果を`is_file()`でフィルタリング。

```rust
for entry in glob(pattern)? {
    let path = entry?;
    if path.is_file() {
        files.push(path);
    }
}
```

### 課題3: post-createコマンドのシェル実行

**対策:** プラットフォームごとに分岐:
- Unix: `sh -c "command"`
- Windows: `cmd /C "command"`

```rust
#[cfg(unix)]
fn shell_command(cmd: &str) -> Command {
    let mut c = Command::new("sh");
    c.args(["-c", cmd]);
    c
}

#[cfg(windows)]
fn shell_command(cmd: &str) -> Command {
    let mut c = Command::new("cmd");
    c.args(["/C", cmd]);
    c
}
```

### 課題4: Git操作のエラーメッセージが不親切

**対策:** gitのstderrを解析して、よくあるエラーには追加の説明を付与（日本語）。

```rust
if stderr.contains("already exists") {
    anyhow::bail!(
        "❌ worktreeは既に存在します: {}\n\n\
         'wtenv list' で既存のworktreeを確認してください。",
        path.display()
    );
}
```

### 課題5: 大量ファイルコピー時のパフォーマンス

**対策:** シンプルな実装を優先（claude.mdの方針に従う）。
ファイル数が100を超える場合は警告を表示。

---

## 各Phase完了時の成果物

### Phase 1完了
- `cargo build`が成功
- 空のバイナリが生成される

### Phase 2完了
- 設定ファイルの読み書きが可能
- `cargo test config`が成功

### Phase 3完了
- worktreeの作成・一覧・削除が可能
- `cargo test worktree`が成功

### Phase 4完了
- globパターンでファイルコピーが可能
- `cargo test copy`が成功

### Phase 5完了
- 外部コマンドの実行が可能
- スピナー表示が動作
- `cargo test commands`が成功

### Phase 6完了（コア機能完成）
- **全サブコマンドが動作**
- `wtenv create feature-x ../feature-x`が完全動作
- `wtenv list`が動作
- `wtenv remove ../feature-x`が動作
- `wtenv init`が動作
- `wtenv config`が動作
- 基本的なカラー出力が適用
- 日本語エラーメッセージが表示

### Phase 7完了
- 引数なしで対話モードが動作
- `wtenv create`で対話的にブランチ名・パス入力
- `wtenv remove`で削除確認ダイアログ表示
- `wtenv init`で上書き確認ダイアログ表示
- dialoguerによる美しい対話UI

### Phase 8完了
- `--verbose`オプションで詳細出力
- `--quiet`オプションでサイレント実行
- プログレスバーによる視覚的フィードバック
- 統一されたエラーメッセージフォーマット
- より洗練されたカラー出力

### Phase 9完了
- README.md（英語）
- README.ja.md（日本語）
- INSTALL.md
- CHANGELOG.md
- LICENSE（MIT）
- CONTRIBUTING.md
- docs/examples/（使用例集）

### Phase 10完了（リリース準備完了）
- GitHub Actions CI/CD設定
- クロスプラットフォームビルド（5プラットフォーム）
- インストールスクリプト

---

## 優先順位と実装レベル

### 必須（Phase 1-6）✅ 完了
**コア機能** - プロダクションで使用可能な最小限の機能
- プロジェクト基盤
- 設定ファイル管理
- Git操作
- ファイル操作
- コマンド実行
- CLI実装

### 推奨（Phase 7-8）
**UX強化** - ユーザー体験を大幅に向上
- 対話モード（引数なしで実行可能）
- 詳細/サイレントモード
- プログレスバー
- 統一されたエラーメッセージ

### オプション（Phase 9）
**ドキュメント** - ユーザー・コントリビューター向け
- README（英語・日本語）
- インストール手順
- 使用例集
- コントリビューションガイド

### オプション（Phase 10）
**配布・CI/CD** - オープンソース公開準備
- GitHub Actions設定
- クロスプラットフォームビルド
- 自動リリース

---

## テスト戦略

### 単体テスト（各モジュール内）

```rust
// src/config.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_yaml_config() {
        let content = "version: 1\ncopy:\n  - .env";
        let config: Config = serde_yaml::from_str(content).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.copy, vec![".env"]);
    }
}
```

### 重点テスト対象
1. 設定ファイルパース（正常系・異常系）
2. globパターンマッチング
3. 除外フィルター
4. Git操作のエラーハンドリング

---

## クリティカルファイル

実装時に重点的に確認すべきファイル:

| ファイル | 重要度 | 理由 |
|---------|--------|------|
| `src/config.rs` | 高 | 全機能の基盤 |
| `src/worktree.rs` | 高 | コア機能 |
| `src/main.rs` | 高 | エントリーポイント |
| `src/copy.rs` | 中 | ファイル操作 |
| `src/commands.rs` | 中 | 外部コマンド |
