# Code Review Supplements - 詳細分析と実装例

## 1. JSONLファイル読み込みの修正実装例

### 現在の問題コード
```rust
// src/commands/claude_task.rs: 223-244
fn load_session_file(&mut self, path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let event: TaskEvent = serde_json::from_str(line).with_context(|| {
            format!(
                "Failed to parse JSON at {}:{}",
                path.display(),
                line_num + 1
            )
        })?;  // ← 1 行エラーで全体失敗

        self.add_event(event);
    }

    Ok(())
}
```

### 修正実装例
```rust
fn load_session_file(&mut self, path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut error_count = 0;
    let mut success_count = 0;

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<TaskEvent>(line) {
            Ok(event) => {
                self.add_event(event);
                success_count += 1;
            }
            Err(e) => {
                // エラーは警告として出力し、処理は継続
                eprintln!(
                    "⚠️  Warning: Failed to parse event at {}:{}: {}",
                    path.display(),
                    line_num + 1,
                    e
                );
                error_count += 1;
            }
        }
    }

    // 全部失敗した場合のみ error にする
    if error_count > 0 && success_count == 0 {
        anyhow::bail!(
            "❌ Could not recover any events from {}: {} parse errors",
            path.display(),
            error_count
        );
    }

    // 一部失敗はOKだが、警告を返す
    if error_count > 0 {
        eprintln!(
            "ℹ️  Recovered {}/{} events from {}",
            success_count,
            success_count + error_count,
            path.display()
        );
    }

    Ok(())
}
```

### テスト例
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_session_with_malformed_json() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");

        // 正常な JSON と不正な JSON を混在させる
        fs::write(
            &session_file,
            r#"{"timestamp":"2025-12-30T00:00:00Z","session_id":"test1","event":"SessionStart","status":"in_progress","message":"Started","cwd":"/tmp"}
{"invalid json}
{"timestamp":"2025-12-30T00:00:01Z","session_id":"test1","event":"PostToolUse","status":"in_progress","message":"Used tool","cwd":"/tmp"}
"#,
        ).unwrap();

        let mut manager = TaskManager::new();

        // 不正な行があっても、他のイベントは復旧される
        let result = manager.load_session_file(&session_file);
        assert!(result.is_ok(), "Should recover partial data");

        let all_tasks = manager.all_tasks();
        assert_eq!(all_tasks.len(), 1);

        let task = all_tasks[0];
        assert_eq!(task.events.len(), 2); // 不正な行はスキップ
    }

    #[test]
    fn test_load_session_all_malformed() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");

        fs::write(&session_file, "not json\n{invalid}\n").unwrap();

        let mut manager = TaskManager::new();
        let result = manager.load_session_file(&session_file);

        // 全部失敗の場合は Error
        assert!(result.is_err(), "Should return error when no events recovered");
    }

    #[test]
    fn test_load_session_empty_lines() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");

        fs::write(
            &session_file,
            r#"
{"timestamp":"2025-12-30T00:00:00Z","session_id":"test1","event":"SessionStart","status":"in_progress","message":"Started","cwd":"/tmp"}

{"timestamp":"2025-12-30T00:00:01Z","session_id":"test1","event":"SessionEnd","status":"completed","message":"Ended","cwd":"/tmp"}

"#,
        ).unwrap();

        let mut manager = TaskManager::new();
        let result = manager.load_session_file(&session_file);

        assert!(result.is_ok());
        let tasks = manager.all_tasks();
        assert_eq!(tasks[0].events.len(), 2);
    }
}
```

---

## 2. パフォーマンス最適化：TaskManager キャッシング

### 現在のパフォーマンス問題

```
100ms ごとに:
- event::poll(Duration::from_millis(100)) で polling
- UI 再描画時に TaskManager::load() が呼ばれる可能性
  ↓
- 全セッションファイルを逐一 read/parse
  ↓
