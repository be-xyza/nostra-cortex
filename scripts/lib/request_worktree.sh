#!/usr/bin/env bash

resolve_request_repo_context() {
  local target_input="${1:-$PWD}"
  local target_dir worktree_top common_dir

  target_dir="$(cd "$target_input" && pwd)"
  git -C "$target_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1 || {
    echo "FAIL: not inside a git worktree: $target_dir" >&2
    return 1
  }

  worktree_top="$(git -C "$target_dir" rev-parse --show-toplevel)"
  common_dir="$(git -C "$target_dir" rev-parse --path-format=absolute --git-common-dir 2>/dev/null || git -C "$target_dir" rev-parse --git-common-dir)"

  export REQUEST_TARGET_DIR="$target_dir"
  export REQUEST_WORKTREE_TOP="$(cd "$worktree_top" && pwd)"
  export REQUEST_GIT_COMMON_DIR="$(cd "$common_dir" && pwd)"
  export REQUEST_REPO_ROOT="$(cd "$REQUEST_GIT_COMMON_DIR/.." && pwd)"
  export REQUEST_IS_ROOT_WORKTREE=0
  export REQUEST_IS_CANONICAL_REQUEST_WORKTREE=0

  if [[ "$REQUEST_WORKTREE_TOP" == "$REQUEST_REPO_ROOT" ]]; then
    export REQUEST_IS_ROOT_WORKTREE=1
  fi
  if [[ "$REQUEST_WORKTREE_TOP" == "$REQUEST_REPO_ROOT/.worktrees/"* ]]; then
    export REQUEST_IS_CANONICAL_REQUEST_WORKTREE=1
  fi
}

current_request_branch() {
  git -C "$REQUEST_WORKTREE_TOP" rev-parse --abbrev-ref HEAD
}

request_status_porcelain() {
  git -C "$REQUEST_WORKTREE_TOP" status --porcelain --untracked-files=all
}

request_upstream_ref() {
  git -C "$REQUEST_WORKTREE_TOP" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true
}

request_ahead_behind() {
  local upstream ahead behind
  upstream="$(request_upstream_ref)"
  if [[ -z "$upstream" ]]; then
    printf 'none 0 0\n'
    return 0
  fi

  read -r ahead behind < <(git -C "$REQUEST_WORKTREE_TOP" rev-list --left-right --count "${upstream}...HEAD")
  printf '%s %s %s\n' "$upstream" "$ahead" "$behind"
}

ensure_request_worktree_or_fail() {
  if [[ "$REQUEST_IS_ROOT_WORKTREE" -eq 1 ]]; then
    echo "FAIL: root worktree is reserved for repo-wide stewardship tasks; use .worktrees/ for request work" >&2
    return 1
  fi
  if [[ "$REQUEST_IS_CANONICAL_REQUEST_WORKTREE" -ne 1 ]]; then
    echo "FAIL: request worktree must live under $REQUEST_REPO_ROOT/.worktrees (actual: $REQUEST_WORKTREE_TOP)" >&2
    return 1
  fi
}
