use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Set executable permissions on a file (Unix only)
#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

/// No-op on non-Unix platforms
#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Claude Code hooks 設定テンプレート
const CLAUDE_SETTINGS_TEMPLATE: &str = r#"{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/session-init.sh"
          },
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          },
          {
            "type": "command",
            "command": "~/.claude/stop-hook-git-check.sh"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/track-progress.py"
          }
        ]
      }
    ]
  }
}
"#;

/// SessionStart hook スクリプトテンプレート
const SESSION_INIT_HOOK_TEMPLATE: &str = r#"#!/bin/bash
# Session initialization hook for Claude Code
# This script provides development context at the start of each session

set -e

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Not in a git repository"
  exit 0
fi

echo "📍 Development Context"
echo ""

# Show current branch and worktree info
echo "🌲 Worktree: $(basename $(pwd))"
echo "🔀 Branch: $(git branch --show-current 2>/dev/null || echo 'detached')"
echo ""

# Show recent commits
echo "📝 Recent commits:"
git log --oneline -n 3 2>/dev/null || true
echo ""

# Show uncommitted changes
if ! git diff --quiet 2>/dev/null; then
  echo "⚠️  Uncommitted changes detected"
fi

# Show staged changes
if ! git diff --cached --quiet 2>/dev/null; then
  echo "📋 Staged changes detected"
fi

# Show untracked files
untracked=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l)
if [ "$untracked" -gt 0 ]; then
  echo "📄 $untracked untracked file(s)"
fi

exit 0
"#;

/// Git check stop hook テンプレート
const STOP_HOOK_GIT_CHECK_TEMPLATE: &str = r#"#!/bin/bash
# Git check hook for Claude Code
# Ensures all changes are committed and pushed before completing tasks

# Read the JSON input from stdin
input=$(cat)

# Check if stop hook is already active (recursion prevention)
stop_hook_active=$(echo "$input" | jq -r '.stop_hook_active // "false"' 2>/dev/null)
if [[ "$stop_hook_active" = "true" ]]; then
  exit 0
fi

# Check if we're in a git repository - bail if not
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  exit 0
fi

# Check for uncommitted changes (both staged and unstaged)
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "There are uncommitted changes in the repository. Please commit and push these changes to the remote branch." >&2
  exit 2
fi

# Check for untracked files that might be important
untracked_files=$(git ls-files --others --exclude-standard)
if [[ -n "$untracked_files" ]]; then
  echo "There are untracked files in the repository. Please commit and push these changes to the remote branch." >&2
  exit 2
fi

current_branch=$(git branch --show-current)
if [[ -n "$current_branch" ]]; then
  if git rev-parse "origin/$current_branch" >/dev/null 2>&1; then
    # Branch exists on remote - compare against it
    unpushed=$(git rev-list "origin/$current_branch..HEAD" --count 2>/dev/null) || unpushed=0
    if [[ "$unpushed" -gt 0 ]]; then
      echo "There are $unpushed unpushed commit(s) on branch '$current_branch'. Please push these changes to the remote repository." >&2
      exit 2
    fi
  else
    # Branch doesn't exist on remote - compare against default branch
    unpushed=$(git rev-list "origin/HEAD..HEAD" --count 2>/dev/null) || unpushed=0
    if [[ "$unpushed" -gt 0 ]]; then
      echo "Branch '$current_branch' has $unpushed unpushed commit(s) and no remote branch. Please push these changes to the remote repository." >&2
      exit 2
    fi
  fi
fi

exit 0
"#;

/// Task progress tracking hook (Python)
const TRACK_PROGRESS_PY_TEMPLATE: &str = r#"#!/usr/bin/env python3
"""
Claude Code Task Progress Tracker Hook

This hook tracks Claude Code session progress and writes events to a JSONL file
that can be consumed by ccmon UI for real-time task monitoring.

Events tracked:
- SessionStart: Task initialization → (no status)
- UserPromptSubmit: User sent a prompt → in_progress
- PostToolUse: Progress updates on tool execution → in_progress (or error)
- Stop: Response completed, user action needed → stop
- SessionEnd: Session ended → session_ended
- Notification: Permission or input needed → stop

Status mapping:
- in_progress: Claude is actively working
- stop: Response completed, waiting for user action
- session_ended: Session has ended
- error: Tool execution failed

Output format: ~/.claude/task-progress/<session_id>.jsonl

Security: All log files are created with 0o600 permissions (user read/write only)
"""

