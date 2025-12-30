use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

use crate::config;
use crate::copy;
use crate::worktree;

/// GitHub PR情報
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrInfo {
    pub number: u32,
    pub title: String,
    pub head_ref_name: String,
    pub head_repository_owner: HeadRepoOwner,
    pub state: String,
}

#[derive(Debug, Deserialize)]
struct HeadRepoOwner {
    pub login: String,
}

/// PR番号からworktreeを作成
pub fn execute(pr_number: u32, custom_path: Option<PathBuf>, verbose: bool) -> Result<()> {
    println!("{}", format!("🔍 Fetching PR #{}...", pr_number).cyan());

    // GitHub CLIが利用可能かチェック
    check_gh_cli_available()?;

    // PR情報を取得
    let pr_info = fetch_pr_info(pr_number)?;

    println!("{}", format!("✓ Found PR: {}", pr_info.title).green());
    println!("  Branch: {}", pr_info.head_ref_name.yellow());
    println!(
        "  Owner: {}",
        pr_info.head_repository_owner.login.bright_black()
    );
    println!("  State: {}", pr_info.state.bright_black());
    println!();

    // PRブランチをフェッチ
    println!("{}", "📥 Fetching PR branch...".cyan());
    fetch_pr_branch(pr_number, &pr_info.head_ref_name)?;

    // worktreeのパスを決定
    let worktree_path = determine_worktree_path(custom_path, &pr_info.head_ref_name)?;

    println!(
        "{}",
        format!("🌲 Creating worktree at {}...", worktree_path.display()).cyan()
    );

    // worktreeを作成
    create_worktree_from_pr(&pr_info.head_ref_name, &worktree_path)?;

    println!(
        "{}",
        format!("✓ Worktree created: {}", worktree_path.display()).green()
    );

    // 設定ファイルを読み込み
    let repo_root = worktree::get_repo_root()?;
    let config = config::load_config_or_default(&repo_root)?;

    // 環境ファイルをコピー
    if !config.copy.is_empty() {
        println!("\n{}", "📋 Copying environment files...".blue());

        // パターンを展開してファイルリストを作成
        let files = copy::expand_patterns(&repo_root, &config.copy)?;

        // 除外パターンでフィルタリング
        let files = copy::filter_excluded(files, &config.exclude);

        if verbose {
            println!("  Found {} files to copy", files.len());
        }

        let result = copy::copy_files(&files, &repo_root, &worktree_path)?;

        if verbose || !result.failed.is_empty() {
            println!(
                "  Copied: {}, Failed: {}",
                result.copied.len().to_string().green(),
                result.failed.len().to_string().red()
            );
        }
    }

    // post-createコマンドを実行
    if !config.post_create.is_empty() {
        use crate::commands::run_post_create_commands;
        run_post_create_commands(&config.post_create, &worktree_path)?;
    }

    println!();
    println!(
        "{}",
        format!("✨ PR #{} worktree is ready!", pr_number)
            .green()
            .bold()
    );
    println!("  cd {}", worktree_path.display().to_string().cyan());

    Ok(())
}

/// GitHub CLIが利用可能かチェック
fn check_gh_cli_available() -> Result<()> {
    let output = Command::new("gh")
        .args(["--version"])
        .output()
        .context("Failed to execute gh command")?;

    if !output.status.success() {
        anyhow::bail!(
            "❌ GitHub CLI (gh) is not available\n\n\
             Please install GitHub CLI: https://cli.github.com/\n\
             On macOS: brew install gh\n\
             On Linux: See https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
        );
    }

    Ok(())
}

/// PR情報を取得
fn fetch_pr_info(pr_number: u32) -> Result<PrInfo> {
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--json",
            "number,title,headRefName,headRepositoryOwner,state",
        ])
        .output()
        .context("Failed to fetch PR info")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "❌ Failed to fetch PR #{}\n\n\
             Error: {}\n\n\
             Make sure:\n\
             - The PR number is correct\n\
             - You have access to this repository\n\
             - You are authenticated with GitHub CLI (gh auth login)",
            pr_number,
            stderr.trim()
        );
    }

    let pr_info: PrInfo =
        serde_json::from_slice(&output.stdout).context("Failed to parse PR info")?;

    Ok(pr_info)
}

/// PRブランチをフェッチ
fn fetch_pr_branch(pr_number: u32, branch_name: &str) -> Result<()> {
    // まずPRをチェックアウト（これでリモートブランチが自動的にフェッチされる）
    let output = Command::new("gh")
        .args(["pr", "checkout", &pr_number.to_string()])
        .output()
        .context("Failed to checkout PR branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // すでにチェックアウト済みの場合は無視
        if !stderr.contains("already exists") {
            anyhow::bail!("Failed to fetch PR branch: {}", stderr.trim());
        }
    }

    // 元のブランチに戻る
    let current_branch = worktree::get_current_branch()?;
    let output = Command::new("git")
        .args(["checkout", &current_branch])
        .output()
        .context("Failed to return to original branch")?;

    if !output.status.success() {
        eprintln!(
            "{}",
            format!("⚠️  Warning: Could not return to branch {}", current_branch).yellow()
        );
    }

    // ブランチが存在するか確認
    let output = Command::new("git")
        .args(["rev-parse", "--verify", branch_name])
        .output()
        .context("Failed to verify branch")?;

    if !output.status.success() {
        anyhow::bail!("❌ Branch {} not found after fetching PR", branch_name);
    }

    Ok(())
}

/// worktreeのパスを決定
fn determine_worktree_path(custom_path: Option<PathBuf>, branch_name: &str) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        return Ok(path);
    }

    // デフォルト: ../リポジトリ名-ブランチ名
    let repo_root = worktree::get_repo_root()?;
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .context("Failed to get repository name")?;

    let parent = repo_root
        .parent()
        .context("Failed to get parent directory")?;

    // ブランチ名をファイルシステム安全な形式に変換
    let safe_branch_name = branch_name.replace('/', "-");

    Ok(parent.join(format!("{}-{}", repo_name, safe_branch_name)))
}

/// PRからworktreeを作成
fn create_worktree_from_pr(branch_name: &str, path: &PathBuf) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "add", worktree::path_to_str(path)?, branch_name])
        .output()
        .context("Failed to create worktree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "❌ Failed to create worktree\n\n\
             Error: {}\n\n\
             Branch: {}\n\
             Path: {}",
            stderr.trim(),
            branch_name,
            path.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_worktree_path_custom() {
        let custom = Some(PathBuf::from("/custom/path"));
        let result = determine_worktree_path(custom, "feature-branch");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_safe_branch_name() {
        let branch = "feature/add-new-feature";
        let safe = branch.replace('/', "-");
        assert_eq!(safe, "feature-add-new-feature");
    }
}
