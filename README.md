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

## Installation

### From Source

```bash
git clone https://github.com/USERNAME/wtenv.git
cd wtenv
cargo install --path .
```

### From Binary

Download from [Releases](https://github.com/USERNAME/wtenv/releases) and place in your PATH.

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

Manage worktrees with an interactive TUI.

```bash
# Launch TUI
wtenv ui
```

**Key bindings:**
- `↑/↓` or `j/k`: Navigate worktrees
- `r`: Refresh status
- `q` or `Esc`: Quit

**Features:**
- List all worktrees with status
- Display detailed information for selected worktree
- Real-time process count display
- Keyboard navigation

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

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## License

MIT
