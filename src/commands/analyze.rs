use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::worktree;

/// worktreeの分析情報
#[derive(Debug)]
pub struct AnalysisInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub disk_usage: u64,
    pub last_modified: Option<SystemTime>,
    pub has_node_modules: bool,
    pub has_package_lock: bool,
    pub has_build: bool,
    pub is_merged: bool,
    pub days_since_update: Option<u64>,
}

impl AnalysisInfo {
    /// worktreeのパスから分析情報を作成
    pub fn from_path(path: &Path, main_branch: &str, branch: Option<String>) -> Result<Self> {
        let disk_usage = calculate_dir_size(path)?;
        let last_modified = get_last_modified(path)?;
        let has_node_modules = path.join("node_modules").exists();
        let has_package_lock = path.join("package-lock.json").exists()
            || path.join("pnpm-lock.yaml").exists()
            || path.join("yarn.lock").exists();
        let has_build =
            path.join("dist").exists() || path.join("build").exists() || path.join("out").exists();

        // mainブランチにマージ済みかチェック
        let is_merged = if let Some(ref b) = branch {
            check_if_merged(&b, main_branch)?
        } else {
            false
        };

        // 最終更新からの日数を計算
        let days_since_update = last_modified.and_then(|lm| {
            SystemTime::now()
                .duration_since(lm)
                .ok()
                .map(|d| d.as_secs() / 86400)
        });

        Ok(Self {
            path: path.to_path_buf(),
            branch,
            disk_usage,
            last_modified,
            has_node_modules,
            has_package_lock,
            has_build,
            is_merged,
            days_since_update,
        })
    }

    /// ディスク使用量を人間が読みやすい形式で返す
    pub fn disk_usage_human(&self) -> String {
        format_size(self.disk_usage)
    }

    /// 最終更新日時を人間が読みやすい形式で返す
    pub fn last_modified_human(&self) -> String {
        match self.days_since_update {
            Some(0) => "Today".to_string(),
            Some(1) => "Yesterday".to_string(),
            Some(days) if days < 7 => format!("{} days ago", days),
            Some(days) if days < 30 => format!("{} weeks ago", days / 7),
            Some(days) => format!("{} months ago", days / 30),
            None => "Unknown".to_string(),
        }
    }
}

/// ディレクトリサイズを計算（再帰的）
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total = 0;

    if !path.exists() {
        return Ok(0);
    }

    // node_modulesやビルド成果物は除外してサイズを計算
    let exclude_dirs = ["node_modules", "dist", "build", "out", "target", ".git"];

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if exclude_dirs.contains(&file_name) {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += calculate_dir_size(&entry_path).unwrap_or(0);
                }
            }
        }
    }

    Ok(total)
}

/// ディレクトリ内の最終更新日時を取得
fn get_last_modified(path: &Path) -> Result<Option<SystemTime>> {
    let output = std::process::Command::new("git")
        .args(["-C", worktree::path_to_str(path)?, "log", "-1", "--format=%ct"])
        .output()?;

    if output.status.success() {
        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(timestamp) = timestamp_str.parse::<u64>() {
            return Ok(Some(
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp),
            ));
        }
    }

    Ok(None)
}

/// ブランチがmainにマージ済みかチェック
fn check_if_merged(branch: &str, main_branch: &str) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["branch", "--merged", main_branch])
        .output()?;

    if output.status.success() {
        let merged_branches = String::from_utf8_lossy(&output.stdout);
        Ok(merged_branches
            .lines()
            .any(|line| line.trim().trim_start_matches('*').trim().eq(branch)))
    } else {
        Ok(false)
    }
}

/// サイズを人間が読みやすい形式にフォーマット
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// analyzeコマンドの実行
pub fn execute(detailed: bool) -> Result<()> {
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found".yellow());
        return Ok(());
    }

    // mainブランチ名を取得
    let main_branch = worktree::get_main_branch_name().unwrap_or_else(|_| "main".to_string());

    println!("{}", "📊 Worktree Analysis".cyan().bold());
    println!();

    let mut total_size = 0u64;
    let mut merged_count = 0;
    let mut stale_count = 0;

    for wt in &worktrees {
        let analysis = AnalysisInfo::from_path(&wt.path, &main_branch, wt.branch.clone())?;
        total_size += analysis.disk_usage;

        if analysis.is_merged {
            merged_count += 1;
        }
        if analysis.days_since_update.unwrap_or(0) > 30 {
            stale_count += 1;
        }

        // ブランチ名
        let branch_display = analysis
            .branch
            .as_deref()
            .unwrap_or("(detached)")
            .green()
            .bold();

        println!("  {}", branch_display);

        // パス
        if detailed {
            println!(
                "    Path: {}",
                analysis.path.display().to_string().bright_black()
            );
        }

        // ディスク使用量
        println!("    Disk: {}", analysis.disk_usage_human().yellow());

        // 最終更新
        let last_update = analysis.last_modified_human();
        let last_update_colored = if analysis.days_since_update.unwrap_or(0) > 30 {
            last_update.red()
        } else if analysis.days_since_update.unwrap_or(0) > 7 {
            last_update.yellow()
        } else {
            last_update.green()
        };
        println!("    Last update: {}", last_update_colored);

        // 依存関係の状態
        let mut status_tags = Vec::new();
        if analysis.has_node_modules {
            status_tags.push("node_modules".blue());
        }
        if analysis.has_package_lock {
            status_tags.push("lockfile".cyan());
        }
        if analysis.has_build {
            status_tags.push("build".magenta());
        }
        if analysis.is_merged {
            status_tags.push("merged".green());
        }

        if !status_tags.is_empty() {
            print!("    Tags: ");
            for (i, tag) in status_tags.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", tag);
            }
            println!();
        }

        println!();
    }

    // サマリー
    println!("{}", "Summary".cyan().bold());
    println!(
        "  Total worktrees: {}",
        worktrees.len().to_string().yellow()
    );
    println!("  Total disk usage: {}", format_size(total_size).yellow());
    println!("  Merged branches: {}", merged_count.to_string().green());
    println!("  Stale (>30 days): {}", stale_count.to_string().red());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_size_decimal() {
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }
}
