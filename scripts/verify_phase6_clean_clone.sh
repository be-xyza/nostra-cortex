#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR_INPUT="${1:-$(pwd)}"
ROOT_DIR="$(cd "${ROOT_DIR_INPUT}" && pwd)"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

pass() {
  echo "PASS: $*"
}

git -C "${ROOT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1 || \
  fail "not a git worktree: ${ROOT_DIR}"

if [[ -n "$(git -C "${ROOT_DIR}" status --porcelain --untracked-files=all)" ]]; then
  git -C "${ROOT_DIR}" status --short
  fail "worktree is not clean"
fi
pass "worktree is clean"

if [[ -f "${ROOT_DIR}/.gitmodules" ]]; then
  submodule_status="$(git -C "${ROOT_DIR}" submodule status --recursive)"
  echo "${submodule_status}"
  if echo "${submodule_status}" | grep -qE '^[-+U]'; then
    fail "submodules are not initialized to the recorded revision"
  fi
  pass "submodules are pinned and initialized"
fi

for required_path in \
  "${ROOT_DIR}/docs/cortex/eudaemon-alpha-phase6-hetzner.md" \
  "${ROOT_DIR}/docs/cortex/eudaemon-alpha-phase6-checklist.md" \
  "${ROOT_DIR}/docs/cortex/eudaemon-alpha-ssh-config.example" \
  "${ROOT_DIR}/.github/ISSUE_TEMPLATE/eudaemon-alpha-phase6-bring-up.md" \
  "${ROOT_DIR}/scripts/run_cortex_runtime_freeze_gates.sh"
do
  [[ -f "${required_path}" ]] || fail "required Phase 6 asset missing: ${required_path}"
done
pass "Phase 6 operator assets are present"

bash "${ROOT_DIR}/scripts/run_cortex_runtime_freeze_gates.sh"

pass "Phase 6 clean clone verification completed"
