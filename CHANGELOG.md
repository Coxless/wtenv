# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-30

### Added

- Initial release
- Core worktree operations:
  - `create` - Create new worktree with optional branch
  - `list` - List all worktrees
  - `remove` - Remove worktree with confirmation
  - `init` - Initialize configuration file
  - `config` - Display current configuration
- Configuration file support (YAML):
  - `.worktree.yml` / `.worktree.yaml`
  - File copying with glob patterns
  - Exclude patterns
  - Post-create command execution
- Interactive mode:
  - Branch name prompt when not specified
  - Worktree path confirmation
  - Delete confirmation dialog
  - Overwrite confirmation dialog
- UX features:
  - `--verbose` flag for detailed output
  - `--quiet` flag for silent mode
  - Progress bar for file operations
  - Colored output
  - Japanese error messages
- Cross-platform support:
  - Linux (x64)
  - macOS (x64, arm64)
  - Windows (x64)
