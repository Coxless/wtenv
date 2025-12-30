use anyhow::{Context, Result};
use colored::Colorize;

use crate::worktree::{self, process::ProcessManager};

/// psコマンドの実行（全プロセス表示）
pub fn execute(worktree_filter: Option<String>) -> Result<()> {
    let repo_root = worktree::get_repo_root()?;
    let mut process_manager = ProcessManager::load(&repo_root)?;

    // 死んだプロセスをクリーンアップ
    process_manager.cleanup_dead_processes();
    process_manager.save(&repo_root)?;

    let running = process_manager.running_processes();

    if running.is_empty() {
        println!("{}", "実行中のプロセスはありません".yellow());
        return Ok(());
    }

    // フィルタリング
    let filtered: Vec<_> = if let Some(filter) = worktree_filter {
        running
            .into_iter()
            .filter(|p| p.worktree_path.contains(&filter) || p.branch.contains(&filter))
            .collect()
    } else {
        running
    };

    if filtered.is_empty() {
        println!("{}", "該当するプロセスが見つかりませんでした".yellow());
        return Ok(());
    }

    println!("{}\n", "Active Processes in Worktrees:".bold());

    let count = filtered.len();

    for proc in &filtered {
        let uptime = format_uptime(proc.uptime_secs());

        println!(
            "{} {}",
            proc.branch.green().bold(),
            format!("(PID: {})", proc.pid).bright_black()
        );
        println!("  {}: {}", "Command".bright_black(), proc.command.cyan());
        println!("  {}: {}", "Started".bright_black(), uptime);
        println!(
            "  {}: {}",
            "Working Dir".bright_black(),
            proc.worktree_path.bright_black()
        );
        println!("  {}: {}", "Status".bright_black(), "Running".green());
        println!();
    }

    println!(
        "{}: {} {}",
        "Total".bright_black(),
        count.to_string().cyan(),
        if count == 1 { "process" } else { "processes" }
    );

    Ok(())
}

/// killコマンドの実行
pub fn kill(pid: Option<u32>, all: bool, worktree_filter: Option<String>) -> Result<()> {
    let repo_root = worktree::get_repo_root()?;
    let mut process_manager = ProcessManager::load(&repo_root)?;

    if all {
        // 全プロセスをkill
        let running = process_manager.running_processes();
        if running.is_empty() {
            println!("{}", "実行中のプロセスはありません".yellow());
            return Ok(());
        }

        // PIDと情報を先に収集
        let to_kill: Vec<_> = running.iter().map(|p| (p.pid, p.branch.clone())).collect();

        println!("{}", "全プロセスを停止中...".blue());
        let mut killed = 0;
        let mut failed = 0;

        for (pid, branch) in to_kill {
            print!("  Killing PID {} ({})... ", pid, branch);

            // プロセスを見つけて停止
            if let Some(proc) = process_manager.processes.iter().find(|p| p.pid == pid) {
                match proc.kill() {
                    Ok(_) => {
                        println!("{}", "✓".green());
                        process_manager.remove_process(pid);
                        killed += 1;
                    }
                    Err(e) => {
                        println!("{} {}", "✗".red(), e);
                        failed += 1;
                    }
                }
            }
        }

        process_manager.save(&repo_root)?;

        println!(
            "\n{} {}個のプロセスを停止しました",
            "✅".green(),
            killed
        );
        if failed > 0 {
            eprintln!(
                "{} {}個のプロセスの停止に失敗しました",
                "⚠️".yellow(),
                failed
            );
        }
    } else if let Some(filter) = worktree_filter {
        // worktreeフィルタでkill
        let running = process_manager.running_processes();
        let to_kill: Vec<_> = running
            .iter()
            .filter(|p| p.worktree_path.contains(&filter) || p.branch.contains(&filter))
            .map(|p| (p.pid, p.branch.clone()))
            .collect();

        if to_kill.is_empty() {
            println!("{}", "該当するプロセスが見つかりませんでした".yellow());
            return Ok(());
        }

        println!(
            "{}個のプロセスを停止中...",
            to_kill.len()
        );

        let mut killed = 0;
        for (pid, branch) in to_kill {
            print!("  Killing PID {} ({})... ", pid, branch);

            if let Some(proc) = process_manager.processes.iter().find(|p| p.pid == pid) {
                match proc.kill() {
                    Ok(_) => {
                        println!("{}", "✓".green());
                        process_manager.remove_process(pid);
                        killed += 1;
                    }
                    Err(e) => {
                        println!("{} {}", "✗".red(), e);
                    }
                }
            }
        }

        process_manager.save(&repo_root)?;

        println!(
            "\n{} {}個のプロセスを停止しました",
            "✅".green(),
            killed
        );
    } else if let Some(pid) = pid {
        // 特定のPIDをkill
        let running = process_manager.running_processes();
        let proc_info = running
            .iter()
            .find(|p| p.pid == pid)
            .context(format!("PID {}のプロセスが見つかりませんでした", pid))?;

        let branch = proc_info.branch.clone();

        println!(
            "プロセスを停止中: PID {} ({})",
            pid,
            branch.green()
        );

        // プロセスを見つけてkill
        if let Some(proc) = process_manager.processes.iter().find(|p| p.pid == pid) {
            proc.kill()?;
        }

        process_manager.remove_process(pid);
        process_manager.save(&repo_root)?;

        println!("{}", "✅ プロセスを停止しました".green());
    } else {
        anyhow::bail!(
            "❌ PID、--all、または worktree フィルタを指定してください\n\n\
             使用例:\n\
             wtenv kill 12345          # 特定のPIDを停止\n\
             wtenv kill --all          # 全プロセスを停止\n\
             wtenv kill feature-a      # feature-a worktreeのプロセスを停止"
        );
    }

    Ok(())
}

/// 秒数を読みやすい形式にフォーマット
fn format_uptime(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {:02}s ago", minutes, secs)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {:02}m ago", hours, minutes)
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        format!("{}d {:02}h ago", days, hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "30s ago");
        assert_eq!(format_uptime(90), "1m 30s ago");
        assert_eq!(format_uptime(3661), "1h 01m ago");
        assert_eq!(format_uptime(90000), "1d 01h ago");
    }
}
