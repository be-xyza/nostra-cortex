#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
source "$ROOT_DIR/scripts/lib/request_worktree.sh"

TARGET_PATH="."
ALLOW_ROOT_DIRTY=0
ALLOW_EXTERNAL_WORKTREE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)
      TARGET_PATH="${2:?missing value for --path}"
      shift 2
      ;;
    --allow-root-dirty)
      ALLOW_ROOT_DIRTY=1
      shift
      ;;
    --allow-external-worktree)
      ALLOW_EXTERNAL_WORKTREE=1
      shift
      ;;
    *)
      echo "usage: check_clean_worktree.sh [--path <dir>] [--allow-root-dirty] [--allow-external-worktree]" >&2
      exit 2
      ;;
  esac
done

resolve_request_repo_context "$TARGET_PATH"

if [[ "$REQUEST_IS_ROOT_WORKTREE" -eq 1 && "$ALLOW_ROOT_DIRTY" -ne 1 ]]; then
  echo "FAIL: root worktree is reserved for repo-wide stewardship tasks; use .worktrees/ or pass --allow-root-dirty" >&2
  exit 1
fi

if [[ "$REQUEST_IS_ROOT_WORKTREE" -eq 0 && "$REQUEST_IS_CANONICAL_REQUEST_WORKTREE" -ne 1 && "$ALLOW_EXTERNAL_WORKTREE" -ne 1 ]]; then
  echo "FAIL: request worktree must live under $REQUEST_REPO_ROOT/.worktrees (actual: $REQUEST_WORKTREE_TOP)" >&2
  exit 1
fi

status="$(request_status_porcelain)"
if [[ -n "$status" ]]; then
  printf '%s\n' "$status"
  echo "FAIL: worktree is not clean" >&2
  exit 1
fi

echo "PASS: worktree is clean"
echo "worktree_top=$REQUEST_WORKTREE_TOP"
echo "repo_root=$REQUEST_REPO_ROOT"
