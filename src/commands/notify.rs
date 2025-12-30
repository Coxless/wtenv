use anyhow::Result;
use colored::Colorize;
use notify_rust::Notification;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use crate::commands::claude_task::{ClaudeTask, TaskStatus};

/// 通知タイプ
#[derive(Debug, Clone, Copy)]
pub enum NotifyType {
    Success,
    Error,
    #[allow(dead_code)]
    Info,
}

/// 通知オプション
pub struct NotifyOptions {
    pub title: String,
    pub message: String,
    pub notify_type: NotifyType,
}

/// コマンド実行と通知
pub fn execute_with_notification(
    command: &str,
    working_dir: &Path,
    notify_on_success: bool,
    notify_on_error: bool,
) -> Result<()> {
    println!("{}", format!("🚀 Executing: {}", command).cyan());
    println!();

    let start = Instant::now();

    // コマンドを実行
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("bash");
        c.args(["-c", command]);
        c
    };

    cmd.current_dir(working_dir);

    let output = cmd.output()?;
    let duration = start.elapsed();

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 結果を表示
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    println!();
    println!(
        "{}",
        format!("⏱️  Duration: {:.2}s", duration.as_secs_f64()).bright_black()
    );

    // 通知を送信
    if success && notify_on_success {
        send_notification(NotifyOptions {
            title: "Command Succeeded".to_string(),
            message: format!("{} completed in {:.2}s", command, duration.as_secs_f64()),
            notify_type: NotifyType::Success,
        })?;

        println!("{}", "✅ Command succeeded".green().bold());
    } else if !success && notify_on_error {
        send_notification(NotifyOptions {
            title: "Command Failed".to_string(),
            message: format!("{} failed after {:.2}s", command, duration.as_secs_f64()),
            notify_type: NotifyType::Error,
        })?;

        println!("{}", "❌ Command failed".red().bold());
    }

    if !success {
        anyhow::bail!("Command failed with exit code: {:?}", output.status.code());
    }

    Ok(())
}

/// デスクトップ通知を送信
pub fn send_notification(opts: NotifyOptions) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        // Linuxではない場合は単純なログ出力のみ
        let icon = match opts.notify_type {
            NotifyType::Success => "✅",
            NotifyType::Error => "❌",
            NotifyType::Info => "ℹ️",
        };

        println!(
            "{} {}: {}",
            icon,
            opts.title.bold(),
            opts.message.bright_black()
        );

        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let mut notification = Notification::new();
        notification.summary(&opts.title);
        notification.body(&opts.message);

        // アイコンとurgencyを設定
        match opts.notify_type {
            NotifyType::Success => {
                notification.icon("dialog-information");
                notification.urgency(notify_rust::Urgency::Normal);
            }
            NotifyType::Error => {
                notification.icon("dialog-error");
                notification.urgency(notify_rust::Urgency::Critical);
            }
            NotifyType::Info => {
                notification.icon("dialog-information");
                notification.urgency(notify_rust::Urgency::Low);
            }
        }

        // タイムアウトを設定
        notification.timeout(5000); // 5秒

        // 通知を表示
        match notification.show() {
            Ok(_) => {
                let icon = match opts.notify_type {
                    NotifyType::Success => "✅",
                    NotifyType::Error => "❌",
                    NotifyType::Info => "ℹ️",
                };

                println!(
                    "{} {}: {}",
                    icon,
                    opts.title.bold(),
                    opts.message.bright_black()
                );

                Ok(())
            }
            Err(e) => {
                // 通知が失敗してもエラーにはしない（通知システムが利用できない環境もある）
                eprintln!(
                    "{}",
                    format!("⚠️  Desktop notification unavailable: {}", e).yellow()
                );
                Ok(())
            }
        }
    }
}

/// ビルド完了通知
#[allow(dead_code)]
pub fn notify_build_complete(success: bool, duration_secs: f64) -> Result<()> {
    let opts = if success {
        NotifyOptions {
            title: "Build Complete".to_string(),
            message: format!("Build succeeded in {:.2}s", duration_secs),
            notify_type: NotifyType::Success,
        }
    } else {
        NotifyOptions {
            title: "Build Failed".to_string(),
            message: format!("Build failed after {:.2}s", duration_secs),
            notify_type: NotifyType::Error,
        }
    };

    send_notification(opts)
}

/// テスト完了通知
#[allow(dead_code)]
pub fn notify_test_complete(success: bool, duration_secs: f64) -> Result<()> {
    let opts = if success {
        NotifyOptions {
            title: "Tests Passed".to_string(),
            message: format!("All tests passed in {:.2}s", duration_secs),
            notify_type: NotifyType::Success,
        }
    } else {
        NotifyOptions {
            title: "Tests Failed".to_string(),
            message: format!("Some tests failed after {:.2}s", duration_secs),
            notify_type: NotifyType::Error,
        }
    };

    send_notification(opts)
}

/// Claude Code タスク状態変化通知
#[allow(dead_code)]
pub fn notify_claude_task_status(task: &ClaudeTask) -> Result<()> {
    let worktree_name = std::path::Path::new(&task.worktree_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let (title, message, notify_type) = match task.status {
        TaskStatus::Completed => (
            format!("✅ Task Completed - {}", worktree_name),
            format!(
                "Claude Code session completed in {}",
                task.duration_string()
            ),
            NotifyType::Success,
        ),
        TaskStatus::WaitingUser => (
            format!("⏸️  Response Needed - {}", worktree_name),
            format!(
                "Claude is waiting for your response after {}",
                task.duration_string()
            ),
            NotifyType::Info,
        ),
        TaskStatus::Error => (
            format!("❌ Task Failed - {}", worktree_name),
            "Claude Code task encountered an error".to_string(),
            NotifyType::Error,
        ),
        TaskStatus::InProgress => {
            // Don't notify for in-progress by default
            return Ok(());
        }
    };

    send_notification(NotifyOptions {
        title,
        message,
        notify_type,
    })
}

/// Claude Code タスク完了通知（成功時のみ）
#[allow(dead_code)]
pub fn notify_claude_task_complete(worktree: &str, duration_secs: f64) -> Result<()> {
    let opts = NotifyOptions {
        title: format!("✅ Claude Task Complete - {}", worktree),
        message: format!("Session completed in {:.1}s", duration_secs),
        notify_type: NotifyType::Success,
    };

    send_notification(opts)
}

/// Claude Code ユーザー応答待ち通知
#[allow(dead_code)]
pub fn notify_claude_needs_response(worktree: &str) -> Result<()> {
    let opts = NotifyOptions {
        title: format!("⏸️  Claude Needs Response - {}", worktree),
        message: "Claude is waiting for your input".to_string(),
        notify_type: NotifyType::Info,
    };

    send_notification(opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_options() {
        let opts = NotifyOptions {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            notify_type: NotifyType::Info,
        };

        assert_eq!(opts.title, "Test");
        assert_eq!(opts.message, "Test message");
    }
}
