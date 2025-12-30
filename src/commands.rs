use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::config::PostCreateCommand;

/// コマンド実行結果
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

/// プラットフォームごとのシェルコマンド作成
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

/// コマンドを実行
pub fn run_command(
    command: &str,
    working_dir: &Path,
    description: Option<&str>,
) -> Result<CommandResult> {
    let start = Instant::now();

    let mut cmd = shell_command(command);
    cmd.current_dir(working_dir);

    let output = cmd
        .output()
        .with_context(|| format!("コマンドの実行に失敗しました: {}", command))?;

    let duration = start.elapsed();

    Ok(CommandResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration,
    })
}

/// スピナー付きでコマンドを実行
pub fn run_with_spinner(
    command: &str,
    working_dir: &Path,
    description: &str,
) -> Result<CommandResult> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(description.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));

    let result = run_command(command, working_dir, Some(description))?;

    spinner.finish_and_clear();

    if result.success {
        println!(
            "  {} {} ({:.2}s)",
            "✓".green(),
            description,
            result.duration.as_secs_f64()
        );
    } else {
        eprintln!(
            "  {} {} ({:.2}s)",
            "✗".red(),
            description,
            result.duration.as_secs_f64()
        );
        if !result.stderr.is_empty() {
            eprintln!("     {}", result.stderr.trim());
        }
    }

    Ok(result)
}

/// post-createコマンドを順次実行
pub fn run_post_create_commands(commands: &[PostCreateCommand], working_dir: &Path) -> Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    println!("\n{}", "📦 post-createコマンドを実行中...".blue());

    for (i, cmd_config) in commands.iter().enumerate() {
        let description = cmd_config
            .description
            .as_deref()
            .unwrap_or(&cmd_config.command);

        println!(
            "\n[{}/{}] {}",
            i + 1,
            commands.len(),
            description.bright_black()
        );

        let result = run_with_spinner(&cmd_config.command, working_dir, description)?;

        if !result.success {
            if cmd_config.optional {
                eprintln!(
                    "  {} {}",
                    "⚠️ ".yellow(),
                    "オプションのコマンドが失敗しましたが続行します".yellow()
                );
            } else {
                anyhow::bail!(
                    "❌ コマンドが失敗しました: {}\n\n\
                     コマンド: {}\n\
                     終了コード: 失敗\n\
                     標準エラー出力:\n{}",
                    description,
                    cmd_config.command,
                    result.stderr.trim()
                );
            }
        }
    }

    println!("\n{}", "✨ post-createコマンドが完了しました".green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_run_command_success() {
        let result = run_command("echo test", &env::current_dir().unwrap(), None).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("test"));
    }

    #[test]
    fn test_run_command_failure() {
        let result = run_command("exit 1", &env::current_dir().unwrap(), None).unwrap();
        assert!(!result.success);
    }
}
