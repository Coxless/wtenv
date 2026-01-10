mod commands;
mod config;
mod errors;
mod output;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ccmon")]
#[command(about = "Claude Code Monitor - Real-time task progress for parallel development", version, long_about = None)]
struct Cli {
    /// Verbose output mode
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Silent mode (suppress non-error output)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Claude Code hooks
    Init(InitArgs),
    /// Interactive TUI for Claude Code task progress
    Ui,
    /// Execute command with desktop notification
    Notify(NotifyArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Overwrite existing configuration
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct NotifyArgs {
    /// Command to execute
    command: String,
    /// Working directory (default: current directory)
    #[arg(short, long)]
    dir: Option<String>,
    /// Notify on success
    #[arg(long, default_value = "true")]
    notify_success: bool,
    /// Notify on error
    #[arg(long, default_value = "true")]
    notify_error: bool,
}

/// Output options
#[derive(Clone, Copy)]
struct OutputOptions {
    verbose: bool,
    quiet: bool,
}

impl OutputOptions {
    fn should_print(&self) -> bool {
        !self.quiet
    }

    #[allow(dead_code)]
    fn should_print_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let opts = OutputOptions {
        verbose: cli.verbose,
        quiet: cli.quiet,
    };

    match cli.command {
        Commands::Init(args) => cmd_init(args, opts),
        Commands::Ui => cmd_ui(),
        Commands::Notify(args) => cmd_notify(args),
    }
}

/// init subcommand - creates Claude Code hooks (default behavior)
fn cmd_init(args: InitArgs, opts: OutputOptions) -> Result<()> {
    let current_dir =
        std::env::current_dir().context("Failed to get current directory")?;

    if opts.should_print() {
        println!("{}", "Initializing Claude Code hooks...".blue());
    }

    let hook_files = config::create_claude_hooks(&current_dir, args.force)?;

    if opts.should_print() {
        println!("{}", "Created Claude Code hooks:".green());
        for file in &hook_files {
            println!(
                "  {} {}",
                "->".bright_black(),
                file.display().to_string().cyan()
            );
        }

        println!("\n{}", "Next steps:".blue());
        println!(
            "  {} Enable hooks for this project: settings are in .claude/settings.json",
            "1.".bright_black()
        );
        println!(
            "  {} Enable globally: cp .claude/settings.json ~/.claude/settings.json",
            "2.".bright_black()
        );
        println!(
            "  {} Customize hooks: edit scripts in .claude/hooks/",
            "3.".bright_black()
        );
    }

    Ok(())
}

/// ui subcommand
fn cmd_ui() -> Result<()> {
    commands::ui::execute()
}

/// notify subcommand
fn cmd_notify(args: NotifyArgs) -> Result<()> {
    let working_dir = if let Some(dir) = args.dir {
        PathBuf::from(dir)
    } else {
        std::env::current_dir()?
    };

    commands::notify::execute_with_notification(
        &args.command,
        &working_dir,
        args.notify_success,
        args.notify_error,
    )
}
