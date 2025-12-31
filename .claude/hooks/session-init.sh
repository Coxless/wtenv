#!/bin/bash
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
