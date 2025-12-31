use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::worktree::{self, info::WorktreeDetail, process::ProcessManager};

/// statusコマンドの実行
pub fn execute(verbose: bool) -> Result<()> {
    let repo_root = worktree::get_repo_root()?;
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "worktreeが見つかりませんでした".yellow());
        return Ok(());
    }

    // プロセス情報を読み込み
    let mut process_manager = ProcessManager::load(&repo_root)?;
    process_manager.cleanup_dead_processes();

    // 詳細情報を取得
    let mut details = Vec::new();
    for wt in &worktrees {
        match WorktreeDetail::from_path(&wt.path, wt.branch.clone(), wt.commit.clone(), wt.is_main)
        {
            Ok(detail) => details.push(detail),
            Err(e) => {
                eprintln!(
                    "{} {}のworktree情報取得に失敗: {}",
                    "⚠️".yellow(),
                    wt.path.display(),
                    e
                );
            }
        }
    }

    // ヘッダー表示
    print_header(&details, &process_manager);

    // worktreeごとの詳細表示
    for detail in &details {
        print_worktree_status(detail, &process_manager, verbose);
    }

    // フッター表示
    print_footer(&details);

    Ok(())
}

/// ヘッダー表示
fn print_header(details: &[WorktreeDetail], process_manager: &ProcessManager) {
    let active_count = process_manager.running_processes().len();

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!(
        "│ {} ({} active, {} processes)                               ",
        "Worktrees Overview".bold(),
        details.len().to_string().cyan(),
        active_count.to_string().green()
    );
    println!("├─────────────────────────────────────────────────────────────┤");
}

/// フッター表示
fn print_footer(details: &[WorktreeDetail]) {
    println!("├─────────────────────────────────────────────────────────────┤");

    let total_modified = details
        .iter()
        .map(|d| d.modified_files + d.untracked_files)
        .sum::<usize>();

    println!(
        "│ {}: {}  |  {}: {} files",
        "📊 Total".bright_black(),
        format!("{} worktrees", details.len()).cyan(),
        "Modified".bright_black(),
        total_modified.to_string().yellow()
    );
    println!("└─────────────────────────────────────────────────────────────┘");
}

/// worktreeの状態を表示
fn print_worktree_status(detail: &WorktreeDetail, process_manager: &ProcessManager, verbose: bool) {
    let path = PathBuf::from(&detail.path);
    let branch_name = detail.branch.as_deref().unwrap_or("detached");

    // プロセス情報を取得
    let processes = process_manager.processes_by_worktree(&detail.path);
    let process_info = if let Some(proc) = processes.first() {
        format!("Process: {}", proc.command)
    } else {
        "No process".to_string()
    };

    // 状態の絵文字とテキスト
    let emoji = detail.status_emoji();
    let status_text = detail.status_text();

    // ブランチ名表示
    println!("│");
    println!(
        "│ {} {:<30} {}",
        emoji,
        branch_name.green(),
        if detail.is_main {
            "(main)".bright_black()
        } else {
            "".bright_black()
        }
    );

    // 状態とプロセス情報
    println!(
        "│    {:<25} {}",
        format!("Status: {}", status_text).bright_black(),
        process_info.bright_black()
    );

    // 変更ファイル数と最終コミット
    if detail.has_changes() {
        println!(
            "│    Modified: {}  |  Last commit: {}",
            format!("{} files", detail.modified_files + detail.untracked_files).yellow(),
            detail.last_commit_time.bright_black()
        );
    } else {
        println!(
            "│    Last commit: {}",
            detail.last_commit_time.bright_black()
        );
    }

    // パス表示
    if verbose {
        println!("│    Path: {}", path.display().to_string().cyan());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_not_git_repo() {
        // Gitリポジトリ外での実行は失敗するはず
        std::env::set_current_dir("/tmp").ok();
        let result = execute(false);
        // Gitリポジトリ内でなければエラー
        assert!(result.is_err(), "Should fail outside git repository");
    }
}
