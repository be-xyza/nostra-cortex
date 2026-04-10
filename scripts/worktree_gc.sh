#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

echo "== worktree list (before prune)"
git -C "$ROOT_DIR" worktree list
echo
echo "== pruning stale worktrees"
git -C "$ROOT_DIR" worktree prune --verbose
echo
echo "== worktree list (after prune)"
git -C "$ROOT_DIR" worktree list
