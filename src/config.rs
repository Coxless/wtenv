use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 設定ファイル名の検索順序
const CONFIG_FILE_NAMES: &[&str] = &[".worktree.yml", ".worktree.yaml"];

/// デフォルト設定テンプレート
const DEFAULT_CONFIG_TEMPLATE: &str = r#"version: 1

copy:
  - .env
  - .env.local

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "依存関係をインストール中..."
"#;

/// 設定ファイル構造体
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub copy: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default, rename = "postCreate")]
    pub post_create: Vec<PostCreateCommand>,
}

/// post-createコマンド
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostCreateCommand {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

/// 指定ディレクトリから設定ファイルを検索
pub fn find_config_file(dir: &Path) -> Option<PathBuf> {
    for name in CONFIG_FILE_NAMES {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// 設定ファイルを読み込む
pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("設定ファイルの読み込みに失敗しました: {}", path.display()))?;

    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("設定ファイルのパースに失敗しました: {}", path.display()))?;

    // バージョンチェック
    if config.version != 1 {
        anyhow::bail!(
            "❌ サポートされていない設定ファイルバージョンです: {}\n\n\
             現在サポートされているバージョン: 1",
            config.version
        );
    }

    Ok(config)
}

/// 設定ファイルを読み込む（見つからない場合はデフォルト設定）
pub fn load_config_or_default(dir: &Path) -> Result<Config> {
    match find_config_file(dir) {
        Some(path) => load_config(&path),
        None => Ok(Config::default()),
    }
}

/// デフォルト設定ファイルを作成
pub fn create_default_config(dir: &Path, force: bool) -> Result<PathBuf> {
    let config_path = dir.join(".worktree.yml");

    // 既に存在する場合
    if config_path.exists() && !force {
        anyhow::bail!(
            "❌ 設定ファイルは既に存在します: {}\n\n\
             上書きする場合は --force オプションを使用してください。",
            config_path.display()
        );
    }

    fs::write(&config_path, DEFAULT_CONFIG_TEMPLATE).with_context(|| {
        format!(
            "設定ファイルの作成に失敗しました: {}",
            config_path.display()
        )
    })?;

    Ok(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_config() {
        let content = "version: 1\ncopy:\n  - .env";
        let config: Config = serde_yaml::from_str(content).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.copy, vec![".env"]);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, 0);
        assert!(config.copy.is_empty());
        assert!(config.exclude.is_empty());
        assert!(config.post_create.is_empty());
    }

    #[test]
    fn test_parse_post_create_command() {
        let content = r#"
version: 1
postCreate:
  - command: npm install
    description: "Installing..."
  - command: npm build
    optional: true
"#;
        let config: Config = serde_yaml::from_str(content).unwrap();
        assert_eq!(config.post_create.len(), 2);
        assert_eq!(config.post_create[0].command, "npm install");
        assert_eq!(
            config.post_create[0].description,
            Some("Installing...".to_string())
        );
        assert!(!config.post_create[0].optional);
        assert!(config.post_create[1].optional);
    }
}
