#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
source "$ROOT_DIR/scripts/lib/request_worktree.sh"

TARGET_PATH="."
ALLOW_ROOT=0
OUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)
      TARGET_PATH="${2:?missing value for --path}"
      shift 2
      ;;
    --allow-root)
      ALLOW_ROOT=1
      shift
      ;;
    --out-dir)
      OUT_DIR="${2:?missing value for --out-dir}"
      shift 2
      ;;
    *)
      echo "usage: checkpoint_request.sh [--path <dir>] [--allow-root] [--out-dir <dir>]" >&2
      exit 2
      ;;
  esac
done

resolve_request_repo_context "$TARGET_PATH"
if [[ "$REQUEST_IS_ROOT_WORKTREE" -eq 1 && "$ALLOW_ROOT" -ne 1 ]]; then
  ensure_request_worktree_or_fail
fi

branch="$(current_request_branch)"
status="$(request_status_porcelain)"
read -r upstream ahead behind < <(request_ahead_behind)
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
slug="${branch//\//-}"
OUT_DIR="${OUT_DIR:-/tmp/icp-request-checkpoint-$slug-$timestamp}"
mkdir -p "$OUT_DIR"

printf '%s\n' "$status" > "$OUT_DIR/status.porcelain"
git -C "$REQUEST_WORKTREE_TOP" diff --binary > "$OUT_DIR/tracked_changes.patch"
git -C "$REQUEST_WORKTREE_TOP" diff --binary --cached > "$OUT_DIR/staged_changes.patch"
git -C "$REQUEST_WORKTREE_TOP" ls-files --others --exclude-standard > "$OUT_DIR/untracked_manifest.txt"

cat > "$OUT_DIR/summary.txt" <<EOF
branch=$branch
worktree_top=$REQUEST_WORKTREE_TOP
upstream=$upstream
ahead=$ahead
behind=$behind
checkpoint_mode=patch_bundle
EOF

echo "PASS: checkpoint bundle created"
echo "out_dir=$OUT_DIR"
if [[ -n "$status" ]]; then
  echo "WARN: worktree still has uncommitted changes; the patch bundle is durable, but prefer a WIP commit + push for non-trivial work"
fi
if [[ "$upstream" == "none" ]]; then
  echo "WARN: branch has no upstream; pushing is recommended before handoff"
elif [[ "$ahead" -gt 0 ]]; then
  echo "WARN: branch is ahead of upstream by $ahead commit(s); pushing is recommended before handoff"
fi
