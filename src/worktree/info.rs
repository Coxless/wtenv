use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// worktreeの詳細情報
#[derive(Debug, Clone)]
pub struct WorktreeDetail {
    pub path: String,
    pub branch: Option<String>,
    pub commit: String,
    pub is_main: bool,
    pub modified_files: usize,
    pub untracked_files: usize,
    pub last_commit_time: String,
    pub ahead_commits: usize,
    pub behind_commits: usize,
}

impl WorktreeDetail {
    /// worktreeの詳細情報を取得
    pub fn from_path(
        path: &Path,
        branch: Option<String>,
        commit: String,
        is_main: bool,
    ) -> Result<Self> {
        let modified_files = count_modified_files(path)?;
        let untracked_files = count_untracked_files(path)?;
        let last_commit_time = get_last_commit_time(path)?;
        let (ahead_commits, behind_commits) = get_ahead_behind_commits(path, &branch)?;

        Ok(Self {
            path: path.display().to_string(),
            branch,
            commit,
            is_main,
            modified_files,
            untracked_files,
            last_commit_time,
            ahead_commits,
            behind_commits,
        })
    }

    /// 変更があるか
    pub fn has_changes(&self) -> bool {
        self.modified_files > 0 || self.untracked_files > 0
    }

    /// 状態の絵文字を取得
    pub fn status_emoji(&self) -> &'static str {
        if self.has_changes() {
            "🔄"
        } else if self.ahead_commits > 0 {
            "✅"
        } else {
            "📁"
        }
    }

    /// 状態の説明を取得
    pub fn status_text(&self) -> String {
        if self.has_changes() {
            format!(
                "Modified ({} files)",
                self.modified_files + self.untracked_files
            )
        } else if self.ahead_commits > 0 {
            format!("Ahead: {} commits", self.ahead_commits)
        } else {
            "Clean".to_string()
        }
    }
}

/// 変更されたファイルの数を取得
fn count_modified_files(path: &Path) -> Result<usize> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .context("git statusの実行に失敗しました")?;

    if !output.status.success() {
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("??"))
        .count();

    Ok(count)
}

/// 未追跡ファイルの数を取得
fn count_untracked_files(path: &Path) -> Result<usize> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .context("git statusの実行に失敗しました")?;

    if !output.status.success() {
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.lines().filter(|line| line.starts_with("??")).count();

    Ok(count)
}

/// 最終コミット時刻を取得
fn get_last_commit_time(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ar"])
        .current_dir(path)
        .output()
        .context("git logの実行に失敗しました")?;

    if !output.status.success() {
        return Ok("unknown".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

/// upstream とのコミット差分を取得
fn get_ahead_behind_commits(path: &Path, branch: &Option<String>) -> Result<(usize, usize)> {
    let branch = match branch {
        Some(b) => b,
        None => return Ok((0, 0)),
    };

    // upstreamブランチが存在するか確認
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--abbrev-ref",
            &format!("{}@{{upstream}}", branch),
        ])
        .current_dir(path)
        .output()
        .context("git rev-parseの実行に失敗しました")?;

    if !output.status.success() {
        // upstreamが設定されていない
        return Ok((0, 0));
    }

    // ahead commits
    let ahead_output = Command::new("git")
        .args(["rev-list", "--count", &format!("@{{upstream}}..HEAD")])
        .current_dir(path)
        .output()
        .context("git rev-listの実行に失敗しました")?;

    let ahead = if ahead_output.status.success() {
        String::from_utf8_lossy(&ahead_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    // behind commits
    let behind_output = Command::new("git")
        .args(["rev-list", "--count", "HEAD..@{upstream}"])
        .current_dir(path)
        .output()
        .context("git rev-listの実行に失敗しました")?;

    let behind = if behind_output.status.success() {
        String::from_utf8_lossy(&behind_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    Ok((ahead, behind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_emoji() {
        let detail = WorktreeDetail {
            path: "/test".to_string(),
            branch: Some("main".to_string()),
            commit: "abc123".to_string(),
            is_main: true,
            modified_files: 3,
            untracked_files: 0,
            last_commit_time: "2 hours ago".to_string(),
            ahead_commits: 0,
            behind_commits: 0,
        };

        assert_eq!(detail.status_emoji(), "🔄");
        assert!(detail.has_changes());
    }
}
