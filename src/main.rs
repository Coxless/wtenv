mod commands;
mod config;
mod copy;
mod errors;
mod interactive;
mod output;
mod worktree;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wtenv")]
#[command(about = "Git worktree environment manager", version, long_about = None)]
struct Cli {
    /// 詳細出力モード
    #[arg(short, long, global = true)]
    verbose: bool,

    /// サイレントモード（エラー以外の出力を抑制）
    #[arg(short, long, global = true)]
    quiet: bool,

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
    /// worktree状態の詳細を表示
    Status,
    /// 実行中プロセス一覧を表示
    Ps(PsArgs),
    /// プロセスを停止
    Kill(KillArgs),
    /// worktree間の環境変数を比較
    DiffEnv(DiffEnvArgs),
    /// インタラクティブTUI
    Ui,
    /// worktreeの分析
    Analyze(AnalyzeArgs),
    /// 不要なworktreeをクリーンアップ
    Clean(CleanArgs),
    /// コマンド実行と通知
    Notify(NotifyArgs),
}

#[derive(Args)]
struct PsArgs {
    /// worktreeフィルタ（ブランチ名またはパス）
    filter: Option<String>,
}

#[derive(Args)]
struct KillArgs {
    /// プロセスID
    pid: Option<u32>,
    /// 全プロセスを停止
    #[arg(long)]
    all: bool,
    /// worktreeフィルタ（ブランチ名またはパス）
    filter: Option<String>,
}

#[derive(Args)]
struct DiffEnvArgs {
    /// 1つ目のworktree（ブランチ名またはパス）
    worktree1: Option<String>,
    /// 2つ目のworktree（ブランチ名またはパス）
    worktree2: Option<String>,
    /// 全worktreeの環境変数を比較
    #[arg(long)]
    all: bool,
}

#[derive(Args)]
struct AnalyzeArgs {
    /// 詳細情報を表示
    #[arg(short, long)]
    detailed: bool,
}

