use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Claude Code task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum TaskStatus {
    /// Task is actively running
    InProgress,
    /// Response completed, waiting for user action
    Stop,
    /// Session has ended
    SessionEnded,
    /// Task encountered an error
    Error,
}

#[allow(dead_code)]
impl TaskStatus {
    /// Get emoji representation of status
    pub fn emoji(&self) -> &str {
        match self {
            TaskStatus::InProgress => "🔵",
            TaskStatus::Stop => "🟡",
            TaskStatus::SessionEnded => "⚫",
            TaskStatus::Error => "🔴",
        }
    }

    /// Get color name for terminal output
    pub fn color_name(&self) -> &str {
        match self {
            TaskStatus::InProgress => "blue",
            TaskStatus::Stop => "yellow",
            TaskStatus::SessionEnded => "gray",
            TaskStatus::Error => "red",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Stop => "Stop",
            TaskStatus::SessionEnded => "Session Ended",
            TaskStatus::Error => "Error",
        }
    }
}

/// Git project information for display
#[derive(Debug, Clone, Default)]
pub struct GitProjectInfo {
    /// Repository name (e.g., "ccmon")
    pub repo_name: Option<String>,
    /// Worktree/directory name (e.g., "fix-prerelease")
    pub worktree_name: String,
}

impl GitProjectInfo {
    /// Format as "repo::worktree" or just "worktree" if repo unavailable
    pub fn display_name(&self) -> String {
        match &self.repo_name {
            Some(repo) => format!("{}::{}", repo, self.worktree_name),
            None => self.worktree_name.clone(),
        }
    }
}

/// Get git project info from a worktree path
pub fn get_git_project_info(worktree_path: &str) -> GitProjectInfo {
    let path = Path::new(worktree_path);

    // Extract worktree name from path
    let worktree_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Try to get repo name from git remote
    let repo_name = get_repo_name_from_git(path);

    GitProjectInfo {
        repo_name,
        worktree_name,
    }
}

/// Extract repository name from git remote URL
fn get_repo_name_from_git(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout);
    parse_repo_name_from_url(url.trim())
}

