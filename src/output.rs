use colored::*;
use std::path::Path;

/// 出力スタイルユーティリティ
#[allow(dead_code)]
pub struct OutputStyle;

#[allow(dead_code)]
impl OutputStyle {
    pub fn success(msg: &str) -> ColoredString {
        format!("✓ {}", msg).green()
    }

    pub fn error(msg: &str) -> ColoredString {
        format!("✗ {}", msg).red()
    }

    pub fn warning(msg: &str) -> ColoredString {
        format!("⚠ {}", msg).yellow()
    }

    pub fn info(msg: &str) -> ColoredString {
        format!("ℹ {}", msg).blue()
    }

    pub fn path(path: &Path) -> ColoredString {
        path.display().to_string().cyan()
    }

    pub fn command(cmd: &str) -> ColoredString {
        cmd.bright_black()
    }

    pub fn header(msg: &str) -> ColoredString {
        msg.bold().blue()
    }
}

/// プログレスバー作成
pub fn create_progress_bar(len: u64, msg: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(msg.to_string());
    pb
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_output_style_success() {
        let result = OutputStyle::success("test");
        assert!(result.to_string().contains("test"));
    }

    #[test]
    fn test_output_style_path() {
        let path = PathBuf::from("/test/path");
        let result = OutputStyle::path(&path);
        assert!(result.to_string().contains("/test/path"));
    }
}
