mod commands;
mod config;
mod errors;
mod output;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

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
}

#[derive(Args)]
struct InitArgs {
    /// Overwrite existing configuration
    #[arg(short, long)]
    force: bool,
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
