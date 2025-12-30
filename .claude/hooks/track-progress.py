#!/usr/bin/env python3
"""
Claude Code Task Progress Tracker Hook

This hook tracks Claude Code session progress and writes events to a JSONL file
that can be consumed by wtenv UI for real-time task monitoring.

Events tracked:
- SessionStart: Task initialization
- PostToolUse: Progress updates on tool execution
- Stop: User response required or task paused
- SessionEnd: Task completion

Output format: ~/.claude/task-progress/<session_id>.jsonl
"""

import json
import sys
import os
from pathlib import Path
from datetime import datetime


def get_task_status(hook_event: str, tool_name: str = "") -> str:
    """Determine task status based on hook event and tool name."""
    if hook_event == "SessionStart":
        return "in_progress"
    elif hook_event == "Stop":
        return "waiting_user"
    elif hook_event == "SessionEnd":
        return "completed"
    elif hook_event == "PostToolUse" and tool_name == "Bash":
        # Bash commands might indicate active work
        return "in_progress"
    else:
        return "in_progress"


def get_event_message(hook_data: dict) -> str:
    """Generate a human-readable message for the event."""
    event = hook_data.get("hook_event_name", "")
    tool = hook_data.get("tool_name", "")

    if event == "SessionStart":
        return "Session started"
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
            "cwd": cwd
        }

        # Append to JSONL file
        with open(progress_file, "a") as f:
            json.dump(event_record, f)
            f.write("\n")

        # For SessionStart, output context message to Claude
        if hook_event == "SessionStart":
            sys.stdout.write("✓ Task progress tracking initialized for wtenv UI")

    except Exception as e:
        # Log errors but don't fail the hook
        error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
        error_log.parent.mkdir(parents=True, exist_ok=True)

        with open(error_log, "a") as f:
            f.write(f"{datetime.utcnow().isoformat()}: {str(e)}\n")


if __name__ == "__main__":
    main()
