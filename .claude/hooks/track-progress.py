#!/usr/bin/env python3
"""
Claude Code Task Progress Tracker Hook

This hook tracks Claude Code session progress and writes events to a JSONL file
that can be consumed by wtenv UI for real-time task monitoring.

Events tracked:
- SessionStart: Task initialization → in_progress
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

    Returns: "in_progress" | "stop" | "session_ended" | "error"
    """
    if hook_event == "SessionStart":
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
            sys.stdout.write("✓ Task progress tracking initialized for wtenv UI")

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