O(n) where n = セッションファイル数
```

### 最適化実装例

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct TaskManager {
    tasks: HashMap<String, ClaudeTask>,

    // キャッシュ関連
    last_load: Option<Instant>,
    cache_ttl: Duration,

    // ファイルシステム監視
    last_progress_dir_mtime: Option<std::time::SystemTime>,
}

impl TaskManager {
    /// キャッシュ有効期間を設定
    pub fn new_with_cache(ttl_secs: u64) -> Self {
        Self {
            tasks: HashMap::new(),
            last_load: None,
            cache_ttl: Duration::from_secs(ttl_secs),
            last_progress_dir_mtime: None,
        }
    }

    /// キャッシュが有効かチェック
    fn is_cache_valid(&self) -> bool {
        if let Some(last_load) = self.last_load {
            let elapsed = last_load.elapsed();
            return elapsed < self.cache_ttl;
        }
        false
    }

    /// ディレクトリの変更を検知
    fn has_directory_changed() -> bool {
        let progress_dir = Self::get_progress_dir();
        if !progress_dir.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&progress_dir) {
            if let Ok(modified) = metadata.modified() {
                return Some(modified) != self.last_progress_dir_mtime;
            }
        }
        false
    }

    /// 差分更新：キャッシュ有効でも新ファイルがあれば読み込み
    pub fn load_or_refresh() -> Result<Self> {
        let progress_dir = Self::get_progress_dir();

        if !progress_dir.exists() {
            return Ok(Self::new_with_cache(500)); // Default: 500ms TTL
        }

        let mut manager = Self::new_with_cache(500);

        // 最後の読み込みより後に変更されたファイルのみ読み込み
        for entry in fs::read_dir(&progress_dir)
            .with_context(|| format!("Failed to read directory: {}", progress_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }

            // mtime をチェック
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Some(last_load) = manager.last_load {
                        if let Ok(elapsed) = modified.elapsed() {
                            // 最後の読み込み時刻より新しければ読み込み
                            if elapsed < Duration::from_secs(60) {
                                if let Err(e) = manager.load_session_file(&path) {
                                    eprintln!("⚠️  Failed to load {}: {}", path.display(), e);
                                }
                            }
                        }
                    } else {
                        // 初回読み込み
                        if let Err(e) = manager.load_session_file(&path) {
                            eprintln!("⚠️  Failed to load {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        manager.last_load = Some(Instant::now());
        Ok(manager)
    }
}
```

### UI での使用

```rust
// ui.rs
struct App {
    worktrees: Vec<WorktreeDetail>,
    process_manager: ProcessManager,
    task_manager: TaskManager,
    task_manager_cache_time: Instant,
    selected_index: usize,
    list_state: ListState,
    should_quit: bool,
}

impl App {
    fn new() -> Result<Self> {
        // ...
        let task_manager = TaskManager::load_or_refresh()?;
        let task_manager_cache_time = Instant::now();

        Ok(Self {
            // ...
            task_manager,
            task_manager_cache_time,
            // ...
        })
    }

    fn refresh(&mut self) -> Result<()> {
        // ...

        // タスク情報はキャッシュ有効期間内なら再読み込みしない
        if self.task_manager_cache_time.elapsed() > Duration::from_millis(500) {
            self.task_manager = TaskManager::load_or_refresh()?;
            self.task_manager_cache_time = Instant::now();
        }

        Ok(())
    }
}
```

---

## 3. コマンド実行のセキュリティ改善

### リスク分析

```
Current code:
wtenv notify "echo hello; rm -rf /"
                        ↓
execute_with_notification(command = "echo hello; rm -rf /", ...)
                        ↓
Command::new("bash").args(["-c", command])
                        ↓
bash -c "echo hello; rm -rf /"
                        ↓
Both commands are executed!
```

### 改善案 1：入力バリデーション

```rust
/// Validate command for dangerous patterns
pub fn validate_command(command: &str) -> Result<()> {
    const DANGEROUS_PATTERNS: &[&str] = &[
        "rm -rf /",
        "dd if=/dev/zero",
        "format",
        "mkfs",
        "chmod -R 777 /",
    ];

    for pattern in DANGEROUS_PATTERNS {
        if command.contains(pattern) {
            anyhow::bail!(
                "❌ Command contains dangerous pattern: {}\n\
                 This command is blocked for safety reasons.",
                pattern
            );
        }
    }

    Ok(())
}

pub fn execute_with_notification(
    command: &str,
    working_dir: &Path,
    notify_on_success: bool,
    notify_on_error: bool,
) -> Result<()> {
    // Validate first
    validate_command(command)?;

    // ... rest of implementation
}
```

### 改善案 2：shell-quoting ライブラリ

```toml
# Cargo.toml
[dependencies]
shell-quote = "1.9"
```

```rust
use shell_quote::Bash;

pub fn execute_with_notification_safe(
    program: &str,
    args: &[&str],
    working_dir: &Path,
    notify_on_success: bool,
    notify_on_error: bool,
) -> Result<()> {
    println!("{}", format!("🚀 Executing: {} {}", program, args.join(" ")).cyan());
    println!();

    let start = Instant::now();

    // Direct execution without shell
    let output = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .output()?;

    let duration = start.elapsed();
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // ... rest of implementation
}
```