#[derive(Args)]
struct CleanArgs {
    /// ドライラン（実際には削除しない）
    #[arg(long)]
    dry_run: bool,
    /// マージ済みブランチのみ削除
    #[arg(long)]
    merged_only: bool,
    /// 指定日数以上更新されていないworktreeを削除
    #[arg(long)]
    stale_days: Option<u64>,
    /// 確認なしで削除
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct NotifyArgs {
    /// 実行するコマンド
    command: String,
    /// 作業ディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(short, long)]
    dir: Option<String>,
    /// 成功時に通知
    #[arg(long, default_value = "true")]
    notify_success: bool,
    /// エラー時に通知
    #[arg(long, default_value = "true")]
    notify_error: bool,
}

#[derive(Args)]
struct CreateArgs {
    /// ブランチ名（省略時は対話モード）
    branch: Option<String>,
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

/// 出力設定
#[derive(Clone, Copy)]
struct OutputOptions {
    verbose: bool,
    quiet: bool,
}

impl OutputOptions {
    fn should_print(&self) -> bool {
        !self.quiet
    }

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
        Commands::Create(args) => cmd_create(args, opts),
        Commands::List => cmd_list(opts),
        Commands::Remove(args) => cmd_remove(args, opts),
        Commands::Init(args) => cmd_init(args, opts),
        Commands::Config => cmd_config(opts),
        Commands::Status => cmd_status(opts),
        Commands::Ps(args) => cmd_ps(args),
        Commands::Kill(args) => cmd_kill(args),
        Commands::DiffEnv(args) => cmd_diff_env(args),
        Commands::Ui => cmd_ui(),
        Commands::Analyze(args) => cmd_analyze(args),
        Commands::Clean(args) => cmd_clean(args),
        Commands::Notify(args) => cmd_notify(args),
    }
}

/// createサブコマンド
fn cmd_create(args: CreateArgs, opts: OutputOptions) -> Result<()> {
    if opts.should_print() {
        println!("{}", "🌲 worktreeを作成中...".blue());
    }

    // 1. メインworktree確認
    let _current_dir =
        std::env::current_dir().context("カレントディレクトリの取得に失敗しました")?;
    let repo_root = worktree::get_repo_root()?;

    if opts.should_print_verbose() {
        println!(
            "  {} リポジトリルート: {}",
            "→".bright_black(),
            repo_root.display()
        );
    }

    // 2. 設定ファイル読み込み
    let config_path = args.config.unwrap_or(repo_root.clone());
    let config = if config_path.is_file() {
        config::load_config(&config_path)?
    } else {
        config::load_config_or_default(&config_path)?
    };

    // 3. ブランチ名を取得（対話モード対応）
    let is_interactive = args.branch.is_none();
    let branch = match args.branch {
        Some(b) => b,
        None => interactive::prompt_branch_name()?,
    };

    // 4. worktreeパス決定（対話モード対応）
    let default_path = repo_root.parent().unwrap_or(&repo_root).join(&branch);
    let worktree_path = match args.path {
        Some(p) => p,
        None => {
            // ブランチ名が引数で指定された場合はデフォルトパスを使用
            // 対話モードの場合はパスも対話的に確認
            if is_interactive {
                interactive::prompt_worktree_path(&default_path.to_string_lossy())?
            } else {
                default_path
            }
        }
    };

    // 5. worktree作成
    if opts.should_print() {
        println!("  ブランチ: {}", branch.cyan());
        println!("  パス: {}", worktree_path.display().to_string().cyan());
    }

    worktree::create_worktree(&worktree_path, &branch).context("worktreeの作成に失敗しました")?;

    if opts.should_print() {
        println!("{}", "✓ worktreeを作成しました".green());
    }

    // 6. ファイルコピー
    if !args.no_copy && !config.copy.is_empty() {
        if opts.should_print() {
            println!("\n{}", "📋 環境ファイルをコピー中...".blue());
        }

        let files = copy::expand_patterns(&repo_root, &config.copy)?;
        let files = copy::filter_excluded(files, &config.exclude);

        if files.is_empty() {
            if opts.should_print() {
                println!("  {} コピーするファイルが見つかりませんでした", "ℹ".blue());
            }
        } else {
            // プログレスバーを使用（quietモードでなければ）
            let result = if opts.should_print() && files.len() > 3 {
                let pb = output::create_progress_bar(files.len() as u64, "コピー中...");
                let mut copied = Vec::new();
                let mut failed = Vec::new();

                for file in &files {
                    let relative_path = file.strip_prefix(&repo_root).unwrap_or(file);
                    let dest_file = worktree_path.join(relative_path);

                    if let Some(parent) = dest_file.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }

                    match std::fs::copy(file, &dest_file) {
                        Ok(_) => copied.push(relative_path.to_path_buf()),
                        Err(e) => failed.push((file.clone(), e.to_string())),
                    }
                    pb.inc(1);
                }
                pb.finish_and_clear();

                copy::CopyResult { copied, failed }
            } else {
                copy::copy_files(&files, &repo_root, &worktree_path)?
            };

            if opts.should_print() {
                println!(
                    "{} {}個のファイルをコピーしました",
                    "✅".green(),
                    result.copied.len()
                );
            }

            if !result.failed.is_empty() {
                eprintln!(
                    "{} {}個のファイルのコピーに失敗しました",
                    "⚠️ ".yellow(),
                    result.failed.len()
                );
                if opts.should_print_verbose() {
                    for (path, error) in &result.failed {
                        eprintln!("  {} {}: {}", "✗".red(), path.display(), error);
                    }
                }
            }
        }
    }

    // 7. post-createコマンド実行
    if !args.no_post_create && !config.post_create.is_empty() {
        commands::run_post_create_commands(&config.post_create, &worktree_path)?;
    }

    if opts.should_print() {
        println!(
            "\n{}",
            "✨ worktreeのセットアップが完了しました!".green().bold()
        );
        println!(
            "  移動するには: {}",
            format!("cd {}", worktree_path.display()).cyan()
        );
    }

    Ok(())
}

/// listサブコマンド
fn cmd_list(opts: OutputOptions) -> Result<()> {
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        if opts.should_print() {
            println!("worktreeが見つかりませんでした");
        }
        return Ok(());
    }

    for wt in worktrees {
        let main_marker = if wt.is_main { " (main)" } else { "" };
        let branch_display = wt
            .branch
            .as_ref()
            .map(|b| format!("[{}]", b))
            .unwrap_or_else(|| "[detached]".to_string());

        if opts.should_print_verbose() {
            // 詳細モード: より多くの情報を表示
            println!(
                "{} {}{}\n    ブランチ: {}\n    コミット: {}",
                "📁".blue(),
                wt.path.display().to_string().cyan(),
                main_marker.bright_black(),
                branch_display.green(),
                wt.commit.bright_black()
            );
        } else {
            println!(
                "{} {}{}  {} {}",
                "📁".blue(),
                wt.path.display().to_string().cyan(),
                main_marker.bright_black(),
                branch_display.green(),
                wt.commit[..7.min(wt.commit.len())].bright_black()
            );
        }
    }

    Ok(())
}