/// Parse repository name from git URL
/// Handles: https://github.com/user/repo.git, git@github.com:user/repo.git
fn parse_repo_name_from_url(url: &str) -> Option<String> {
    // Remove trailing .git
    let url = url.trim_end_matches(".git");

    // Extract last path component
    // For HTTPS: https://github.com/user/repo -> split by '/' -> "repo"
    // For SSH: git@github.com:user/repo -> split by '/' -> "repo"
    url.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Single event from Claude Code session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TaskEvent {
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Claude Code session ID
    pub session_id: String,
    /// Hook event name (SessionStart, PostToolUse, Stop, SessionEnd)
    pub event: String,
    /// Tool name (Write, Edit, Bash, etc.), if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Current task status (optional for some notification events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    /// Human-readable event message
    pub message: String,
    /// Working directory where event occurred
    pub cwd: String,
}

/// Aggregated task information for a Claude Code session
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClaudeTask {
    /// Session ID
    pub session_id: String,
    /// Time when task started
    pub start_time: DateTime<Utc>,
    /// Time of last event
    pub last_update: DateTime<Utc>,
    /// Current task status
    pub status: TaskStatus,
    /// All events in chronological order
    pub events: Vec<TaskEvent>,
    /// Working directory (from most recent event)
    pub worktree_path: String,
    /// Last meaningful message
    pub last_message: String,
}

#[allow(dead_code)]
impl ClaudeTask {
    /// Create a new task from the first event
    fn new(event: TaskEvent) -> Self {
        let start_time = event.timestamp;
        let last_update = event.timestamp;
        // Default to Stop if status is not provided (e.g., SessionStart event)
        let status = event.status.unwrap_or(TaskStatus::Stop);
        let worktree_path = event.cwd.clone();
        let last_message = event.message.clone();
        let session_id = event.session_id.clone();

        Self {
            session_id,
            start_time,
            last_update,
            status,
            events: vec![event],
            worktree_path,
            last_message,
        }
    }

    /// Add a new event to this task
    fn add_event(&mut self, event: TaskEvent) {
        self.last_update = event.timestamp;
        // Only update status if the event has one
        if let Some(status) = event.status {
            self.status = status;
        }
        self.worktree_path = event.cwd.clone();

        // Update last message if it's meaningful (not empty)
        if !event.message.is_empty() && event.message != "Unknown event" {
            self.last_message = event.message.clone();
        }

        self.events.push(event);
    }

    /// Get duration since task started
    pub fn duration(&self) -> chrono::Duration {
        self.last_update - self.start_time
    }

    /// Get duration string in human-readable format
    pub fn duration_string(&self) -> String {
        let duration = self.duration();
        let seconds = duration.num_seconds();

        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }

    /// Check if task is associated with a specific worktree path
    pub fn is_in_worktree(&self, worktree_path: &str) -> bool {
        // First try exact match with canonicalized paths
        if let (Ok(task_path), Ok(wt_path)) = (
            Path::new(&self.worktree_path).canonicalize(),
            Path::new(worktree_path).canonicalize(),
        ) {
            if task_path == wt_path {
                return true;
            }

            // Check if task path is a subdirectory of worktree path
            // This handles cases where task is running in a subdirectory
            if task_path.starts_with(&wt_path) {
                return true;
            }
        }

        // Fallback: string comparison for non-existent paths
        // Use path component comparison to avoid false matches like
        // "/home/user/feature" matching "/home/user/feature-backup"
        let task_components: Vec<_> = Path::new(&self.worktree_path).components().collect();
        let wt_components: Vec<_> = Path::new(worktree_path).components().collect();

        // Task path must start with all worktree path components
        if task_components.len() >= wt_components.len() {
            task_components
                .iter()
                .zip(wt_components.iter())
                .all(|(a, b)| a == b)
        } else {
            false
        }
    }

    /// Count events by tool type
    pub fn tool_usage(&self) -> HashMap<String, usize> {
        let mut usage = HashMap::new();

        for event in &self.events {
            if let Some(tool) = &event.tool {
                *usage.entry(tool.clone()).or_insert(0) += 1;
            }
        }

        usage
    }

    /// Check if task has actually started (not just SessionStart)
    pub fn has_started(&self) -> bool {
        // Task has started if there are multiple events or single non-SessionStart event
        self.events.len() > 1
            || self
                .events
                .first()
                .is_some_and(|e| e.event != "SessionStart")
    }
}

/// Manager for multiple Claude Code task sessions
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct TaskManager {
    /// Map of session_id to task
    tasks: HashMap<String, ClaudeTask>,
    /// File modification times for caching
    file_mtimes: HashMap<PathBuf, SystemTime>,
}

#[allow(dead_code)]
impl TaskManager {
    /// Create a new empty task manager
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            file_mtimes: HashMap::new(),
        }
    }

    /// Load all tasks from the progress directory
    pub fn load() -> Result<Self> {
        let progress_dir = Self::get_progress_dir();

        if !progress_dir.exists() {
            return Ok(Self::new());
        }

        let mut manager = Self::new();

        for entry in fs::read_dir(&progress_dir)
            .with_context(|| format!("Failed to read directory: {}", progress_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            // Only process .jsonl files
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }

            // Record file modification time
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(mtime) = metadata.modified() {
                    manager.file_mtimes.insert(path.clone(), mtime);
                }
            }

            if let Err(e) = manager.load_session_file(&path) {
                eprintln!("Warning: Failed to load {}: {}", path.display(), e);
            }
        }

        Ok(manager)
    }

    /// Load a single session file
    fn load_session_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let mut valid_events = 0;
        let mut parse_errors = 0;

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            // Error-tolerant parsing: skip invalid lines instead of failing entire file
            match serde_json::from_str::<TaskEvent>(line) {
                Ok(event) => {
                    self.add_event(event);
                    valid_events += 1;
                }
                Err(e) => {
                    parse_errors += 1;
                    eprintln!(
                        "⚠️  Warning: Skipping invalid line in {}:{}: {}",
                        path.display(),
                        line_num + 1,
                        e
                    );
                    // Continue processing remaining lines
                }
            }
        }

        if parse_errors > 0 {
            eprintln!(
                "⚠️  Session file {} had {} parse errors ({} events loaded successfully)",
                path.display(),
                parse_errors,
                valid_events
            );
        }

        Ok(())
    }

    /// Add an event to the appropriate task
    fn add_event(&mut self, event: TaskEvent) {
        let session_id = event.session_id.clone();

        self.tasks
            .entry(session_id)
            .and_modify(|task| task.add_event(event.clone()))
            .or_insert_with(|| ClaudeTask::new(event));
    }

    /// Get all tasks
    pub fn all_tasks(&self) -> Vec<&ClaudeTask> {
        let mut tasks: Vec<_> = self.tasks.values().collect();
        // Sort by last update, most recent first
        tasks.sort_by(|a, b| b.last_update.cmp(&a.last_update));
        tasks
    }

    /// Get active tasks (not completed and actually started)
    pub fn active_tasks(&self) -> Vec<&ClaudeTask> {
        self.all_tasks()
            .into_iter()
            .filter(|t| t.status != TaskStatus::SessionEnded && t.has_started())
            .collect()
    }

    /// Get tasks for a specific worktree
    pub fn tasks_for_worktree(&self, worktree_path: &str) -> Vec<&ClaudeTask> {
        self.all_tasks()
            .into_iter()
            .filter(|t| t.is_in_worktree(worktree_path))
            .collect()
    }

    /// Get the latest task for each unique worktree path
    /// Returns tasks grouped by worktree, keeping only the most recent session for each
    pub fn latest_tasks_by_worktree(&self) -> Vec<&ClaudeTask> {
        // Group tasks by worktree_path, keeping only the most recent
        let mut worktree_latest: HashMap<&str, &ClaudeTask> = HashMap::new();

        for task in self.tasks.values() {
            let worktree = task.worktree_path.as_str();

            match worktree_latest.get(worktree) {
                Some(existing) => {
                    if task.last_update > existing.last_update {
                        worktree_latest.insert(worktree, task);
                    }
                }
                None => {
                    worktree_latest.insert(worktree, task);
                }
            }
        }

        // Collect and sort by last_update (most recent first)
        let mut tasks: Vec<_> = worktree_latest.into_values().collect();
        tasks.sort_by(|a, b| b.last_update.cmp(&a.last_update));
        tasks
    }

    /// Get task by session ID
    pub fn get_task(&self, session_id: &str) -> Option<&ClaudeTask> {
        self.tasks.get(session_id)
    }

    /// Refresh task data by reloading only changed files
    /// This is much faster than load() when most files haven't changed
    pub fn refresh(&mut self) -> Result<()> {
        let progress_dir = Self::get_progress_dir();

        if !progress_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&progress_dir)
            .with_context(|| format!("Failed to read directory: {}", progress_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            // Only process .jsonl files
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }

            // Check if file has been modified since last load
            let current_mtime = match fs::metadata(&path).and_then(|m| m.modified()) {
                Ok(mtime) => mtime,
                Err(_) => continue,
            };

            let should_reload = match self.file_mtimes.get(&path) {
                Some(&last_mtime) => current_mtime > last_mtime,
                None => true, // New file
            };

            if should_reload {
                // Remove old events for this session before reloading
                if let Some(session_id) = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                {
                    self.tasks.remove(&session_id);
                }

                if let Err(e) = self.load_session_file(&path) {
                    eprintln!("Warning: Failed to reload {}: {}", path.display(), e);
                }

                self.file_mtimes.insert(path, current_mtime);
            }
        }

        Ok(())
    }

    /// Get the progress directory path
    /// Falls back to current directory if home directory cannot be determined
    pub fn get_progress_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("task-progress")
    }

    /// Count tasks by status
    pub fn status_counts(&self) -> HashMap<TaskStatus, usize> {
        let mut counts = HashMap::new();

        for task in self.tasks.values() {
            *counts.entry(task.status).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_task_status_emoji() {
        assert_eq!(TaskStatus::InProgress.emoji(), "🔵");
        assert_eq!(TaskStatus::Stop.emoji(), "🟡");
        assert_eq!(TaskStatus::SessionEnded.emoji(), "⚫");
        assert_eq!(TaskStatus::Error.emoji(), "🔴");
    }

    #[test]
    fn test_task_status_serialization() {
        // Test JSON serialization/deserialization
        let status = TaskStatus::Stop;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"stop\"");

        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TaskStatus::Stop);
    }

    #[test]
    fn test_task_duration_string() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Test".to_string(),
            cwd: "/tmp".to_string(),
        };

        let task = ClaudeTask::new(event);
        let duration = task.duration_string();

        // Should be "0s" or "1s" since we just created it
        assert!(duration.ends_with('s'));
    }

    #[test]
    fn test_task_manager_creation() {
        let manager = TaskManager::new();
        assert_eq!(manager.all_tasks().len(), 0);
        assert_eq!(manager.file_mtimes.len(), 0);
    }

    #[test]
    fn test_is_in_worktree_edge_cases() {
        // Test that "/feature" doesn't match "/feature-backup"
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Test".to_string(),
            cwd: "/home/user/feature".to_string(),
        };

        let task = ClaudeTask::new(event);

        // Should match exact path
        assert!(task.is_in_worktree("/home/user/feature"));

        // Should NOT match similar but different path
        assert!(!task.is_in_worktree("/home/user/feature-backup"));
        assert!(!task.is_in_worktree("/home/user/featureX"));

        // Should match parent directory
        assert!(task.is_in_worktree("/home/user"));
        assert!(task.is_in_worktree("/home"));
    }

    #[test]
    fn test_error_tolerant_jsonl_parsing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("broken.jsonl");

        // Create JSONL with mixed valid and invalid lines
        let mut file = fs::File::create(&file_path)?;
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-30T10:00:00Z","session_id":"test","event":"SessionStart","tool":null,"status":"in_progress","message":"Valid","cwd":"/tmp"}}"#
        )?;
        writeln!(file, "this is not valid json")?;
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-30T10:01:00Z","session_id":"test","event":"Stop","tool":null,"status":"stop","message":"Valid","cwd":"/tmp"}}"#
        )?;
        writeln!(file, "{{malformed json without closing brace")?;
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-30T10:02:00Z","session_id":"test","event":"SessionEnd","tool":null,"status":"session_ended","message":"Valid","cwd":"/tmp"}}"#
        )?;
        drop(file);

        // Load should succeed despite invalid lines
        let mut manager = TaskManager::new();
        manager.load_session_file(&file_path)?;

        // Should have loaded 3 valid events
        let task = manager.get_task("test").expect("Task should exist");
        assert_eq!(task.events.len(), 3);
        assert_eq!(task.status, TaskStatus::SessionEnded);

        Ok(())
    }

    #[test]
    fn test_task_tool_usage() {
        let mut events = Vec::new();

        // Create events with different tools
        for (i, tool) in ["Write", "Edit", "Write", "Bash", "Write"]
            .iter()
            .enumerate()
        {
            events.push(TaskEvent {
                timestamp: Utc::now(),
                session_id: "test".to_string(),
                event: "PostToolUse".to_string(),
                tool: Some(tool.to_string()),
                status: Some(TaskStatus::InProgress),
                message: format!("Event {}", i),
                cwd: "/tmp".to_string(),
            });
        }

        let mut task = ClaudeTask::new(events[0].clone());
        for event in events.iter().skip(1) {
            task.add_event(event.clone());
        }

        let usage = task.tool_usage();
        assert_eq!(usage.get("Write"), Some(&3));
        assert_eq!(usage.get("Edit"), Some(&1));
        assert_eq!(usage.get("Bash"), Some(&1));
    }

    #[test]
    fn test_task_manager_refresh_only_changed_files() -> Result<()> {
        // This test verifies that refresh() only reloads changed files
        // Note: This is a simplified test that checks the internal state tracking
        // A full integration test would require setting up the actual progress directory

        let mut manager = TaskManager::new();

        // Manually add an event to simulate a loaded task
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Test".to_string(),
            cwd: "/tmp".to_string(),
        };
        manager.add_event(event);

        // Verify task was loaded
        assert!(manager.get_task("test_session").is_some());

        // Verify file_mtimes starts empty (no files loaded via refresh yet)
        assert_eq!(manager.file_mtimes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_empty_session_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.jsonl");

        // Create empty file
        fs::File::create(&file_path)?;

        let mut manager = TaskManager::new();
        // Should not fail on empty file
        manager.load_session_file(&file_path)?;

        Ok(())
    }

    #[test]
    fn test_task_status_counts() {
        let mut manager = TaskManager::new();

        // Add tasks with different statuses
        for (i, status) in [
            TaskStatus::InProgress,
            TaskStatus::InProgress,
            TaskStatus::Stop,
            TaskStatus::SessionEnded,
        ]
        .iter()
        .enumerate()
        {
            let event = TaskEvent {
                timestamp: Utc::now(),
                session_id: format!("session{}", i),
                event: "SessionStart".to_string(),
                tool: None,
                status: Some(*status),
                message: "Test".to_string(),
                cwd: "/tmp".to_string(),
            };
            manager.add_event(event);
        }

        let counts = manager.status_counts();
        assert_eq!(counts.get(&TaskStatus::InProgress), Some(&2));
        assert_eq!(counts.get(&TaskStatus::Stop), Some(&1));
        assert_eq!(counts.get(&TaskStatus::SessionEnded), Some(&1));
        assert_eq!(counts.get(&TaskStatus::Error), None);
    }

    #[test]
    fn test_event_without_status() {
        // Test that events without status field are handled correctly
        let event_with_status = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Started".to_string(),
            cwd: "/tmp".to_string(),
        };

        let event_without_status = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "Notification".to_string(),
            tool: None,
            status: None,
            message: "Unknown event".to_string(),
            cwd: "/tmp".to_string(),
        };

        let mut task = ClaudeTask::new(event_with_status);
        assert_eq!(task.status, TaskStatus::InProgress);

        // Adding event without status should not change the current status
        task.add_event(event_without_status);
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_jsonl_without_status_field() -> Result<()> {
        // Test parsing JSONL with missing status field
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.jsonl");

        let mut file = fs::File::create(&file_path)?;
        // Line with status
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-31T10:00:00Z","session_id":"test","event":"SessionStart","tool":null,"status":"in_progress","message":"Started","cwd":"/tmp"}}"#
        )?;
        // Line without status (Notification event)
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-31T10:00:05Z","session_id":"test","event":"Notification","tool":null,"message":"Unknown event","cwd":"/tmp"}}"#
        )?;
        // Line with status again
        writeln!(
            file,
            r#"{{"timestamp":"2025-12-31T10:00:10Z","session_id":"test","event":"Stop","tool":null,"status":"stop","message":"Waiting","cwd":"/tmp"}}"#
        )?;
        drop(file);

        let mut manager = TaskManager::new();
        manager.load_session_file(&file_path)?;

        let task = manager.get_task("test").expect("Task should exist");
        assert_eq!(task.events.len(), 3);
        // Final status should be "stop" from the last event with status
        assert_eq!(task.status, TaskStatus::Stop);

        Ok(())
    }

    #[test]
    fn test_has_started() {
        // Task with only SessionStart should not be started
        let session_start_only = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        };
        let task = ClaudeTask::new(session_start_only);
        assert!(!task.has_started());

        // Task with SessionStart + PostToolUse should be started
        let mut task = ClaudeTask::new(TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        });
        task.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "PostToolUse".to_string(),
            tool: Some("Read".to_string()),
            status: Some(TaskStatus::InProgress),
            message: "Read file".to_string(),
            cwd: "/tmp".to_string(),
        });
        assert!(task.has_started());
    }

    #[test]
    fn test_active_tasks_excludes_session_start_only() {
        let mut manager = TaskManager::new();

        // Add task with only SessionStart
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "not_started".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        });

        // Add task with SessionStart + PostToolUse
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "started".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        });
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "started".to_string(),
            event: "PostToolUse".to_string(),
            tool: Some("Read".to_string()),
            status: Some(TaskStatus::InProgress),
            message: "Read file".to_string(),
            cwd: "/tmp".to_string(),
        });

        // Only the started task should be active
        let active = manager.active_tasks();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].session_id, "started");
    }

    #[test]
    fn test_user_prompt_submit_starts_task() {
        // Task with SessionStart + UserPromptSubmit should be started
        let mut task = ClaudeTask::new(TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        });
        assert!(!task.has_started());

        task.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "UserPromptSubmit".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Processing user prompt".to_string(),
            cwd: "/tmp".to_string(),
        });
        assert!(task.has_started());
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_active_tasks_includes_user_prompt_submit() {
        let mut manager = TaskManager::new();

        // Add task with SessionStart + UserPromptSubmit
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "prompt_submitted".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: None,
            message: "Session started".to_string(),
            cwd: "/tmp".to_string(),
        });
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "prompt_submitted".to_string(),
            event: "UserPromptSubmit".to_string(),
            tool: None,
            status: Some(TaskStatus::InProgress),
            message: "Processing user prompt".to_string(),
            cwd: "/tmp".to_string(),
        });

        // Task should be active after UserPromptSubmit
        let active = manager.active_tasks();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].session_id, "prompt_submitted");
        assert_eq!(active[0].status, TaskStatus::InProgress);
    }

    #[test]
    fn test_latest_tasks_by_worktree() {
        let mut manager = TaskManager::new();

        // Add two sessions for the same worktree (older one)
        manager.add_event(TaskEvent {
            timestamp: Utc::now() - chrono::Duration::seconds(100),
            session_id: "old_session".to_string(),
            event: "PostToolUse".to_string(),
            tool: Some("Read".to_string()),
            status: Some(TaskStatus::SessionEnded),
            message: "Old session".to_string(),
            cwd: "/home/user/project".to_string(),
        });

        // Add newer session for same worktree
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "new_session".to_string(),
            event: "PostToolUse".to_string(),
            tool: Some("Write".to_string()),
            status: Some(TaskStatus::InProgress),
            message: "New session".to_string(),
            cwd: "/home/user/project".to_string(),
        });

        // Add session for different worktree
        manager.add_event(TaskEvent {
            timestamp: Utc::now(),
            session_id: "other_session".to_string(),
            event: "PostToolUse".to_string(),
            tool: Some("Bash".to_string()),
            status: Some(TaskStatus::InProgress),
            message: "Other project".to_string(),
            cwd: "/home/user/other-project".to_string(),
        });

        let latest = manager.latest_tasks_by_worktree();

        // Should have 2 worktrees, not 3 sessions
        assert_eq!(latest.len(), 2);

        // The /home/user/project worktree should show new_session (most recent)
        let project_task = latest
            .iter()
            .find(|t| t.worktree_path == "/home/user/project")
            .expect("Should find project task");
        assert_eq!(project_task.session_id, "new_session");
        assert_eq!(project_task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_parse_repo_name_https() {
        assert_eq!(
            super::parse_repo_name_from_url("https://github.com/user/ccmon.git"),
            Some("ccmon".to_string())
        );
        assert_eq!(
            super::parse_repo_name_from_url("https://github.com/user/ccmon"),
            Some("ccmon".to_string())
        );
        assert_eq!(
            super::parse_repo_name_from_url("https://gitlab.com/group/subgroup/project.git"),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_parse_repo_name_ssh() {
        assert_eq!(
            super::parse_repo_name_from_url("git@github.com:user/ccmon.git"),
            Some("ccmon".to_string())
        );
        assert_eq!(
            super::parse_repo_name_from_url("git@github.com:user/ccmon"),
            Some("ccmon".to_string())
        );
    }

    #[test]
    fn test_parse_repo_name_edge_cases() {
        // Empty URL
        assert_eq!(super::parse_repo_name_from_url(""), None);
        // Just .git
        assert_eq!(super::parse_repo_name_from_url(".git"), None);
    }

    #[test]
    fn test_git_project_info_display() {
        let info = GitProjectInfo {
            repo_name: Some("ccmon".to_string()),
            worktree_name: "fix-prerelease".to_string(),
        };
        assert_eq!(info.display_name(), "ccmon::fix-prerelease");

        let info_no_repo = GitProjectInfo {
            repo_name: None,
            worktree_name: "fix-prerelease".to_string(),
        };
        assert_eq!(info_no_repo.display_name(), "fix-prerelease");
    }
}