import json
import sys
import os
import traceback
from pathlib import Path
from datetime import datetime


def get_task_status(hook_event: str, tool_name: str = "", hook_data: dict = None) -> str:
    """
    Determine task status based on hook event and tool name.

    Returns: "in_progress" | "stop" | "session_ended" | "error" | None
    """
    if hook_event == "SessionStart":
        # Don't set status on session start - wait for user prompt
        return None
    elif hook_event == "UserPromptSubmit":
        # User submitted a prompt - task is now in progress
        return "in_progress"
    elif hook_event == "Stop":
        # Response completed, waiting for user action
        return "stop"
    elif hook_event == "SessionEnd":
        # Session has ended
        return "session_ended"
    elif hook_event == "Notification":
        # Permission or input needed
        if hook_data:
            message = hook_data.get("message", "").lower()
            # "Claude needs permission" or "waiting for input"
            if "permission" in message or "waiting" in message or "input" in message:
                return "stop"
        return None  # Other notifications don't change status
    elif hook_event == "PostToolUse":
        # Check for tool errors
        if hook_data:
            tool_result = hook_data.get("tool_result", {})
            # Bash tool errors
            if tool_name == "Bash" and isinstance(tool_result, dict):
                error = tool_result.get("error")
                if error:
                    return "error"
            # Generic tool errors
            if isinstance(tool_result, str) and "error" in tool_result.lower():
                return "error"
        return "in_progress"
    else:
        return "in_progress"


def get_event_message(hook_data: dict) -> str:
    """Generate a human-readable message for the event."""
    event = hook_data.get("hook_event_name", "")
    tool = hook_data.get("tool_name", "")

    if event == "SessionStart":
        return "Session started"
    elif event == "UserPromptSubmit":
        return "Processing user prompt"
    elif event == "SessionEnd":
        return "Session completed"
    elif event == "Stop":
        return "Waiting for user response"
    elif event == "PostToolUse":
        tool_input = hook_data.get("tool_input", {})

        if tool == "Write":
            file_path = tool_input.get("file_path", "")
            return f"Created file: {Path(file_path).name if file_path else 'unknown'}"
        elif tool == "Edit":
            file_path = tool_input.get("file_path", "")
            return f"Edited file: {Path(file_path).name if file_path else 'unknown'}"
        elif tool == "Bash":
            command = tool_input.get("command", "")
            # Truncate long commands
            cmd_preview = command[:50] + "..." if len(command) > 50 else command
            return f"Executed: {cmd_preview}"
        elif tool == "Read":
            file_path = tool_input.get("file_path", "")
            return f"Read file: {Path(file_path).name if file_path else 'unknown'}"
        else:
            return f"Used tool: {tool}"

    return "Unknown event"


