#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

BRANCH_NAME=""
WORKTREE_NAME=""
BASE_REF="HEAD"
ALLOW_DIRTY_ROOT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch)
      BRANCH_NAME="${2:?missing value for --branch}"
      shift 2
      ;;
    --name)
      WORKTREE_NAME="${2:?missing value for --name}"
      shift 2
      ;;
    --base)
      BASE_REF="${2:?missing value for --base}"
      shift 2
      ;;
    --allow-dirty-root)
      ALLOW_DIRTY_ROOT=1
      shift
      ;;
    *)
      echo "usage: start_request_worktree.sh --branch <branch> [--name <dir>] [--base <ref>] [--allow-dirty-root]" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$BRANCH_NAME" ]]; then
  echo "FAIL: --branch is required" >&2
  exit 1
fi

WORKTREE_NAME="${WORKTREE_NAME:-${BRANCH_NAME#codex/}}"
TARGET_PATH="$ROOT_DIR/.worktrees/$WORKTREE_NAME"
PROBE_PATH="$ROOT_DIR/.worktrees/.ignore-probe-$WORKTREE_NAME"

git -C "$ROOT_DIR" check-ignore -q "$PROBE_PATH" || {
  echo "FAIL: .worktrees/ is not ignored by git; fix .gitignore before creating request worktrees" >&2
  exit 1
}

if [[ "$ALLOW_DIRTY_ROOT" -ne 1 && -n "$(git -C "$ROOT_DIR" status --porcelain --untracked-files=all)" ]]; then
  echo "FAIL: root worktree is dirty; create a recovery snapshot or pass --allow-dirty-root for a repo-wide stewardship task" >&2
  exit 1
fi

if [[ -e "$TARGET_PATH" ]]; then
  echo "FAIL: target worktree path already exists: $TARGET_PATH" >&2
  exit 1
fi

if git -C "$ROOT_DIR" show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
  echo "FAIL: branch already exists: $BRANCH_NAME" >&2
  exit 1
fi

git -C "$ROOT_DIR" worktree prune
git -C "$ROOT_DIR" worktree add "$TARGET_PATH" -b "$BRANCH_NAME" "$BASE_REF"
bash "$TARGET_PATH/scripts/check_clean_worktree.sh" --path "$TARGET_PATH"

echo "PASS: request worktree ready"
echo "branch=$BRANCH_NAME"
echo "path=$TARGET_PATH"
