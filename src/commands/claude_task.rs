use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Claude Code task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum TaskStatus {
    /// Task is actively running
    InProgress,
    /// Claude is waiting for user response
    WaitingUser,
    /// Task has completed successfully
    Completed,
    /// Task encountered an error
    Error,
}

#[allow(dead_code)]
impl TaskStatus {
    /// Get emoji representation of status
    pub fn emoji(&self) -> &str {
        match self {
            TaskStatus::InProgress => "🔵",
            TaskStatus::WaitingUser => "🟡",
            TaskStatus::Completed => "🟢",
            TaskStatus::Error => "🔴",
        }
    }

    /// Get color name for terminal output
    pub fn color_name(&self) -> &str {
        match self {
            TaskStatus::InProgress => "blue",
            TaskStatus::WaitingUser => "yellow",
            TaskStatus::Completed => "green",
            TaskStatus::Error => "red",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            TaskStatus::InProgress => "In Progress",
            TaskStatus::WaitingUser => "Waiting for User",
            TaskStatus::Completed => "Completed",
            TaskStatus::Error => "Error",
        }
    }
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
    /// Current task status
    pub status: TaskStatus,
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
        let status = event.status;
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
        self.status = event.status;
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
}

/// Manager for multiple Claude Code task sessions
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct TaskManager {
    /// Map of session_id to task
    tasks: HashMap<String, ClaudeTask>,
}

#[allow(dead_code)]
impl TaskManager {
    /// Create a new empty task manager
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
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

    /// Get active tasks (not completed)
    pub fn active_tasks(&self) -> Vec<&ClaudeTask> {
        self.all_tasks()
            .into_iter()
            .filter(|t| t.status != TaskStatus::Completed)
            .collect()
    }

    /// Get tasks for a specific worktree
    pub fn tasks_for_worktree(&self, worktree_path: &str) -> Vec<&ClaudeTask> {
        self.all_tasks()
            .into_iter()
            .filter(|t| t.is_in_worktree(worktree_path))
            .collect()
    }

    /// Get task by session ID
    pub fn get_task(&self, session_id: &str) -> Option<&ClaudeTask> {
        self.tasks.get(session_id)
    }

    /// Get the progress directory path
    fn get_progress_dir() -> PathBuf {
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

    #[test]
    fn test_task_status_emoji() {
        assert_eq!(TaskStatus::InProgress.emoji(), "🔵");
        assert_eq!(TaskStatus::WaitingUser.emoji(), "🟡");
        assert_eq!(TaskStatus::Completed.emoji(), "🟢");
        assert_eq!(TaskStatus::Error.emoji(), "🔴");
    }

    #[test]
    fn test_task_duration_string() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: TaskStatus::InProgress,
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
    }
}
