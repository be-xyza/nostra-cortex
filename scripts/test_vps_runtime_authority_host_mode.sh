#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CHECK_SCRIPT="$ROOT_DIR/scripts/check_vps_runtime_authority.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

make_fixture() {
  local tmpdir="$1"
  local worker_exec="$2"
  local commit

  mkdir -p \
    "$tmpdir/repo/cortex/target/release" \
    "$tmpdir/repo/nostra/worker/target/release" \
    "$tmpdir/repo/cortex/apps/cortex-web" \
    "$tmpdir/state" \
    "$tmpdir/systemd"

  git -C "$tmpdir/repo" init -q
  git -C "$tmpdir/repo" config user.email test@example.com
  git -C "$tmpdir/repo" config user.name test
  printf 'authority fixture\n' >"$tmpdir/repo/README.md"
  git -C "$tmpdir/repo" add README.md
  git -C "$tmpdir/repo" commit -q -m 'fixture'
  commit="$(git -C "$tmpdir/repo" rev-parse HEAD)"

  cat >"$tmpdir/state/cortex_runtime_authority.json" <<EOF
{
  "schemaVersion": "1.0.0",
  "generatedAt": "2026-03-26T00:00:00Z",
  "deployRoot": "$tmpdir",
  "repoRoot": "$tmpdir/repo",
  "git": {
    "branch": "master",
    "commit": "$commit",
    "origin": "git@example.com:test/repo.git"
  },
  "runtime": {
    "gateway": {
      "execPath": "$tmpdir/repo/cortex/target/release/cortex-gateway",
      "workingDirectory": "$tmpdir/repo/cortex"
    },
    "worker": {
      "execPath": "$tmpdir/repo/nostra/worker/target/release/cortex_worker",
      "workingDirectory": "$tmpdir/repo/nostra/worker"
    },
    "cortexWeb": {
      "deploymentMode": "not_deployed",
      "sourceRoot": "$tmpdir/repo/cortex/apps/cortex-web"
    }
  },
  "authorityDocs": {
    "primaryRunbook": "$ROOT_DIR/docs/cortex/eudaemon-alpha-phase6-hetzner.md",
    "operationsIndex": "$ROOT_DIR/docs/cortex/README.md"
  }
}
EOF

  cat >"$tmpdir/systemd/cortex-gateway.service" <<EOF
[Service]
WorkingDirectory=$tmpdir/repo/cortex
ExecStart=$tmpdir/repo/cortex/target/release/cortex-gateway
EOF

  cat >"$tmpdir/systemd/cortex-worker.service" <<EOF
[Service]
WorkingDirectory=$tmpdir/repo/nostra/worker
ExecStart=$worker_exec
EOF
}

pass_fixture="$(mktemp -d /tmp/cortex-vps-auth-pass.XXXXXX)"
make_fixture "$pass_fixture" "$pass_fixture/repo/nostra/worker/target/release/cortex_worker"

NOSTRA_VPS_DEPLOY_ROOT="$pass_fixture" \
NOSTRA_VPS_REPO_ROOT="$pass_fixture/repo" \
NOSTRA_VPS_STATE_ROOT="$pass_fixture/state" \
NOSTRA_VPS_AUTHORITY_MANIFEST="$pass_fixture/state/cortex_runtime_authority.json" \
NOSTRA_VPS_SYSTEMD_ROOT="$pass_fixture/systemd" \
NOSTRA_VPS_SKIP_PROCESS_PROVENANCE=1 \
bash "$CHECK_SCRIPT" >/dev/null

fail_fixture="$(mktemp -d /tmp/cortex-vps-auth-fail.XXXXXX)"
make_fixture "$fail_fixture" "/usr/local/bin/cortex_worker"

set +e
output="$(
  NOSTRA_VPS_DEPLOY_ROOT="$fail_fixture" \
  NOSTRA_VPS_REPO_ROOT="$fail_fixture/repo" \
  NOSTRA_VPS_STATE_ROOT="$fail_fixture/state" \
  NOSTRA_VPS_AUTHORITY_MANIFEST="$fail_fixture/state/cortex_runtime_authority.json" \
  NOSTRA_VPS_SYSTEMD_ROOT="$fail_fixture/systemd" \
  NOSTRA_VPS_SKIP_PROCESS_PROVENANCE=1 \
  bash "$CHECK_SCRIPT" 2>&1
)"
rc=$?
set -e

if [[ "$rc" -eq 0 ]]; then
  fail "host-mode contract unexpectedly passed detached worker fixture"
fi

if ! grep -Fq "/usr/local/bin/cortex_worker" <<<"$output"; then
  fail "host-mode contract did not report detached worker path"
fi

echo "PASS: vps runtime authority host-mode fixture coverage"
