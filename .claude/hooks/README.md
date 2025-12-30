# Claude Code Task Progress Tracking

This directory contains hooks for tracking Claude Code task progress in wtenv UI.

## Setup

1. Copy the example settings file to your Claude Code configuration:

```bash
# For project-level configuration
cp .claude/settings.json.example .claude/settings.json

# For user-level configuration
cp .claude/settings.json.example ~/.claude/settings.json
```

2. Make sure the hook script is executable:

```bash
chmod +x .claude/hooks/track-progress.py
```

## How It Works

The `track-progress.py` hook intercepts Claude Code events and writes progress data to:

```
~/.claude/task-progress/<session_id>.jsonl
```

### Events Tracked

- **SessionStart**: Task initialization
- **PostToolUse**: Progress updates when Claude uses tools (Write, Edit, Bash, etc.)
- **Stop**: User response required
- **SessionEnd**: Task completion

### Data Format

Each line in the JSONL file contains:

```json
{
  "timestamp": "2025-12-30T10:30:45Z",
  "session_id": "abc123",
  "event": "PostToolUse",
  "tool": "Write",
  "status": "in_progress",
  "message": "Created file: main.rs",
  "cwd": "/home/user/wtenv"
}
```

### Status Values

- `in_progress`: Claude is actively working
- `waiting_user`: Claude is waiting for user response (Stop event)
- `completed`: Session has ended
- `error`: An error occurred

## Integration with wtenv UI

Run `wtenv ui` to see real-time Claude Code task progress across all your worktrees.

The UI will display:
- Active Claude Code sessions
- Current task status
- Last activity timestamp
- Worktree association

## Troubleshooting

If hooks are not working:

1. Check that Python 3 is available: `python3 --version`
2. Verify hook script permissions: `ls -l .claude/hooks/track-progress.py`
3. Check error logs: `cat ~/.claude/task-progress/errors.log`
4. Test hook manually:
   ```bash
   echo '{"session_id":"test","hook_event_name":"SessionStart","cwd":"/tmp"}' | .claude/hooks/track-progress.py
   cat ~/.claude/task-progress/test.jsonl
   ```

## Privacy Note

Progress data is stored locally in `~/.claude/task-progress/` and is never sent to external services.