### テスト例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command_detection() {
        let result = validate_command("rm -rf /");
        assert!(result.is_err(), "Should reject rm -rf");

        let result = validate_command("echo hello && rm -rf /");
        assert!(result.is_err(), "Should detect dangerous pattern after semicolon");

        let result = validate_command("echo hello");
        assert!(result.is_ok(), "Safe command should pass");
    }

    #[test]
    fn test_execute_with_valid_working_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = execute_with_notification(
            "echo test",
            temp_dir.path(),
            false,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_invalid_working_dir() {
        let invalid_path = std::path::PathBuf::from("/nonexistent/directory/xyz");
        let result = execute_with_notification(
            "echo test",
            &invalid_path,
            false,
            false,
        );
        assert!(result.is_err());
    }
}
```

---

## 4. Python スクリプトのエラーハンドリング改善

### 現在の問題

```python
# track-progress.py: 115-122
except Exception as e:
    error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
    error_log.parent.mkdir(parents=True, exist_ok=True)

    with open(error_log, "a") as f:
        f.write(f"{datetime.utcnow().isoformat()}: {str(e)}\n")
    # hook は silent に終了
```

**問題点**:
- Claude には何も通知されない
- User は error log を手動で確認する必要がある
- Multiple exceptions が発生する可能性

### 改善実装

```python
#!/usr/bin/env python3
"""
Claude Code Task Progress Tracker Hook - Enhanced Error Handling
"""

import json
import sys
import os
from pathlib import Path
from datetime import datetime
import traceback


class TaskProgressError(Exception):
    """Base exception for task progress tracking."""
    pass


def create_progress_directory() -> Path:
    """Create progress directory with proper permissions."""
    progress_dir = Path.home() / ".claude" / "task-progress"
    try:
        progress_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
        return progress_dir
    except OSError as e:
        raise TaskProgressError(f"Failed to create progress directory: {e}")


def log_error(error: Exception, context: dict = None):
    """Log error with context information."""
    try:
        progress_dir = Path.home() / ".claude" / "task-progress"
        progress_dir.mkdir(parents=True, exist_ok=True, mode=0o700)

        error_log = progress_dir / "errors.log"

        # Set proper file permissions on first creation
        if not error_log.exists():
            error_log.touch(mode=0o600)

        timestamp = datetime.utcnow().isoformat()
        error_type = type(error).__name__
        error_msg = str(error)
        stack_trace = traceback.format_exc()

        log_entry = (
            f"{timestamp}\n"
            f"Error Type: {error_type}\n"
            f"Message: {error_msg}\n"
            f"Stack Trace:\n{stack_trace}\n"
        )

        if context:
            log_entry += f"Context: {json.dumps(context)}\n"

        log_entry += "-" * 80 + "\n"

        with open(error_log, "a") as f:
            f.write(log_entry)

    except OSError as log_error:
        # If logging fails, write to stderr at least
        sys.stderr.write(
            f"Failed to write to error log: {log_error}\n"
            f"Original error: {error}\n"
        )


def get_task_status(hook_event: str, tool_name: str = "") -> str:
    """Determine task status based on hook event and tool name."""
    status_map = {
        "SessionStart": "in_progress",
        "Stop": "waiting_user",
        "SessionEnd": "completed",
        "PostToolUse": "in_progress",
    }

    return status_map.get(hook_event, "in_progress")


def get_event_message(hook_data: dict) -> str:
    """Generate a human-readable message for the event."""
    event = hook_data.get("hook_event_name", "")
    tool = hook_data.get("tool_name", "")
    tool_input = hook_data.get("tool_input", {})

    if event == "SessionStart":
        return "Session started"
    elif event == "SessionEnd":
        return "Session completed"
    elif event == "Stop":
        return "Waiting for user response"
    elif event == "PostToolUse":
        if tool == "Write":
            file_path = tool_input.get("file_path", "")
            name = Path(file_path).name if file_path else "unknown"
            return f"Created file: {name[:50]}"
        elif tool == "Edit":
            file_path = tool_input.get("file_path", "")
            name = Path(file_path).name if file_path else "unknown"
            return f"Edited file: {name[:50]}"
        elif tool == "Bash":
            command = tool_input.get("command", "")
            cmd_preview = command[:50] + "..." if len(command) > 50 else command
            return f"Executed: {cmd_preview}"
        elif tool == "Read":
            file_path = tool_input.get("file_path", "")
            name = Path(file_path).name if file_path else "unknown"
            return f"Read file: {name[:50]}"
        else:
            return f"Used tool: {tool}"

    return "Unknown event"


