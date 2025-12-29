mod commands;
mod config;
mod copy;
mod worktree;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wtenv")]
#[command(about = "Git worktree environment manager", version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 新しいworktreeを作成
    Create(CreateArgs),
    /// worktree一覧を表示
    List,
    /// worktreeを削除
    Remove(RemoveArgs),
    /// 設定ファイルを初期化
    Init(InitArgs),
    /// 設定ファイルを表示
    Config,
}

#[derive(Args)]
struct CreateArgs {
    /// ブランチ名
    branch: String,
    /// worktreeパス（省略時: ../branch-name）
    path: Option<PathBuf>,
    /// ファイルコピーをスキップ
    #[arg(long)]
    no_copy: bool,
    /// post-createコマンドをスキップ
    #[arg(long)]
    no_post_create: bool,
    /// 設定ファイルパス
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Args)]
struct RemoveArgs {
    /// 削除するworktreeのパス
    path: PathBuf,
    /// 強制削除
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct InitArgs {
    /// 既存設定を上書き
    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => cmd_create(args),
        Commands::List => cmd_list(),
        Commands::Remove(args) => cmd_remove(args),
        Commands::Init(args) => cmd_init(args),
        Commands::Config => cmd_config(),
    }
}

/// createサブコマンド
fn cmd_create(args: CreateArgs) -> Result<()> {
    println!("{}", "🌲 worktreeを作成中...".blue());

    // 1. メインworktree確認
    let current_dir = std::env::current_dir()
        .context("カレントディレクトリの取得に失敗しました")?;
    let repo_root = worktree::get_repo_root()?;

    // 2. 設定ファイル読み込み
    let config_path = args.config.unwrap_or(repo_root.clone());
    let config = if config_path.is_file() {
        config::load_config(&config_path)?
    } else {
        config::load_config_or_default(&config_path)?
    };

    // 3. worktreeパス決定
    let worktree_path = args.path.unwrap_or_else(|| {
        let parent = repo_root.parent().unwrap_or(&repo_root);
        parent.join(&args.branch)
    });

    // 4. worktree作成
    println!("  ブランチ: {}", args.branch.cyan());
    println!("  パス: {}", worktree_path.display().to_string().cyan());

    worktree::create_worktree(&worktree_path, &args.branch)
        .context("worktreeの作成に失敗しました")?;

    println!("{}", "✓ worktreeを作成しました".green());

    // 5. ファイルコピー
    if !args.no_copy && !config.copy.is_empty() {
        println!("\n{}", "📋 環境ファイルをコピー中...".blue());

        let files = copy::expand_patterns(&repo_root, &config.copy)?;
        let files = copy::filter_excluded(files, &config.exclude);

        if files.is_empty() {
            println!("  {} コピーするファイルが見つかりませんでした", "ℹ".blue());
        } else {
            let result = copy::copy_files(&files, &repo_root, &worktree_path)?;

            println!(
                "\n{} {}個のファイルをコピーしました",
                "✅".green(),
                result.copied.len()
            );

            if !result.failed.is_empty() {
                eprintln!(
                    "{} {}個のファイルのコピーに失敗しました",
                    "⚠️ ".yellow(),
                    result.failed.len()
                );
            }
        }
    }

    // 6. post-createコマンド実行
    if !args.no_post_create && !config.post_create.is_empty() {
        commands::run_post_create_commands(&config.post_create, &worktree_path)?;
    }

    println!("\n{}", "✨ worktreeのセットアップが完了しました!".green().bold());
    println!("  移動するには: {}", format!("cd {}", worktree_path.display()).cyan());

    Ok(())
}

/// listサブコマンド
fn cmd_list() -> Result<()> {
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        println!("worktreeが見つかりませんでした");
        return Ok(());
    }

    for wt in worktrees {
        let main_marker = if wt.is_main { " (main)" } else { "" };
        let branch_display = wt
            .branch
            .as_ref()
            .map(|b| format!("[{}]", b))
            .unwrap_or_else(|| "[detached]".to_string());

        println!(
            "{} {}{}  {} {}",
            "📁".blue(),
            wt.path.display().to_string().cyan(),
            main_marker.bright_black(),
            branch_display.green(),
            wt.commit[..7.min(wt.commit.len())].bright_black()
        );
    }

    Ok(())
}

/// removeサブコマンド
fn cmd_remove(args: RemoveArgs) -> Result<()> {
    println!("{}", "🗑️  worktreeを削除中...".blue());
    println!("  パス: {}", args.path.display().to_string().cyan());

    worktree::remove_worktree(&args.path, args.force)?;

    println!("{}", "✓ worktreeを削除しました".green());

    Ok(())
}

/// initサブコマンド
fn cmd_init(args: InitArgs) -> Result<()> {
    let current_dir = std::env::current_dir()
        .context("カレントディレクトリの取得に失敗しました")?;

    println!("{}", "📝 設定ファイルを作成中...".blue());

    let config_path = config::create_default_config(&current_dir, args.force)?;

    println!(
        "{} {}",
        "✅ 設定ファイルを作成しました:".green(),
        config_path.display().to_string().cyan()
    );

    Ok(())
}

/// configサブコマンド
fn cmd_config() -> Result<()> {
    let current_dir = std::env::current_dir()
        .context("カレントディレクトリの取得に失敗しました")?;

    match config::find_config_file(&current_dir) {
        Some(path) => {
            println!("{}", "📄 設定ファイル:".blue());
            println!("  パス: {}", path.display().to_string().cyan());
            println!();

            let content = std::fs::read_to_string(&path)
                .context("設定ファイルの読み込みに失敗しました")?;

            println!("{}", content);

            // バリデーション
            match config::load_config(&path) {
                Ok(_) => println!("{}", "✅ 設定ファイルは有効です".green()),
                Err(e) => {
                    eprintln!("{}", "❌ 設定ファイルにエラーがあります:".red());
                    eprintln!("  {}", e);
                }
            }
        }
        None => {
            println!("{}", "ℹ  設定ファイルが見つかりませんでした".blue());
            println!("  'wtenv init' で設定ファイルを作成できます");
        }
    }

    Ok(())
}
