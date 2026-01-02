# wtenv - Git Worktree Environment Manager

> **Warning**
> This tool is under development and not stable. Please use with caution.

Fast and user-friendly git worktree management CLI tool with **parallel development control center** features.

## Features

### Core Worktree Management
- Easy worktree creation with branch management
- Automatic environment file copying (based on config)
- Post-create command execution
- Interactive mode (no arguments required)
- Progress indicators and colored output
- Verbose and quiet output modes

### **NEW: Parallel Development Control Center** 🚀
- **Real-time worktree status monitoring** - See all worktrees at a glance with file changes and commit info
- **Process management** - Track and manage processes running in each worktree
- **Process control** - Kill processes by PID, worktree, or all at once
- **Persistent process tracking** - Process information survives terminal sessions
- **Claude Code integration** 🤖 - Track Claude Code task progress across all worktrees in real-time
  - Monitor active AI coding sessions
  - Get notified when Claude needs your response
  - View task duration and status at a glance

## Installation

### Requirements

- **Rust** 1.91.0 or later (for building from source)
- **Python** 3.6 or later (for Claude Code integration hooks)
- **Git** 2.17 or later (for worktree support)
- **GitHub CLI** (`gh`) - Optional, required only for `wtenv pr` command

### From Source

```bash
git clone https://github.com/USERNAME/wtenv.git
cd wtenv
cargo install --path .
```

### From Binary

