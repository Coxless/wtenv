# wtenv - Git Worktree Environment Manager

Fast and user-friendly git worktree management CLI tool.

## Features

- Easy worktree creation with branch management
- Automatic environment file copying (based on config)
- Post-create command execution
- Interactive mode (no arguments required)
- Progress indicators and colored output
- Verbose and quiet output modes

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

### `wtenv create [BRANCH] [PATH]`

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

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## License

MIT