def validate_json_input(hook_data: dict) -> None:
    """Validate required fields in hook data."""
    required_fields = ["session_id", "hook_event_name", "cwd"]

    missing_fields = [f for f in required_fields if f not in hook_data]
    if missing_fields:
        raise TaskProgressError(
            f"Missing required fields: {', '.join(missing_fields)}"
        )


def write_event_record(progress_file: Path, event_record: dict) -> None:
    """Write event record to JSONL file."""
    try:
        with open(progress_file, "a") as f:
            json.dump(event_record, f)
            f.write("\n")
    except IOError as e:
        raise TaskProgressError(f"Failed to write to {progress_file}: {e}")
    except json.JSONEncodeError as e:
        raise TaskProgressError(f"Failed to encode event record: {e}")


def main():
    """Main hook execution."""
    try:
        # Read JSON input from stdin
        try:
            hook_data = json.load(sys.stdin)
        except json.JSONDecodeError as e:
            raise TaskProgressError(f"Invalid JSON input: {e}")

        # Validate input
        validate_json_input(hook_data)

        session_id = hook_data.get("session_id")
        hook_event = hook_data.get("hook_event_name")
        tool_name = hook_data.get("tool_name", "")
        cwd = hook_data.get("cwd", "")

        # Create progress directory
        progress_dir = create_progress_directory()

        # Session-specific progress file
        progress_file = progress_dir / f"{session_id}.jsonl"

        # Determine task status
        status = get_task_status(hook_event, tool_name)
        message = get_event_message(hook_data)

        # Create event record
        event_record = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "session_id": session_id,
            "event": hook_event,
            "tool": tool_name if tool_name else None,
            "status": status,
            "message": message,
            "cwd": cwd,
        }

        # Write event record
        write_event_record(progress_file, event_record)

        # Output feedback only for SessionStart
        if hook_event == "SessionStart":
            sys.stdout.write(
                "✓ Task progress tracking initialized for wtenv UI\n"
                f"  Session: {session_id}\n"
                f"  File: {progress_file}\n"
            )
            sys.stdout.flush()

    except TaskProgressError as e:
        # Expected errors
        log_error(e, {"type": "TaskProgressError"})
        # Don't fail the hook - tracking is optional
        sys.stderr.write(f"⚠️  Task progress tracking error: {e}\n")
        sys.stderr.flush()

    except Exception as e:
        # Unexpected errors
        log_error(e, {"type": "UnexpectedError"})
        # Don't fail the hook - tracking is optional
        sys.stderr.write(
            f"❌ Unexpected error in task progress tracking: {e}\n"
            f"   Check ~/.claude/task-progress/errors.log for details\n"
        )
        sys.stderr.flush()


if __name__ == "__main__":
    main()
```

### テスト例

```python
import unittest
import tempfile
from pathlib import Path
from datetime import datetime
import json

