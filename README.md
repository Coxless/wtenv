# ccmon - Claude Code Monitor

Real-time task progress monitoring for Claude Code parallel development sessions.

## Features

- **Real-time Claude Code task monitoring** - Track multiple Claude Code sessions across worktrees
- **Interactive TUI** - Visual dashboard showing task status, duration, and last activity
- **Desktop notifications** - Get notified when commands complete
- **Auto-refresh** - UI automatically updates every second
- **Claude Code hooks** - Automatic task progress tracking via hooks

### Task Status Display

| Status | Icon | Description |
|--------|------|-------------|
| **In Progress** | 🔵 | Claude is actively working |
| **Stop** | 🟡 | Response completed, waiting for user |
| **Session Ended** | ⚫ | Session has completed |
| **Error** | 🔴 | Task encountered an error |

## Installation

### Requirements

- **Rust** 1.91.0 or later (for building from source)
- **Python** 3.6 or later (for Claude Code hooks)

### From Source

```bash
git clone https://github.com/Coxless/ccmon.git
cd ccmon
cargo install --path .
```

### From Binary

Download from [Releases](https://github.com/Coxless/ccmon/releases) and place in your PATH.

## Quick Start

```bash
# Initialize Claude Code hooks
ccmon init

# Launch interactive TUI
ccmon ui

# Run command with desktop notification
ccmon notify "cargo build"
```

## Setup

### Initialize Hooks

After installation, initialize Claude Code hooks in your repository:

```bash
cd /path/to/your/repo
ccmon init
```

This creates:
- `.claude/settings.json` - Claude Code hook configuration
- `.claude/hooks/session-init.sh` - Session start hook (shows git context)
- `.claude/hooks/track-progress.py` - Task progress tracking hook
- `~/.claude/stop-hook-git-check.sh` - Global stop hook (git status check)

### Enable Hooks Globally

To enable hooks for all projects:

```bash
cp .claude/settings.json ~/.claude/settings.json
```

### Verify Setup

```bash
# Start a Claude Code session in the repository
# Then run in another terminal:
ccmon ui

# You should see your active Claude session!
```

## Commands

### `ccmon init`

Initialize Claude Code hooks in the current directory.

```bash
ccmon init          # Create hooks
ccmon init --force  # Overwrite existing hooks
```

### `ccmon ui`

Launch interactive TUI for monitoring Claude Code tasks.

```bash
ccmon ui
```

#### Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move to next task |
| `k` / `↑` | Move to previous task |
| `r` | Manual refresh |
| `q` / `Esc` | Quit |

#### Display Information

- Session ID
- Working directory
- Current status and duration
- Last activity (tool used, file edited, etc.)

### `ccmon notify <command>`

Execute a command with desktop notification on completion.

```bash
# Run build with notification
ccmon notify "cargo build --release"

# Run tests with notification
ccmon notify "cargo test"

# Run in specific directory
ccmon notify --dir ./project "npm test"

# Control notification behavior
ccmon notify --notify-success=false "npm run lint"  # No success notification
ccmon notify --notify-error=false "make check"      # No error notification
```

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## How It Works

1. `ccmon init` creates hook scripts that Claude Code executes at various points:
   - **SessionStart**: Records session start, shows git context
   - **UserPromptSubmit**: Sets status to in_progress when user sends a prompt
   - **PostToolUse**: Tracks tool usage (Edit, Bash, Read, etc.)
   - **Stop**: Records when Claude is waiting for user input
   - **SessionEnd**: Records session completion

2. Hook events are written to `~/.claude/task-progress/<session_id>.jsonl`

3. `ccmon ui` reads these files and displays real-time status

## License

MIT
