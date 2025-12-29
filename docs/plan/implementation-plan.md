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

## Phase 7-10: スコープ外

以下のPhaseは今回のスコープ外です（Phase 6完了後、必要に応じて追加実装）:

- **Phase 7**: 対話モード（dialoguer使用）
- **Phase 8**: UX向上（カラー出力強化、--verbose）
- **Phase 9**: ドキュメント（README、LICENSE等）
- **Phase 10**: CI/CD・配布

**ただし、Phase 6でも最低限のカラー出力とエラーメッセージは実装します。**

---

## 実装順序の理由

```
config.rs (Phase 2)
    ↓ 依存
worktree.rs (Phase 3) ← 設定読み込みで使用
    ↓ 依存
copy.rs (Phase 4) ← worktree作成後に使用
    ↓ 依存
commands.rs (Phase 5) ← コピー後に使用
    ↓ 依存
main.rs (Phase 6) ← すべてを統合
```

**理由:**
1. **config.rs を最初に**: すべての機能が設定を参照するため
2. **worktree.rs を次に**: コア機能であり、他の機能の前提条件
3. **copy.rs をその後**: worktree作成後に実行される
4. **commands.rs をその後**: コピー完了後に実行される
5. **main.rs で統合**: すべてのモジュールを組み合わせる

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

### Phase 6完了（最終目標）
- **全サブコマンドが動作**
- `wtenv create feature-x ../feature-x`が完全動作
- `wtenv list`が動作
- `wtenv remove ../feature-x`が動作
- `wtenv init`が動作
- `wtenv config`が動作
- 基本的なカラー出力が適用
- 日本語エラーメッセージが表示

---

## 優先順位

### 今回実装（Phase 1-6）
- プロジェクト基盤
- 設定ファイル管理
- Git操作
- ファイル操作
- コマンド実行
- CLI実装

### 将来の拡張（Phase 7-10）
- 対話モード
- UX向上
- ドキュメント
- CI/CD・配布

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