class TestTrackProgress(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.progress_dir = Path(self.temp_dir.name) / ".claude" / "task-progress"

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_get_task_status_mappings(self):
        from track_progress import get_task_status

        assert get_task_status("SessionStart") == "in_progress"
        assert get_task_status("Stop") == "waiting_user"
        assert get_task_status("SessionEnd") == "completed"
        assert get_task_status("PostToolUse", "Bash") == "in_progress"

    def test_validate_json_input_valid(self):
        from track_progress import validate_json_input

        valid_data = {
            "session_id": "test-session",
            "hook_event_name": "SessionStart",
            "cwd": "/tmp"
        }

        # Should not raise
        validate_json_input(valid_data)

    def test_validate_json_input_missing_fields(self):
        from track_progress import validate_json_input, TaskProgressError

        invalid_data = {
            "session_id": "test-session"
            # Missing hook_event_name and cwd
        }

        with self.assertRaises(TaskProgressError):
            validate_json_input(invalid_data)

    def test_get_event_message_truncation(self):
        from track_progress import get_event_message

        long_command = "a" * 100
        hook_data = {
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": long_command}
        }

        msg = get_event_message(hook_data)
        assert len(msg) < len(long_command)
        assert "..." in msg

if __name__ == "__main__":
    unittest.main()
```

---

## 5. is_in_worktree() の改善実装

### 問題の再現

```
Task path: /home/user/projects/feature
Worktree to check: /home/user/projects/feature-backup

Current implementation:
"/home/user/projects/feature".starts_with("/home/user/projects/feature-backup")
→ false

but:

"/home/user/projects/feature-backup".starts_with("/home/user/projects/feature")
→ true (WRONG!)
```

### 修正実装

```rust
/// Check if task is associated with a specific worktree path.
///
/// Uses canonical paths for accurate comparison, handles symlinks,
/// and prevents path prefix matching false positives.
pub fn is_in_worktree(&self, worktree_path: &str) -> bool {
    // Try canonical path comparison first
    if let (Ok(task_canonical), Ok(wt_canonical)) = (
        Path::new(&self.worktree_path).canonicalize(),
        Path::new(worktree_path).canonicalize(),
    ) {
        // Exact match or task is under worktree directory
        return task_canonical == wt_canonical
            || task_canonical.starts_with(&wt_canonical);
    }

    // Fallback: Parse paths and compare components
    // This avoids the "/a/b" vs "/a/b-c" problem
    let task_parts: Vec<&str> = self.worktree_path.split('/').collect();
    let wt_parts: Vec<&str> = worktree_path.split('/').collect();

    // Check if task path starts with worktree path (component-wise)
    if task_parts.len() < wt_parts.len() {
        return false;
    }

    task_parts[..wt_parts.len()] == wt_parts[..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_worktree_exact_match() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: TaskStatus::InProgress,
            message: "Test".to_string(),
            cwd: "/home/user/projects/feature".to_string(),
        };

        let task = ClaudeTask::new(event);

        assert!(task.is_in_worktree("/home/user/projects/feature"));
    }

    #[test]
    fn test_is_in_worktree_prefix_false_positive() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: TaskStatus::InProgress,
            message: "Test".to_string(),
            cwd: "/home/user/projects/feature-backup".to_string(),
        };

        let task = ClaudeTask::new(event);

        // Should NOT match /feature just because path starts with it
        assert!(!task.is_in_worktree("/home/user/projects/feature"));
    }

    #[test]
    fn test_is_in_worktree_subdirectory() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: TaskStatus::InProgress,
            message: "Test".to_string(),
            cwd: "/home/user/projects/feature/src".to_string(),
        };

        let task = ClaudeTask::new(event);

        // Should match parent directory
        assert!(task.is_in_worktree("/home/user/projects/feature"));
    }

    #[test]
    fn test_is_in_worktree_component_boundary() {
        let event = TaskEvent {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            event: "SessionStart".to_string(),
            tool: None,
            status: TaskStatus::InProgress,
            message: "Test".to_string(),
            cwd: "/tmp/abc-def".to_string(),
        };

        let task = ClaudeTask::new(event);

        // Should not match /tmp/abc
        assert!(!task.is_in_worktree("/tmp/abc"));
    }
}
```

---

## 6. パフォーマンステスト

### ベンチマーク例

```rust
#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;
    use tempfile::TempDir;

    #[test]
    fn bench_load_many_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let progress_dir = temp_dir.path();

        // 100 個のセッションファイルを作成
        for i in 0..100 {
            let session_file = progress_dir.join(format!("session-{}.jsonl", i));
            let mut content = String::new();

            // 各ファイルに 10 個のイベント
            for j in 0..10 {
                let event = TaskEvent {
                    timestamp: Utc::now(),
                    session_id: format!("session-{}", i),
                    event: "PostToolUse".to_string(),
                    tool: Some("Bash".to_string()),
                    status: TaskStatus::InProgress,
                    message: format!("Event {}", j),
                    cwd: "/tmp".to_string(),
                };

                let json = serde_json::to_string(&event).unwrap();
                content.push_str(&format!("{}\n", json));
            }

            std::fs::write(&session_file, content).unwrap();
        }

        // Benchmark load
        let start = Instant::now();
        let mut manager = TaskManager::new();

        // This would normally call load()
        for entry in std::fs::read_dir(progress_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                manager.load_session_file(&path).ok();
            }
        }

        let elapsed = start.elapsed();
        println!("Loaded 100 sessions (1000 events) in {:.2}ms", elapsed.as_secs_f64() * 1000.0);

        // Target: < 200ms for 100 sessions
        assert!(elapsed.as_millis() < 200, "Performance regression detected");
    }
}
```

---

## 7. 推奨 CI/CD チェック

### Pre-commit hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Rust tests
cargo test --lib --no-fail-fast
if [ $? -ne 0 ]; then
  echo "❌ Rust tests failed"
  exit 1
fi

# Python tests
python -m pytest .claude/hooks/ -v
if [ $? -ne 0 ]; then
  echo "❌ Python tests failed"
  exit 1
fi

# Clippy
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
  echo "❌ Clippy warnings found"
  exit 1
fi

echo "✅ All checks passed"
```

### GitHub Actions workflow

```yaml
name: Code Review Checks

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --lib

      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Install Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Run Python tests
        run: |
          cd .claude/hooks
          python -m pytest -v

      - name: Code coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

**End of Supplements**
