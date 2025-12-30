use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::commands::analyze::AnalysisInfo;
use crate::worktree;

/// cleanオプション
pub struct CleanOptions {
    pub dry_run: bool,
    pub merged_only: bool,
    pub stale_days: Option<u64>,
    pub force: bool,
}

/// cleanコマンドの実行
pub fn execute(opts: CleanOptions) -> Result<()> {
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found".yellow());
        return Ok(());
    }

    // mainブランチ名を取得
    let main_branch = get_main_branch_name().unwrap_or_else(|_| "main".to_string());

    println!(
        "{}",
        if opts.dry_run {
            "🔍 Dry run: Analyzing worktrees for cleanup (no changes will be made)"
                .cyan()
                .bold()
        } else {
            "🧹 Cleaning up worktrees".cyan().bold()
        }
    );
    println!();

    let mut candidates = Vec::new();

    for wt in &worktrees {
        // mainワークツリーはスキップ
        if wt.is_main {
            continue;
        }

        let analysis = AnalysisInfo::from_path(&wt.path, &main_branch, wt.branch.clone())?;

        let mut should_clean = false;
        let mut reason = Vec::new();

        // マージ済みチェック
        if opts.merged_only && analysis.is_merged {
            should_clean = true;
            reason.push("merged to main".green());
        }

        // 古いworktreeチェック
        if let Some(stale_days) = opts.stale_days {
            if analysis.days_since_update.unwrap_or(0) > stale_days {
                should_clean = true;
                reason.push(format!("stale (>{} days)", stale_days).red());
            }
        }

        // merged_onlyもstale_daysも指定されていない場合は、両方の条件をチェック
        if !opts.merged_only && opts.stale_days.is_none() {
            if analysis.is_merged {
                should_clean = true;
                reason.push("merged to main".green());
            }
            if analysis.days_since_update.unwrap_or(0) > 30 {
                should_clean = true;
                reason.push("stale (>30 days)".red());
            }
        }

        if should_clean {
            candidates.push((wt.clone(), analysis, reason));
        }
    }

    if candidates.is_empty() {
        println!(
            "{}",
            "✨ No worktrees need cleaning. Everything looks good!".green()
        );
        return Ok(());
    }

    println!("Found {} worktree(s) to clean:", candidates.len());
    println!();

    for (wt, analysis, reasons) in &candidates {
        let branch = analysis.branch.as_deref().unwrap_or("(detached)").yellow();
        println!("  {} {}", "•".bright_black(), branch);
        println!("    Path: {}", wt.path.display().to_string().bright_black());
        println!("    Disk: {}", analysis.disk_usage_human().bright_black());

        print!("    Reason: ");
        for (i, r) in reasons.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", r);
        }
        println!();
        println!();
    }

    if opts.dry_run {
        println!(
            "{}",
            "ℹ️  Dry run complete. Use --force to actually remove these worktrees.".blue()
        );
        return Ok(());
    }

    // 確認プロンプト
    if !opts.force {
        println!(
            "{}",
            format!("Remove {} worktree(s)?", candidates.len())
                .yellow()
                .bold()
        );
        println!("{}", "Type 'yes' to confirm:".bright_black());

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() != "yes" {
            println!("{}", "❌ Aborted.".red());
            return Ok(());
        }
    }

    // 削除実行
    let mut removed_count = 0;
    let mut failed_count = 0;

    for (wt, analysis, _) in &candidates {
        let branch_display = analysis.branch.as_deref().unwrap_or("(detached)");

        match remove_worktree(&wt.path) {
            Ok(_) => {
                println!("  {} Removed {}", "✓".green(), branch_display.yellow());
                removed_count += 1;
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to remove {}: {}",
                    "✗".red(),
                    branch_display.yellow(),
                    e
                );
                failed_count += 1;
            }
        }
    }

    println!();
    println!(
        "{}",
        format!(
            "✨ Cleanup complete: {} removed, {} failed",
            removed_count, failed_count
        )
        .green()
        .bold()
    );

    Ok(())
}

/// worktreeを削除
fn remove_worktree(path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["worktree", "remove", path.to_str().unwrap(), "--force"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to remove worktree: {}", stderr);
    }

    Ok(())
}

/// mainブランチ名を取得
fn get_main_branch_name() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()?;

    if output.status.success() {
        let full_ref = String::from_utf8_lossy(&output.stdout);
        if let Some(branch) = full_ref.trim().strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    Ok("main".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_options_default() {
        let opts = CleanOptions {
            dry_run: true,
            merged_only: false,
            stale_days: None,
            force: false,
        };

        assert!(opts.dry_run);
        assert!(!opts.force);
    }
}
