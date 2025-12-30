pub mod info;
pub mod process;

// Re-export from worktree.rs for backward compatibility
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// worktree情報
#[derive(Debug)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_main: bool,
}

/// Gitリポジトリのルートディレクトリを取得
pub fn get_repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("gitコマンドの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "❌ Gitリポジトリではありません\n\n\
             このコマンドはGitリポジトリ内で実行する必要があります。\n\
             エラー: {}",
            stderr.trim()
        );
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// メインworktreeのパスを取得
#[allow(dead_code)]
pub fn get_main_worktree() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("git worktree listの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git worktree listが失敗しました: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 最初のworktreeがメインworktree
    for line in stdout.lines() {
        if line.starts_with("worktree ") {
            let path = line.strip_prefix("worktree ").unwrap_or("");
            return Ok(PathBuf::from(path));
        }
    }

    anyhow::bail!("メインworktreeが見つかりませんでした");
}

/// 現在のディレクトリがメインworktreeかどうか
#[allow(dead_code)]
pub fn is_main_worktree() -> Result<bool> {
    let current = std::env::current_dir().context("カレントディレクトリの取得に失敗しました")?;
    let main_worktree = get_main_worktree()?;

    Ok(current == main_worktree)
}

/// ブランチが存在するか確認
pub fn branch_exists(branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .output()
        .context("git rev-parseの実行に失敗しました")?;

    Ok(output.status.success())
}

/// 現在のブランチ名を取得
#[allow(dead_code)]
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("git branchの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("現在のブランチの取得に失敗しました: {}", stderr.trim());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        anyhow::bail!("ブランチ名を取得できませんでした（detached HEADの可能性があります）");
    }

    Ok(branch)
}

/// worktreeを作成
pub fn create_worktree(path: &Path, branch: &str) -> Result<()> {
    let exists = branch_exists(branch)?;

    let mut cmd = Command::new("git");
    cmd.arg("worktree").arg("add");

    if !exists {
        // 新規ブランチ
        cmd.arg("-b").arg(branch);
    }

    cmd.arg(path);

    if exists {
        // 既存ブランチ
        cmd.arg(branch);
    }

    let output = cmd
        .output()
        .context("git worktree addの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // エラーメッセージをわかりやすく
        if stderr.contains("already exists") {
            anyhow::bail!(
                "❌ worktreeは既に存在します: {}\n\n\
                 'wtenv list' で既存のworktreeを確認してください。",
                path.display()
            );
        } else if stderr.contains("is already checked out") {
            anyhow::bail!(
                "❌ ブランチ '{}' は既に別のworktreeでチェックアウトされています\n\n\
                 'wtenv list' で既存のworktreeを確認してください。",
                branch
            );
        } else {
            anyhow::bail!(
                "❌ worktreeの作成に失敗しました\n\n\
                 エラー: {}",
                stderr.trim()
            );
        }
    }

    Ok(())
}

/// worktree一覧を取得
pub fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("git worktree listの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git worktree listが失敗しました: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_worktree: Option<WorktreeInfo> = None;

    for line in stdout.lines() {
        if line.starts_with("worktree ") {
            if let Some(wt) = current_worktree.take() {
                worktrees.push(wt);
            }
            let path = line.strip_prefix("worktree ").unwrap_or("");
            current_worktree = Some(WorktreeInfo {
                path: PathBuf::from(path),
                branch: None,
                commit: String::new(),
                is_main: worktrees.is_empty(), // 最初のworktreeがメイン
            });
        } else if line.starts_with("HEAD ") {
            if let Some(ref mut wt) = current_worktree {
                let commit = line.strip_prefix("HEAD ").unwrap_or("");
                wt.commit = commit.to_string();
            }
        } else if line.starts_with("branch ") {
            if let Some(ref mut wt) = current_worktree {
                let branch = line.strip_prefix("branch ").unwrap_or("");
                let branch = branch.strip_prefix("refs/heads/").unwrap_or(branch);
                wt.branch = Some(branch.to_string());
            }
        }
    }

    if let Some(wt) = current_worktree {
        worktrees.push(wt);
    }

    Ok(worktrees)
}

/// worktreeを削除
pub fn remove_worktree(path: &Path, force: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "remove"]);

    if force {
        cmd.arg("--force");
    }

    cmd.arg(path);

    let output = cmd
        .output()
        .context("git worktree removeの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("does not exist") {
            anyhow::bail!(
                "❌ worktreeが見つかりません: {}\n\n\
                 'wtenv list' で既存のworktreeを確認してください。",
                path.display()
            );
        } else if stderr.contains("contains modified or untracked files") {
            anyhow::bail!(
                "❌ worktreeに変更されたファイルまたは未追跡のファイルがあります: {}\n\n\
                 強制的に削除する場合は --force オプションを使用してください。",
                path.display()
            );
        } else {
            anyhow::bail!(
                "❌ worktreeの削除に失敗しました\n\n\
                 エラー: {}",
                stderr.trim()
            );
        }
    }

    Ok(())
}
