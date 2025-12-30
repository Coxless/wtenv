use anyhow::{Context, Result};
use colored::Colorize;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

/// ファイルコピー結果
#[derive(Debug, Default)]
pub struct CopyResult {
    pub copied: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

/// globパターンにマッチするファイルを取得
pub fn expand_patterns(base_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        // ベースディレクトリからの相対パスまたは絶対パス
        let full_pattern = if pattern.starts_with('/') {
            pattern.clone()
        } else {
            base_dir.join(pattern).to_string_lossy().to_string()
        };

        match glob(&full_pattern) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(path) => {
                            // ファイルのみを追加（ディレクトリを除外）
                            if path.is_file() {
                                files.push(path);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", format!("⚠️  パターンマッチエラー: {}", e).yellow());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("⚠️  無効なglobパターン '{}': {}", pattern, e).yellow()
                );
            }
        }
    }

    // 重複を削除
    files.sort();
    files.dedup();

    Ok(files)
}

/// 除外パターンにマッチするファイルを除外
pub fn filter_excluded(files: Vec<PathBuf>, excludes: &[String]) -> Vec<PathBuf> {
    if excludes.is_empty() {
        return files;
    }

    files
        .into_iter()
        .filter(|path| {
            let path_str = path.to_string_lossy();

            for exclude_pattern in excludes {
                // 単純なパターンマッチング
                // グロブパターンとして解釈
                if let Ok(exclude_glob) = glob::Pattern::new(exclude_pattern) {
                    if exclude_glob.matches(&path_str) {
                        return false;
                    }
                }

                // ファイル名の部分一致
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy().contains(exclude_pattern) {
                        return false;
                    }
                }
            }

            true
        })
        .collect()
}

/// ファイルをコピー（個別エラーでも続行）
pub fn copy_files(files: &[PathBuf], source_dir: &Path, dest_dir: &Path) -> Result<CopyResult> {
    let mut result = CopyResult {
        copied: Vec::new(),
        failed: Vec::new(),
    };

    for file in files {
        // 相対パスを計算
        let relative_path = match file.strip_prefix(source_dir) {
            Ok(rel) => rel,
            Err(_) => {
                // source_dirからの相対パスではない場合、ファイル名のみ使用
                match file.file_name() {
                    Some(name) => Path::new(name),
                    None => {
                        result
                            .failed
                            .push((file.clone(), "ファイル名を取得できません".to_string()));
                        continue;
                    }
                }
            }
        };

        let dest_file = dest_dir.join(relative_path);

        // 親ディレクトリを作成
        if let Some(parent) = dest_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                result
                    .failed
                    .push((file.clone(), format!("ディレクトリ作成失敗: {}", e)));
                continue;
            }
        }

        // ファイルをコピー
        match fs::copy(file, &dest_file) {
            Ok(_) => {
                result.copied.push(relative_path.to_path_buf());
                println!("  {} {}", "✓".green(), relative_path.display());
            }
            Err(e) => {
                result
                    .failed
                    .push((file.clone(), format!("コピー失敗: {}", e)));
                eprintln!("  {} {}: {}", "✗".red(), relative_path.display(), e);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_filter_excluded() {
        let files = vec![
            PathBuf::from("/path/to/.env"),
            PathBuf::from("/path/to/.env.production"),
            PathBuf::from("/path/to/.env.local"),
        ];

        let excludes = vec![".env.production".to_string()];
        let filtered = filter_excluded(files, &excludes);

        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|p| !p.to_string_lossy().contains(".env.production")));
    }

    #[test]
    fn test_expand_patterns_empty() {
        let temp_dir = std::env::temp_dir();
        let patterns: Vec<String> = vec![];
        let result = expand_patterns(&temp_dir, &patterns).unwrap();
        assert!(result.is_empty());
    }
}
