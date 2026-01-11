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
#[allow(dead_code)]
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

/// Format byte size into human-readable format (B, KB, MB, GB)
///
/// Converts raw byte counts into a readable string with appropriate units.
/// Uses binary units (1 KB = 1024 bytes).
///
/// # Arguments
///
/// * `bytes` - The number of bytes to format
///
/// # Examples
///
/// ```
/// use wtenv::output::format_size;
/// assert_eq!(format_size(1024), "1.00 KB");
/// assert_eq!(format_size(1048576), "1.00 MB");
/// assert_eq!(format_size(500), "500 B");
/// ```
#[allow(dead_code)]
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
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

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_size_decimal() {
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }
}
