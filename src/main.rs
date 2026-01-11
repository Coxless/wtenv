mod commands;
mod config;
mod errors;
mod output;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};

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
    /// Clear task progress history
    Clear(ClearArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Overwrite existing configuration
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct ClearArgs {
    /// Skip confirmation prompt
    #[arg(short, long)]
    force: bool,
}

/// Output options for controlling console output verbosity
#[derive(Clone, Copy)]
struct OutputOptions {
    #[allow(dead_code)]
    verbose: bool,
    quiet: bool,
}

impl OutputOptions {
    fn should_print(&self) -> bool {
        !self.quiet
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
        Commands::Clear(args) => cmd_clear(args, opts),
    }
}

/// init subcommand - creates Claude Code hooks (default behavior)
fn cmd_init(args: InitArgs, opts: OutputOptions) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

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

/// clear subcommand - clears task progress history
fn cmd_clear(args: ClearArgs, opts: OutputOptions) -> Result<()> {
    use commands::claude_task::TaskManager;

    let progress_dir = TaskManager::get_progress_dir();

    // Check if directory exists
    if !progress_dir.exists() {
        if opts.should_print() {
            println!("{}", output::OutputStyle::success("No task progress files to clear"));
        }
        return Ok(());
    }

    // Collect .jsonl files
    let files: Vec<_> = fs::read_dir(&progress_dir)
        .with_context(|| format!("Failed to read directory: {}", progress_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("jsonl"))
        .collect();

    if files.is_empty() {
        if opts.should_print() {
            println!("{}", output::OutputStyle::success("No task progress files to clear"));
        }
        return Ok(());
    }

    // Confirm unless --force
    if !args.force {
        println!(
            "Found {} task progress file(s) in {}",
            files.len(),
            output::OutputStyle::path(&progress_dir)
        );
        print!("Clear all files? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    // Delete files
    for file in &files {
        fs::remove_file(file)
            .with_context(|| format!("Failed to delete file: {}", file.display()))?;
    }

    if opts.should_print() {
        println!(
            "{}",
            output::OutputStyle::success(&format!("Cleared {} task progress file(s)", files.len()))
        );
    }

    Ok(())
}
