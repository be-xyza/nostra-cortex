#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
source "$ROOT_DIR/scripts/lib/request_worktree.sh"

TARGET_PATH="."
ALLOW_ROOT=0
ALLOW_STAGED_ONLY=0

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
    --allow-staged-only)
      ALLOW_STAGED_ONLY=1
      shift
      ;;
    *)
      echo "usage: close_request.sh [--path <dir>] [--allow-root] [--allow-staged-only]" >&2
      exit 2
      ;;
  esac
done

resolve_request_repo_context "$TARGET_PATH"
if [[ "$REQUEST_IS_ROOT_WORKTREE" -eq 1 && "$ALLOW_ROOT" -ne 1 ]]; then
  ensure_request_worktree_or_fail
fi

branch="$(current_request_branch)"
unstaged="$(git -C "$REQUEST_WORKTREE_TOP" diff --name-only)"
staged="$(git -C "$REQUEST_WORKTREE_TOP" diff --cached --name-only)"
untracked="$(git -C "$REQUEST_WORKTREE_TOP" ls-files --others --exclude-standard)"

if [[ -n "$unstaged" || -n "$untracked" ]]; then
  echo "FAIL: request has unstaged or untracked work; create a checkpoint or commit before closeout" >&2
  exit 1
fi

if [[ -n "$staged" && "$ALLOW_STAGED_ONLY" -ne 1 ]]; then
  echo "FAIL: request has staged but uncommitted work; pass --allow-staged-only only when you intend to hand off a staged closeout state" >&2
  exit 1
fi

declare -a checks=(
  "bash scripts/check_workspace_merge_integrity.sh"
  "bash scripts/check_dynamic_config_contract.sh"
  "bash scripts/check_tracked_generated_artifacts.sh"
)

for cmd in "${checks[@]}"; do
  (cd "$REQUEST_WORKTREE_TOP" && eval "$cmd")
done

if [[ -z "$staged" ]]; then
  bash "$REQUEST_WORKTREE_TOP/scripts/check_clean_worktree.sh" --path "$REQUEST_WORKTREE_TOP"
fi

read -r upstream ahead behind < <(request_ahead_behind)
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
evidence_dir="$REQUEST_REPO_ROOT/logs/alignment"
mkdir -p "$evidence_dir"
evidence_file="$evidence_dir/request_closeout_$timestamp.json"
latest_file="$evidence_dir/request_closeout_latest.json"

cat > "$evidence_file" <<EOF
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "branch": "$branch",
  "worktree_top": "$REQUEST_WORKTREE_TOP",
  "status": "$(if [[ -n "$staged" ]]; then printf staged_only; else printf clean; fi)",
  "upstream": "$upstream",
  "ahead": $ahead,
  "behind": $behind
}
EOF
cp "$evidence_file" "$latest_file"

echo "PASS: request closeout checks completed"
echo "evidence=$evidence_file"
if [[ "$upstream" == "none" ]]; then
  echo "WARN: branch has no upstream; push before deletion or merge"
elif [[ "$ahead" -gt 0 ]]; then
  echo "WARN: branch is ahead of upstream by $ahead commit(s); push before deletion or merge"
fi