Download from [Releases](https://github.com/USERNAME/wtenv/releases) and place in your PATH.

## Setup

### Basic Setup

After installation, initialize the configuration file in your repository:

```bash
# Navigate to your git repository
cd /path/to/your/repo

# Initialize wtenv configuration
wtenv init
```

This creates a `.worktree.yml` file where you can configure file copying and post-create commands.

### Claude Code Hooks Setup

To enable Claude Code integration for real-time task tracking:

#### 1. Generate Hook Files

```bash
# Generate hooks and configuration files
wtenv init --hooks
```

This creates:
- `.claude/settings.json` - Claude Code hook configuration
- `.claude/hooks/session-init.sh` - Session start hook (shows git context)
- `.claude/hooks/track-progress.py` - Task progress tracking hook (Python)
- `~/.claude/stop-hook-git-check.sh` - Global stop hook (git status check)

#### 2. Verify Python Installation

Ensure Python 3.6+ is available:

```bash
python3 --version
```

#### 3. Make Hook Scripts Executable

```bash
chmod +x .claude/hooks/session-init.sh
chmod +x .claude/hooks/track-progress.py
chmod +x ~/.claude/stop-hook-git-check.sh
```

#### 4. Enable Hooks in Claude Code

**Option A: Project-level (Recommended)**

The hooks are automatically enabled for this project after running `wtenv init --hooks`.

**Option B: Global (All Projects)**

To enable hooks for all projects:

```bash
# Copy to global Claude Code settings
cp .claude/settings.json ~/.claude/settings.json
```

#### 5. Start Using

Once configured, Claude Code will:
- ✅ Show development context at session start
- ✅ Track task progress in real-time
- ✅ Verify git status before stopping
- ✅ Display tasks in `wtenv ui`

**Verify it's working:**

```bash
# Start a Claude Code session in the repository
# Then run in another terminal:
wtenv ui

# You should see your active Claude session!
```

## Quick Start

```bash
# Initialize config file
wtenv init

# Create worktree (interactive mode)
wtenv create

# Create worktree with branch name
wtenv create feature-branch

# List worktrees
wtenv list

# Remove worktree
wtenv remove ../feature-branch
```

## Configuration

Create `.worktree.yml` in your repository root:

```yaml
version: 1

copy:
  - .env
  - .env.local
  - config/*.local.json

exclude:
  - .env.production

postCreate:
  - command: npm install
    description: "Installing dependencies..."
  - command: npm run build
    description: "Building project..."
    optional: true
```

### Configuration Options

| Field | Description |
|-------|-------------|
| `version` | Config file version (currently: 1) |
| `copy` | Glob patterns for files to copy |
| `exclude` | Glob patterns for files to exclude |
| `postCreate` | Commands to run after worktree creation |

### Post-Create Command Options

| Field | Description |
|-------|-------------|
| `command` | Shell command to execute |
| `description` | Description shown during execution |
| `optional` | If true, failure won't stop the process |

## Commands

### Monitoring & Control Commands

#### `wtenv status`

Display detailed status of all worktrees with process information.

```bash
# Show worktree overview
wtenv status

# Verbose mode (shows full paths)
wtenv status --verbose
```

**Output example:**
```
┌─────────────────────────────────────────────────────────────┐
│ Worktrees Overview (3 active, 2 processes)                  │
├─────────────────────────────────────────────────────────────┤
│ 🔄 feature-a                      main → feature-a          │
│    Status: Modified (3 files)     Process: pnpm test        │
│    Modified: 3 files  |  Last commit: 2h ago                │
│                                                              │
│ 🔨 feature-b                      main → feature-b          │
│    Status: Running                Process: pnpm build       │
│    Modified: 1 file   |  Last commit: 30m ago               │
│                                                              │
│ ✅ bugfix-123                     main → bugfix-123         │
│    Status: Clean                  No process                │
│    Last commit: 5m ago                                      │
├─────────────────────────────────────────────────────────────┤
│ 📊 Total: 3 worktrees  |  Modified: 4 files                │
└─────────────────────────────────────────────────────────────┘
```

#### `wtenv ps [FILTER]`

List all running processes in worktrees.

```bash
# Show all processes
wtenv ps

# Filter by worktree/branch name
wtenv ps feature-a
```

**Output example:**
```
Active Processes in Worktrees:

feature-a (PID: 12345)
  Command: pnpm test:e2e
  Started: 9m 12s ago
  Working Dir: /home/user/projects/myapp-feature-a
  Status: Running

Total: 1 process
```

#### `wtenv kill [OPTIONS]`

Stop running processes.

```bash
# Kill specific PID
wtenv kill 12345

# Kill all processes
wtenv kill --all

# Kill processes in specific worktree
wtenv kill feature-a
```

### Worktree Management Commands

#### `wtenv create [BRANCH] [PATH]`

Create a new worktree.

```bash
# Interactive mode
wtenv create

# Specify branch (path defaults to ../branch-name)
wtenv create feature-auth

# Specify branch and path
wtenv create feature-auth ~/projects/feature-auth

# Skip file copying
wtenv create feature-auth --no-copy

# Skip post-create commands
wtenv create feature-auth --no-post-create
```

### `wtenv list`

List all worktrees.

```bash
wtenv list

# Verbose mode (shows full commit hash)
wtenv list --verbose
```

### `wtenv remove <PATH>`

Remove a worktree.

```bash
# Interactive confirmation
wtenv remove ../feature-branch

# Force removal (no confirmation)
wtenv remove ../feature-branch --force
```

### `wtenv init`

Initialize configuration file.

```bash
wtenv init

# Overwrite existing config
wtenv init --force
```

### `wtenv config`

Display current configuration.

```bash
wtenv config

# Show detailed information
wtenv config --verbose
```

### `wtenv diff-env`

Display environment variable differences between worktrees.

```bash
# Compare environment variables between two worktrees
wtenv diff-env feature-a feature-b

# Compare environment variables across all worktrees
wtenv diff-env --all
```

**Output example:**
```
🔍 Differences in environment variables between feature-a and feature-b:

.env:
  API_PORT:
    - 3001
    + 3002
  DATABASE_URL:
    - postgresql://localhost/auth_db
    + postgresql://localhost/payment_db

.env.local:
  DEBUG (feature-a only)
    - true
```

### `wtenv ui`

Manage worktrees with an interactive TUI, including real-time Claude Code task monitoring.

```bash
# Launch TUI
wtenv ui
```

#### Interface Overview

The UI is divided into three main sections:

```
┌─────────────────────────────────────────────────────────────┐
│ Worktrees (3) | Processes (2) | Claude Tasks (1)            │ ← Header
├─────────────────────────────────────────────────────────────┤
│ > feature-auth     ✓ Clean        Process: npm test         │ ← Worktree List
│   bugfix-123       ⚠ Modified     No process                │   (Left Panel)
│   feature-payment  🔄 Running     Process: pnpm build        │
├─────────────────────────────────────────────────────────────┤
│ Worktree Details: feature-auth                              │ ← Details Panel
│ Branch: main → feature-auth                                 │   (Right Panel)
│ Path: /home/user/projects/myapp-feature-auth               │
│ Modified: 0 files | Staged: 0 files                         │
│ Last commit: 5m ago                                         │
│                                                              │
│ Active Processes: 1                                         │
│   PID 12345: npm test (Running for 9m 12s)                  │
│                                                              │
│ Claude Code Tasks:                                          │
│   🔵 feature-auth (In Progress) - 15m 30s                   │
│      Last: Edit(src/auth.rs) - 2s ago                       │
└─────────────────────────────────────────────────────────────┘
Press 'r' to refresh | 'q' to quit                            ← Footer
```

#### Key Bindings

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate worktrees (move up/down) |
| `j/k` | Vim-style navigation (down/up) |
| `r` | **Refresh** - Reload worktrees, processes, and Claude tasks |
| `q` or `Esc` | Quit the UI |
| `Enter` | View detailed information for selected worktree |

#### Worktree Status Icons

| Icon | Status | Description |
|------|--------|-------------|
| ✓ | **Clean** | No modified files, all changes committed |
| ⚠ | **Modified** | Files have been changed but not committed |
| 🔄 | **Running** | Process is actively running in this worktree |
| 📝 | **Staged** | Changes are staged for commit |
| 🔀 | **Ahead** | Local commits not pushed to remote |
| 🔽 | **Behind** | Remote commits not pulled locally |

#### Claude Code Task Status

The UI displays real-time status of Claude Code sessions across all worktrees:

| Status | Icon | Description |
|--------|------|-------------|
| **In Progress** | 🔵 | Claude is actively working on tasks |
| **Stop** | 🟡 | Claude has stopped and may need user input |
| **Session Ended** | ⚫ | Session has completed normally |
| **Error** | 🔴 | Task encountered an error |

**Task Information Displayed:**
- Worktree/branch name where Claude is working
- Current status and duration
- Last activity (e.g., "Edit(src/main.rs)", "Bash(cargo build)")
- Time since last activity

**Auto-refresh:**
The UI automatically refreshes Claude task status every 5 seconds to show real-time updates.

#### Process Information

For each worktree with running processes:
- **PID**: Process ID
- **Command**: Full command line
- **Duration**: How long the process has been running
- **Status**: Running, Stopped, or Zombie

#### Usage Tips

1. **Monitor Multiple Worktrees**: See all your parallel development branches at a glance
2. **Track Long-running Processes**: Keep an eye on builds, tests, or dev servers
3. **Claude Code Integration**: Know exactly what Claude is doing and when it needs your input
4. **Quick Refresh**: Press `r` anytime to get the latest status
5. **Keyboard-friendly**: Navigate entirely with keyboard for speed

#### Setting Up Claude Code Integration

To see Claude Code tasks in the UI, you must set up hooks first:

```bash
# Generate hook files (if not already done)
wtenv init --hooks

# Verify hooks are executable
chmod +x .claude/hooks/track-progress.py

# Start using Claude Code
# Tasks will automatically appear in `wtenv ui`
```

See the [Setup](#setup) section for detailed hook configuration instructions.

### `wtenv analyze`

Analyze worktree status, disk usage, and dependencies.

```bash
# Analyze worktrees
wtenv analyze

# Show detailed information
wtenv analyze --detailed
```

**Output example:**
```
📊 Worktree Analysis

  feature-auth
    Disk: 12.45 MB
    Last update: 2 days ago
    Tags: node_modules, lockfile, build

  feature-payment
    Disk: 8.32 MB
    Last update: Yesterday
    Tags: node_modules, lockfile, merged

Summary
  Total worktrees: 3
  Total disk usage: 35.12 MB
  Merged branches: 1
  Stale (>30 days): 0
```

### `wtenv clean`

Clean up merged or stale worktrees.

```bash
# Dry run (show candidates)
wtenv clean --dry-run

# Remove only merged branches
wtenv clean --merged-only

# Remove worktrees not updated in 30 days
wtenv clean --stale-days 30

# Force removal without confirmation
wtenv clean --force
```

### `wtenv notify`

Execute commands with desktop notifications.

```bash
# Run build with notification
wtenv notify "npm run build"

# Run command in specific directory
wtenv notify --dir ./worktrees/feature-a "npm test"

# Notify only on success
wtenv notify --notify-error false "npm run deploy"
```

### `wtenv pr`

Create worktree from GitHub PR. Requires GitHub CLI (`gh`).

```bash
# Create worktree from PR #123
wtenv pr 123

# Specify custom path
wtenv pr 456 /path/to/worktree
```

**Features:**
- Automatically fetch PR information using GitHub CLI
- Automatically fetch remote branch
- Automatically create worktree
- Automatically copy environment files
- Automatically run post-create commands

**Requirements:**
- GitHub CLI (`gh`) must be installed
- Must be authenticated with GitHub CLI (`gh auth login`)

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## License

MIT
