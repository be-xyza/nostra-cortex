#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
GIT_BIN="${GIT_BIN:-git}"
SSH_BIN="${SSH_BIN:-ssh}"
HOST_ALIAS="${NOSTRA_EUDAEMON_VPS_HOST:-eudaemon-alpha-hetzner}"
SSH_EXTRA_ARGS_RAW="${NOSTRA_EUDAEMON_VPS_SSH_ARGS:-}"
PROMOTABLE_REF="${NOSTRA_EUDAEMON_PROMOTABLE_REF:-origin/main}"
REMOTE_REPO_ROOT="${NOSTRA_EUDAEMON_VPS_REPO_ROOT:-/srv/nostra/eudaemon-alpha/repo}"
REMOTE_DEPLOY_SCRIPT="${NOSTRA_EUDAEMON_VPS_DEPLOY_SCRIPT:-$REMOTE_REPO_ROOT/ops/hetzner/deploy.sh}"
REMOTE_CHECK_SCRIPT="${NOSTRA_EUDAEMON_VPS_CHECK_SCRIPT:-$REMOTE_REPO_ROOT/scripts/check_vps_runtime_authority.sh}"
REMOTE_MANIFEST_PATH="${NOSTRA_EUDAEMON_VPS_MANIFEST_PATH:-/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json}"
FETCH_REMOTE="${NOSTRA_EUDAEMON_PROMOTE_FETCH_REMOTE:-1}"
SSH_EXTRA_ARGS=()

if [[ -n "${SSH_EXTRA_ARGS_RAW// }" ]]; then
  # shellcheck disable=SC2206
  SSH_EXTRA_ARGS=($SSH_EXTRA_ARGS_RAW)
fi

usage() {
  cat <<EOF2
Usage: $(basename "$0") [commit-ish]

Promote a commit from origin/main to the Eudaemon Alpha VPS using operator-local SSH.
If no commit is provided, the script promotes the current origin/main commit.

Optional environment overrides:
  NOSTRA_EUDAEMON_VPS_HOST      SSH alias or direct host token.
  NOSTRA_EUDAEMON_VPS_SSH_ARGS  Extra ssh arguments (for example: -F /path/to/config -p 2222).
EOF2
}

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ "$#" -gt 1 ]]; then
  usage >&2
  exit 2
fi

if [[ "$FETCH_REMOTE" == "1" ]]; then
  "$GIT_BIN" -C "$ROOT_DIR" fetch origin '+refs/heads/*:refs/remotes/origin/*' >/dev/null
fi

target_ref="${1:-$PROMOTABLE_REF}"
if ! target_commit="$($GIT_BIN -C "$ROOT_DIR" rev-parse --verify "${target_ref}^{commit}" 2>/dev/null)"; then
  fail "unable to resolve target commit from $target_ref"
fi

promotable_commit="$($GIT_BIN -C "$ROOT_DIR" rev-parse --verify "${PROMOTABLE_REF}^{commit}")"
if ! "$GIT_BIN" -C "$ROOT_DIR" merge-base --is-ancestor "$target_commit" "$promotable_commit"; then
  fail "target commit $target_commit is not reachable from $PROMOTABLE_REF"
fi

ssh_cmd=("$SSH_BIN")
if (( ${#SSH_EXTRA_ARGS[@]} > 0 )); then
  ssh_cmd+=("${SSH_EXTRA_ARGS[@]}")
fi
ssh_cmd+=("$HOST_ALIAS" bash -s --)

if ! remote_output="$({
  "${ssh_cmd[@]}" \
    "$target_commit" \
    "$REMOTE_DEPLOY_SCRIPT" \
    "$REMOTE_CHECK_SCRIPT" \
    "$REMOTE_REPO_ROOT" \
    "$REMOTE_MANIFEST_PATH" <<'REMOTE'
set -euo pipefail

target_commit="$1"
deploy_script="$2"
check_script="$3"
repo_root="$4"
manifest_path="$5"

bash "$deploy_script" "$target_commit"
bash "$check_script"

host_commit="$(git -C "$repo_root" rev-parse HEAD)"
if [[ "$host_commit" != "$target_commit" ]]; then
  echo "FAIL: host repo HEAD $host_commit does not match intended commit $target_commit" >&2
  exit 1
fi

echo "DEPLOYED_COMMIT=$host_commit"
echo "MANIFEST_PATH=$manifest_path"
REMOTE
} 2>&1)"; then
  printf '%s\n' "$remote_output" >&2
  exit 1
fi

printf '%s\n' "$remote_output"

deployed_commit="$(printf '%s\n' "$remote_output" | sed -n 's/^DEPLOYED_COMMIT=//p' | tail -n 1)"
manifest_path="$(printf '%s\n' "$remote_output" | sed -n 's/^MANIFEST_PATH=//p' | tail -n 1)"

if [[ "$deployed_commit" != "$target_commit" ]]; then
  fail "post-deploy commit mismatch: expected $target_commit got ${deployed_commit:-<missing>}"
fi

if [[ -z "$manifest_path" ]]; then
  fail "remote promotion did not report manifest path"
fi

echo "Promoted commit $deployed_commit to $HOST_ALIAS"
echo "Authority manifest: $manifest_path"
