use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, System};

/// プロセス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub worktree_path: String,
    pub branch: String,
    pub pid: u32,
    pub command: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub started_at: DateTime<Utc>,
    pub cwd: String,
}

/// プロセスマネージャー
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessManager {
    pub processes: Vec<ProcessInfo>,
}

impl ProcessInfo {
    /// 新しいプロセス情報を作成
    pub fn new(
        worktree_path: impl Into<String>,
        branch: impl Into<String>,
        pid: u32,
        command: impl Into<String>,
        cwd: impl Into<String>,
    ) -> Self {
        Self {
            worktree_path: worktree_path.into(),
            branch: branch.into(),
            pid,
            command: command.into(),
            started_at: Utc::now(),
            cwd: cwd.into(),
        }
    }

    /// プロセスが実行中かチェック
    pub fn is_running(&self) -> bool {
        let mut sys = System::new();
        use sysinfo::ProcessesToUpdate;
        sys.refresh_processes(ProcessesToUpdate::All, true);
        sys.process(Pid::from_u32(self.pid)).is_some()
    }

    /// プロセス開始からの経過時間（秒）
    pub fn uptime_secs(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }

    /// プロセスをkill
    #[cfg(unix)]
    pub fn kill(&self) -> Result<()> {
        use std::process::Command;
        let output = Command::new("kill")
            .args([self.pid.to_string().as_str()])
            .output()
            .context("killコマンドの実行に失敗しました")?;

        if !output.status.success() {
            anyhow::bail!(
                "プロセスの停止に失敗しました (PID: {})",
                self.pid
            );
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn kill(&self) -> Result<()> {
        use std::process::Command;
        let output = Command::new("taskkill")
            .args(["/PID", &self.pid.to_string(), "/F"])
            .output()
            .context("taskkillコマンドの実行に失敗しました")?;

        if !output.status.success() {
            anyhow::bail!(
                "プロセスの停止に失敗しました (PID: {})",
                self.pid
            );
        }
        Ok(())
    }
}

impl ProcessManager {
    /// 新しいプロセスマネージャーを作成
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }

    /// プロセス情報ファイルのパスを取得
    fn process_file_path(repo_root: &Path) -> PathBuf {
        repo_root.join(".worktree").join("processes.json")
    }

    /// プロセス情報を読み込み
    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = Self::process_file_path(repo_root);

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)
            .context("プロセス情報ファイルの読み込みに失敗しました")?;

        let manager: ProcessManager = serde_json::from_str(&content)
            .context("プロセス情報のパースに失敗しました")?;

        Ok(manager)
    }

    /// プロセス情報を保存
    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let worktree_dir = repo_root.join(".worktree");
        if !worktree_dir.exists() {
            fs::create_dir_all(&worktree_dir)
                .context(".worktreeディレクトリの作成に失敗しました")?;
        }

        let path = Self::process_file_path(repo_root);
        let content = serde_json::to_string_pretty(self)
            .context("プロセス情報のシリアライズに失敗しました")?;

        fs::write(&path, content)
            .context("プロセス情報ファイルの書き込みに失敗しました")?;

        Ok(())
    }

    /// プロセスを追加
    pub fn add_process(&mut self, process: ProcessInfo) {
        self.processes.push(process);
    }

    /// プロセスを削除（PID指定）
    pub fn remove_process(&mut self, pid: u32) -> bool {
        let len = self.processes.len();
        self.processes.retain(|p| p.pid != pid);
        self.processes.len() < len
    }

    /// worktreeのプロセスを削除
    pub fn remove_worktree_processes(&mut self, worktree_path: &str) {
        self.processes.retain(|p| p.worktree_path != worktree_path);
    }

    /// 実行中のプロセスのみにフィルタ
    pub fn cleanup_dead_processes(&mut self) {
        self.processes.retain(|p| p.is_running());
    }

    /// 実行中のプロセスを取得
    pub fn running_processes(&self) -> Vec<&ProcessInfo> {
        self.processes.iter().filter(|p| p.is_running()).collect()
    }

    /// worktreeごとのプロセスを取得
    pub fn processes_by_worktree(&self, worktree_path: &str) -> Vec<&ProcessInfo> {
        self.processes
            .iter()
            .filter(|p| p.worktree_path == worktree_path && p.is_running())
            .collect()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_creation() {
        let info = ProcessInfo::new(
            "/path/to/worktree",
            "feature-a",
            12345,
            "pnpm test",
            "/path/to/worktree",
        );

        assert_eq!(info.worktree_path, "/path/to/worktree");
        assert_eq!(info.branch, "feature-a");
        assert_eq!(info.pid, 12345);
        assert_eq!(info.command, "pnpm test");
    }

    #[test]
    fn test_process_manager() {
        let mut manager = ProcessManager::new();
        assert_eq!(manager.processes.len(), 0);

        let info = ProcessInfo::new(
            "/path/to/worktree",
            "feature-a",
            12345,
            "pnpm test",
            "/path/to/worktree",
        );

        manager.add_process(info);
        assert_eq!(manager.processes.len(), 1);

        manager.remove_process(12345);
        assert_eq!(manager.processes.len(), 0);
    }
}