/// removeサブコマンド
fn cmd_remove(args: RemoveArgs, opts: OutputOptions) -> Result<()> {
    // --forceがない場合は確認ダイアログを表示
    if !args.force && !interactive::confirm_remove(&args.path)? {
        if opts.should_print() {
            println!("{}", "キャンセルされました".yellow());
        }
        return Ok(());
    }

    if opts.should_print() {
        println!("{}", "🗑️  worktreeを削除中...".blue());
        println!("  パス: {}", args.path.display().to_string().cyan());
    }

    worktree::remove_worktree(&args.path, args.force)?;

    if opts.should_print() {
        println!("{}", "✓ worktreeを削除しました".green());
    }

    Ok(())
}

/// initサブコマンド
fn cmd_init(args: InitArgs, opts: OutputOptions) -> Result<()> {
    let current_dir =
        std::env::current_dir().context("カレントディレクトリの取得に失敗しました")?;

    let config_path = current_dir.join(".worktree.yml");

    // --forceがない場合で既存ファイルがある場合は確認
    let force = if config_path.exists() && !args.force {
        if !interactive::confirm_overwrite(&config_path)? {
            if opts.should_print() {
                println!("{}", "キャンセルされました".yellow());
            }
            return Ok(());
        }
        true // 確認済みなので強制上書き
    } else {
        args.force
    };

    if opts.should_print() {
        println!("{}", "📝 設定ファイルを作成中...".blue());
    }

    let created_path = config::create_default_config(&current_dir, force)?;

    if opts.should_print() {
        println!(
            "{} {}",
            "✅ 設定ファイルを作成しました:".green(),
            created_path.display().to_string().cyan()
        );
    }

    Ok(())
}

/// configサブコマンド
fn cmd_config(opts: OutputOptions) -> Result<()> {
    let current_dir =
        std::env::current_dir().context("カレントディレクトリの取得に失敗しました")?;

    match config::find_config_file(&current_dir) {
        Some(path) => {
            println!("{}", "📄 設定ファイル:".blue());
            println!("  パス: {}", path.display().to_string().cyan());
            println!();

            let content =
                std::fs::read_to_string(&path).context("設定ファイルの読み込みに失敗しました")?;

            println!("{}", content);

            // バリデーション
            match config::load_config(&path) {
                Ok(cfg) => {
                    println!("{}", "✅ 設定ファイルは有効です".green());
                    if opts.should_print_verbose() {
                        println!("\n{}", "詳細情報:".bright_black());
                        println!("  バージョン: {}", cfg.version);
                        println!("  コピー対象: {} パターン", cfg.copy.len());
                        println!("  除外対象: {} パターン", cfg.exclude.len());
                        println!("  post-createコマンド: {} 個", cfg.post_create.len());
                    }
                }
                Err(e) => {
                    eprintln!("{}", "❌ 設定ファイルにエラーがあります:".red());
                    eprintln!("  {}", e);
                }
            }
        }
        None => {
            if opts.should_print() {
                println!("{}", "ℹ  設定ファイルが見つかりませんでした".blue());
                println!("  'wtenv init' で設定ファイルを作成できます");
            }
        }
    }

    Ok(())
}

/// statusサブコマンド
fn cmd_status(opts: OutputOptions) -> Result<()> {
    commands::status::execute(opts.should_print_verbose())
}

/// psサブコマンド
fn cmd_ps(args: PsArgs) -> Result<()> {
    commands::ps::execute(args.filter)
}

/// killサブコマンド
fn cmd_kill(args: KillArgs) -> Result<()> {
    commands::ps::kill(args.pid, args.all, args.filter)
}

/// diff-envサブコマンド
fn cmd_diff_env(args: DiffEnvArgs) -> Result<()> {
    commands::diff_env::execute(args.worktree1, args.worktree2, args.all)
}

/// uiサブコマンド
fn cmd_ui() -> Result<()> {
    commands::ui::execute()
}

/// analyzeサブコマンド
fn cmd_analyze(args: AnalyzeArgs) -> Result<()> {
    commands::analyze::execute(args.detailed)
}

/// cleanサブコマンド
fn cmd_clean(args: CleanArgs) -> Result<()> {
    use crate::commands::clean::CleanOptions;

    let opts = CleanOptions {
        dry_run: args.dry_run,
        merged_only: args.merged_only,
        stale_days: args.stale_days,
        force: args.force,
    };

    commands::clean::execute(opts)
}

/// notifyサブコマンド
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
