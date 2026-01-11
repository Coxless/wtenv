use std::path::Path;

/// Gitエラーメッセージのフォーマット
#[allow(dead_code)]
pub fn format_git_error(operation: &str, stderr: &str) -> String {
    format!(
        "❌ Git操作が失敗しました: {}\n\n\
         エラー内容:\n{}\n\n\
         ヒント:\n\
         - gitがインストールされているか確認してください\n\
         - Gitリポジトリ内で実行しているか確認してください\n\
         - 'git status' で状態を確認してください",
        operation,
        stderr.trim()
    )
}

/// ファイル操作エラーメッセージのフォーマット
#[allow(dead_code)]
pub fn format_file_error(operation: &str, path: &Path, error: &std::io::Error) -> String {
    format!(
        "❌ ファイル操作が失敗しました: {}\n\n\
         パス: {}\n\
         エラー: {}\n\n\
         ヒント:\n\
         - ファイル/ディレクトリが存在するか確認してください\n\
         - 書き込み権限があるか確認してください",
        operation,
        path.display(),
        error
    )
}

/// 設定ファイルエラーメッセージのフォーマット
#[allow(dead_code)]
pub fn format_config_error(path: &Path, details: &str) -> String {
    format!(
        "❌ 設定ファイルエラー\n\n\
         パス: {}\n\
         詳細: {}\n\n\
         ヒント:\n\
         - 'ccmon init --force' で設定ファイルを再作成できます",
        path.display(),
        details
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_git_error() {
        let result = format_git_error("checkout", "fatal: not a git repository");
        assert!(result.contains("checkout"));
        assert!(result.contains("fatal"));
    }

    #[test]
    fn test_format_file_error() {
        let path = PathBuf::from("/test/file.txt");
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let result = format_file_error("読み込み", &path, &error);
        assert!(result.contains("/test/file.txt"));
    }

    #[test]
    fn test_format_config_error() {
        let path = PathBuf::from(".worktree.yml");
        let result = format_config_error(&path, "invalid syntax");
        assert!(result.contains(".worktree.yml"));
        assert!(result.contains("invalid syntax"));
    }
}
