use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::worktree;

/// 環境変数ファイルのパターン
const ENV_FILE_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.development",
    ".env.production",
    ".env.test",
];

/// 環境変数ファイルの内容
#[derive(Debug, Clone)]
struct EnvFile {
    path: PathBuf,
    variables: HashMap<String, String>,
}

/// diff-envコマンドの実行
pub fn execute(worktree1: Option<String>, worktree2: Option<String>, all: bool) -> Result<()> {
    let _repo_root = worktree::get_repo_root()?;
    let worktrees = worktree::list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "worktreeが見つかりませんでした".yellow());
        return Ok(());
    }

    if all {
        // 全worktreeの環境変数を比較
        print_all_env_comparison(&worktrees)?;
    } else if let (Some(wt1), Some(wt2)) = (worktree1, worktree2) {
        // 2つのworktree間の比較
        let path1 = find_worktree_path(&worktrees, &wt1)?;
        let path2 = find_worktree_path(&worktrees, &wt2)?;

        print_env_diff(&path1, &path2, &wt1, &wt2)?;
    } else {
        anyhow::bail!(
            "❌ 2つのworktreeを指定するか、--allオプションを使用してください\n\n\
             使用例:\n\
             wtenv diff-env feature-a feature-b  # 2つのworktreeを比較\n\
             wtenv diff-env --all                # 全worktreeを比較"
        );
    }

    Ok(())
}

/// worktree名からパスを検索
fn find_worktree_path(worktrees: &[worktree::WorktreeInfo], name: &str) -> Result<PathBuf> {
    // ブランチ名で検索
    if let Some(wt) = worktrees
        .iter()
        .find(|w| w.branch.as_ref().map(|b| b.contains(name)).unwrap_or(false))
    {
        return Ok(wt.path.clone());
    }

    // パスで検索
    if let Some(wt) = worktrees
        .iter()
        .find(|w| w.path.to_string_lossy().contains(name))
    {
        return Ok(wt.path.clone());
    }

    anyhow::bail!("worktree '{}' が見つかりませんでした", name);
}

/// 環境変数ファイルを読み込む
fn load_env_files(worktree_path: &Path) -> Result<Vec<EnvFile>> {
    let mut env_files = Vec::new();

    for pattern in ENV_FILE_PATTERNS {
        let path = worktree_path.join(pattern);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("環境変数ファイルの読み込みに失敗: {}", path.display()))?;

        let variables = parse_env_file(&content);

        env_files.push(EnvFile {
            path: PathBuf::from(pattern),
            variables,
        });
    }

    Ok(env_files)
}

/// .envファイルをパース
fn parse_env_file(content: &str) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // コメントと空行をスキップ
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // KEY=VALUE形式をパース
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            variables.insert(key, value);
        }
    }

    variables
}

/// 2つのworktree間の環境変数diffを表示
fn print_env_diff(path1: &Path, path2: &Path, name1: &str, name2: &str) -> Result<()> {
    println!(
        "\n{} {} と {} の環境変数の違い:\n",
        "🔍".blue(),
        name1.cyan(),
        name2.cyan()
    );

    let env_files1 = load_env_files(path1)?;
    let env_files2 = load_env_files(path2)?;

    if env_files1.is_empty() && env_files2.is_empty() {
        println!("{}", "環境変数ファイルが見つかりませんでした".yellow());
        return Ok(());
    }

    let mut has_diff = false;

    // 各環境変数ファイルごとに比較
    for file_pattern in ENV_FILE_PATTERNS {
        let file1 = env_files1
            .iter()
            .find(|f| f.path.to_str() == Some(file_pattern));
        let file2 = env_files2
            .iter()
            .find(|f| f.path.to_str() == Some(file_pattern));

        if file1.is_none() && file2.is_none() {
            continue;
        }

        println!("{}:", file_pattern.bright_black());

        let (vars1, vars2) = match (&file1, &file2) {
            (Some(f1), Some(f2)) => (&f1.variables, &f2.variables),
            (None, Some(_)) => {
                println!("  {} にのみ存在", name2.cyan());
                has_diff = true;
                continue;
            }
            (Some(_), None) => {
                println!("  {} にのみ存在", name1.cyan());
                has_diff = true;
                continue;
            }
            (None, None) => unreachable!(), // Already handled above
        };

        // 全てのキーを収集
        let mut all_keys: Vec<_> = vars1.keys().chain(vars2.keys()).collect();
        all_keys.sort();
        all_keys.dedup();

        for key in all_keys {
            let val1 = vars1.get(key);
            let val2 = vars2.get(key);

            match (val1, val2) {
                (Some(v1), Some(v2)) if v1 != v2 => {
                    println!("  {}:", key.yellow());
                    println!("    {} {}", "-".red(), v1.red());
                    println!("    {} {}", "+".green(), v2.green());
                    has_diff = true;
                }
                (Some(v1), None) => {
                    println!("  {} ({}のみ)", key.yellow(), name1);
                    println!("    {} {}", "-".red(), v1.red());
                    has_diff = true;
                }
                (None, Some(v2)) => {
                    println!("  {} ({}のみ)", key.yellow(), name2);
                    println!("    {} {}", "+".green(), v2.green());
                    has_diff = true;
                }
                _ => {} // 同じ値なのでスキップ
            }
        }

        println!();
    }

    if !has_diff {
        println!("{}", "環境変数に違いはありません".green());
    }

    Ok(())
}