def main():
    """Main hook execution."""
    try:
        # Read JSON input from stdin
        hook_data = json.load(sys.stdin)

        session_id = hook_data.get("session_id", "unknown")
        hook_event = hook_data.get("hook_event_name", "")
        tool_name = hook_data.get("tool_name", "")
        cwd = hook_data.get("cwd", "")

        # Create progress directory
        progress_dir = Path.home() / ".claude" / "task-progress"
        progress_dir.mkdir(parents=True, exist_ok=True)

        # Session-specific progress file
        progress_file = progress_dir / f"{session_id}.jsonl"

        # Determine task status (with error detection)
        status = get_task_status(hook_event, tool_name, hook_data)
        message = get_event_message(hook_data)

        # Create event record
        event_record = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "session_id": session_id,
            "event": hook_event,
            "tool": tool_name if tool_name else None,
            "message": message,
            "cwd": cwd
        }

        # Only include status if it's not None
        if status is not None:
            event_record["status"] = status

        # Append to JSONL file with secure permissions (0o600)
        file_exists = progress_file.exists()
        with open(progress_file, "a") as f:
            json.dump(event_record, f)
            f.write("\n")

        # Set secure permissions on new files (user read/write only)
        if not file_exists:
            os.chmod(progress_file, 0o600)

        # For SessionStart, output context message to Claude
        if hook_event == "SessionStart":
            sys.stdout.write("✓ Task progress tracking initialized for ccmon UI")

    except Exception as e:
        # Log errors with full traceback but don't fail the hook
        error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
        error_log.parent.mkdir(parents=True, exist_ok=True)

        # Set secure permissions on error log
        file_exists = error_log.exists()
        with open(error_log, "a") as f:
            f.write(f"\n{'='*60}\n")
            f.write(f"Time: {datetime.utcnow().isoformat()}Z\n")
            f.write(f"Error: {str(e)}\n")
            f.write(f"Traceback:\n")
            f.write(traceback.format_exc())
            f.write(f"{'='*60}\n")

        if not file_exists:
            os.chmod(error_log, 0o600)


if __name__ == "__main__":
    main()
"#;

/// Claude Code hooks ファイルを作成
pub fn create_claude_hooks(dir: &Path, force: bool) -> Result<Vec<PathBuf>> {
    let mut created_files = Vec::new();

    // .claude ディレクトリ作成
    let claude_dir = dir.join(".claude");
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir).with_context(|| {
            format!(
                "Failed to create .claude directory: {}",
                claude_dir.display()
            )
        })?;
    }

    // .claude/hooks ディレクトリ作成
    let hooks_dir = claude_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).with_context(|| {
            format!(
                "Failed to create .claude/hooks directory: {}",
                hooks_dir.display()
            )
        })?;
    }

    // 1. .claude/settings.json
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() && !force {
        anyhow::bail!(
            "Claude Code settings file already exists: {}\n\n\
             Use --force to overwrite.",
            settings_path.display()
        );
    }
    fs::write(&settings_path, CLAUDE_SETTINGS_TEMPLATE).with_context(|| {
        format!(
            "Failed to create Claude Code settings: {}",
            settings_path.display()
        )
    })?;
    created_files.push(settings_path);

    // 2. .claude/hooks/session-init.sh
    let session_init_path = hooks_dir.join("session-init.sh");
    fs::write(&session_init_path, SESSION_INIT_HOOK_TEMPLATE).with_context(|| {
        format!(
            "Failed to create session-init.sh: {}",
            session_init_path.display()
        )
    })?;
    set_executable(&session_init_path)?;
    created_files.push(session_init_path);

    // 3. .claude/hooks/track-progress.py
    let track_progress_path = hooks_dir.join("track-progress.py");
    fs::write(&track_progress_path, TRACK_PROGRESS_PY_TEMPLATE).with_context(|| {
        format!(
            "Failed to create track-progress.py: {}",
            track_progress_path.display()
        )
    })?;
    set_executable(&track_progress_path)?;
    created_files.push(track_progress_path);

    // 4. ~/.claude/stop-hook-git-check.sh
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let home_claude_dir = home_dir.join(".claude");
    if !home_claude_dir.exists() {
        fs::create_dir_all(&home_claude_dir).with_context(|| {
            format!(
                "Failed to create ~/.claude directory: {}",
                home_claude_dir.display()
            )
        })?;
    }

    // Skip if already exists (do not overwrite even with --force)
    let stop_hook_path = home_claude_dir.join("stop-hook-git-check.sh");
    if !stop_hook_path.exists() {
        fs::write(&stop_hook_path, STOP_HOOK_GIT_CHECK_TEMPLATE).with_context(|| {
            format!(
                "Failed to create stop-hook-git-check.sh: {}",
                stop_hook_path.display()
            )
        })?;
        set_executable(&stop_hook_path)?;
        created_files.push(stop_hook_path);
    }

    Ok(created_files)
}
