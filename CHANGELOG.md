# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-30

### Added

- Initial release
- **Core worktree operations:**
  - `create` - Create new worktree with optional branch
  - `list` - List all worktrees
  - `remove` - Remove worktree with confirmation
  - `init` - Initialize configuration file
  - `config` - Display current configuration
- **Monitoring and process management (Phase 1):**
  - `status` - Display detailed status of all worktrees with process information
  - `ps` - List all running processes in worktrees
  - `kill` - Stop running processes by PID, worktree, or all at once
  - Persistent process tracking across terminal sessions
- **TUI and environment management (Phase 2):**
  - `ui` - Interactive terminal UI for worktree management
  - `diff-env` - Compare environment variables between worktrees
  - Real-time keyboard navigation in TUI
  - Support for comparing all worktrees at once
- **Analytics and cleanup (Phase 3):**
  - `analyze` - Analyze worktree disk usage, dependencies, and merge status
  - `clean` - Clean up merged or stale worktrees with dry-run support
  - `notify` - Execute commands with desktop notifications
  - Automatic detection of merged branches
  - Configurable stale worktree detection
- **GitHub integration (Phase 4):**
  - `pr` - Create worktree directly from GitHub Pull Request
  - Automatic PR information fetching via GitHub CLI
  - Automatic remote branch fetching
  - Integration with existing worktree creation workflow
- **Configuration file support (YAML):**
  - `.worktree.yml` / `.worktree.yaml`
  - File copying with glob patterns
  - Exclude patterns
  - Post-create command execution with optional flag
- **Interactive mode:**
  - Branch name prompt when not specified
  - Worktree path confirmation
  - Delete confirmation dialog
  - Overwrite confirmation dialog
- **UX features:**
  - `--verbose` flag for detailed output
  - `--quiet` flag for silent mode
  - Progress bar for file operations
  - Colored output with emoji support
  - Japanese error messages
  - Formatted output with borders and tables
- **Cross-platform support:**
  - Linux (x64)
  - macOS (x64, arm64)
  - Windows (x64)