/// 全worktreeの環境変数を比較
fn print_all_env_comparison(worktrees: &[worktree::WorktreeInfo]) -> Result<()> {
    println!("\n{}\n", "全worktreeの環境変数比較:".bold());

    // 各worktreeの環境変数を収集
    let mut worktree_envs = Vec::new();

    for wt in worktrees {
        let branch_name = wt.branch.as_deref().unwrap_or("detached");
        let env_files = load_env_files(&wt.path)?;

        worktree_envs.push((branch_name, env_files));
    }

    // 全ての環境変数キーを収集
    // ファイル名 -> キー名 -> worktree名 -> 値
    let mut all_keys: HashMap<String, HashMap<String, HashMap<String, Option<String>>>> =
        HashMap::new();

    for (wt_name, env_files) in &worktree_envs {
        for env_file in env_files {
            let file_name = env_file.path.to_str().unwrap_or("unknown");

            for (key, value) in &env_file.variables {
                let file_map = all_keys.entry(file_name.to_string()).or_default();

                let key_entry = file_map.entry(key.clone()).or_default();

                key_entry.insert(wt_name.to_string(), Some(value.clone()));
            }
        }
    }

    if all_keys.is_empty() {
        println!("{}", "環境変数ファイルが見つかりませんでした".yellow());
        return Ok(());
    }

    // ファイルごとに表示
    for (file_name, keys) in all_keys {
        println!("{}:", file_name.bright_black());

        let mut sorted_keys: Vec<_> = keys.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let values = keys.get(key).unwrap();

            // すべてのworktreeで同じ値かチェック
            let unique_values: Vec<_> = values.values().filter_map(|v| v.as_ref()).collect();

            if unique_values.iter().all(|v| v == &unique_values[0]) {
                // 全て同じ値
                continue;
            }

            println!("  {}:", key.yellow());

            for (wt_name, _) in &worktree_envs {
                if let Some(Some(value)) = values.get(*wt_name) {
                    println!("    {}: {}", wt_name.cyan(), value);
                } else {
                    println!("    {}: {}", wt_name.cyan(), "(not set)".bright_black());
                }
            }

            println!();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_file() {
        let content = r#"
# Comment
API_KEY=secret123
DATABASE_URL="postgresql://localhost/db"
PORT=3000

# Another comment
DEBUG=true
"#;

        let vars = parse_env_file(content);

        assert_eq!(vars.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(
            vars.get("DATABASE_URL"),
            Some(&"postgresql://localhost/db".to_string())
        );
        assert_eq!(vars.get("PORT"), Some(&"3000".to_string()));
        assert_eq!(vars.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_env_file_with_quotes() {
        let content = r#"
SINGLE_QUOTE='value with spaces'
DOUBLE_QUOTE="another value"
NO_QUOTE=simple
"#;

        let vars = parse_env_file(content);

        assert_eq!(
            vars.get("SINGLE_QUOTE"),
            Some(&"value with spaces".to_string())
        );
        assert_eq!(vars.get("DOUBLE_QUOTE"), Some(&"another value".to_string()));
        assert_eq!(vars.get("NO_QUOTE"), Some(&"simple".to_string()));
    }
}
