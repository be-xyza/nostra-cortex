#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNIT="${ROOT_DIR}/ops/hetzner/systemd/cortex-workrouter.service"
STUB="${ROOT_DIR}/scripts/work_router_service_stub.sh"
OBSERVE_SERVICE="${ROOT_DIR}/scripts/work_router_observe_service.py"
CONTRACT="${ROOT_DIR}/research/132-eudaemon-alpha-initiative/VPS_WORKROUTER_BOOTSTRAP_CONTRACT.md"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local pattern="$2"
  local label="$3"
  if ! grep -Fq "$pattern" "$path"; then
    fail "$label missing from $path: $pattern"
  fi
}

reject_contains() {
  local path="$1"
  local pattern="$2"
  local label="$3"
  if grep -Fiq "$pattern" "$path"; then
    fail "$label forbidden in $path: $pattern"
  fi
}

require_file "$UNIT"
require_file "$STUB"
require_file "$OBSERVE_SERVICE"
require_file "$CONTRACT"

require_contains "$UNIT" "Description=Cortex WorkRouter D0-D1 Bootstrap Service" "unit description"
require_contains "$UNIT" "WorkingDirectory=__DEPLOY_ROOT__/repo" "repo working directory"
require_contains "$UNIT" "Environment=WORK_ROUTER_MAX_DISPATCH_LEVEL=D1" "D1 ceiling"
require_contains "$UNIT" "Environment=WORK_ROUTER_SOURCE_MUTATION_ALLOWED=0" "source mutation disabled"
require_contains "$UNIT" "Environment=WORK_ROUTER_RUNTIME_MUTATION_ALLOWED=0" "runtime mutation disabled"
require_contains "$UNIT" "Environment=WORK_ROUTER_REQUIRE_REQUEST_ID=1" "request id requirement"
require_contains "$UNIT" "Environment=WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY=1" "unknown review-only mode"
require_contains "$UNIT" "Environment=WORK_ROUTER_TRANSPORTS_ENABLED=cli" "CLI-only bootstrap transport"
require_contains "$UNIT" "Environment=WORK_ROUTER_LIVE_TRANSPORT_ENABLED=0" "live transport disabled"
require_contains "$UNIT" "Environment=WORK_ROUTER_MODE=observe" "observe mode"
require_contains "$UNIT" "ExecStart=__DEPLOY_ROOT__/repo/scripts/work_router_service_stub.sh" "observe service wrapper exec"
require_contains "$UNIT" "StandardOutput=append:__DEPLOY_ROOT__/logs/cortex-workrouter.log" "local runtime log"

for forbidden in \
  "git commit" \
  "git push" \
  "gh pr" \
  "dfx canister call" \
  "dfx deploy" \
  "icp deploy" \
  "canister call" \
  "NOSTRA_PROVIDER" \
  "PROVIDER_API_KEY" \
  "DEPLOY_KEY" \
  "CANISTER"
do
  reject_contains "$UNIT" "$forbidden" "mutation or sensitive credential surface"
done

require_contains "$STUB" "WORK_ROUTER_MAX_DISPATCH_LEVEL" "wrapper ceiling check"
require_contains "$STUB" "WORK_ROUTER_SOURCE_MUTATION_ALLOWED" "wrapper source mutation check"
require_contains "$STUB" "WORK_ROUTER_RUNTIME_MUTATION_ALLOWED" "wrapper runtime mutation check"
require_contains "$STUB" "WORK_ROUTER_REQUIRE_REQUEST_ID" "wrapper request id check"
require_contains "$STUB" "WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY" "wrapper unknown review check"
require_contains "$STUB" "WORK_ROUTER_TRANSPORTS_ENABLED" "wrapper transport check"
require_contains "$STUB" "WORK_ROUTER_LIVE_TRANSPORT_ENABLED" "wrapper live transport check"
require_contains "$STUB" "work_router_observe_service.py" "wrapper observe service handoff"

if bash -n "$STUB"; then
  :
else
  fail "workrouter service stub shell syntax is invalid"
fi

python3 -m py_compile "$OBSERVE_SERVICE"

tmpdir="$(mktemp -d)"
WORK_ROUTER_MAX_DISPATCH_LEVEL=D1 \
WORK_ROUTER_SOURCE_MUTATION_ALLOWED=0 \
WORK_ROUTER_RUNTIME_MUTATION_ALLOWED=0 \
WORK_ROUTER_REQUIRE_REQUEST_ID=1 \
WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY=1 \
WORK_ROUTER_TRANSPORTS_ENABLED=cli \
WORK_ROUTER_LIVE_TRANSPORT_ENABLED=0 \
WORK_ROUTER_MODE=observe \
  python3 "$OBSERVE_SERVICE" \
    --once \
    --outbox "$tmpdir/outbox" \
    --heartbeat "$tmpdir/heartbeat.json" \
    --evidence "$tmpdir/evidence.json" \
    --observed-at 2026-05-01T02:00:00Z \
    >/tmp/workrouter_observe_service_once.json

require_file "$tmpdir/heartbeat.json"
require_file "$tmpdir/evidence.json"
require_contains "$tmpdir/heartbeat.json" '"mutationAllowed": false' "observe heartbeat mutation flag"
require_contains "$tmpdir/heartbeat.json" '"liveTransportEnabled": false' "observe heartbeat live transport flag"
require_contains "$tmpdir/evidence.json" '"agentId": "workrouter"' "observe evidence agent id"
require_contains "$tmpdir/evidence.json" '"sourceMutationAllowed": false' "observe evidence source mutation flag"
require_contains "$tmpdir/evidence.json" '"runtimeMutationAllowed": false' "observe evidence runtime mutation flag"

require_contains "$CONTRACT" "cortex-workrouter.service" "bootstrap contract service reference"
require_contains "$CONTRACT" "It does not require new infrastructure for v1." "bootstrap contract infrastructure stance"
require_contains "$CONTRACT" "WORK_ROUTER_MAX_DISPATCH_LEVEL=D1" "bootstrap contract D1 ceiling"
require_contains "$CONTRACT" "WORK_ROUTER_SOURCE_MUTATION_ALLOWED=0" "bootstrap contract source mutation disabled"
require_contains "$CONTRACT" "WORK_ROUTER_RUNTIME_MUTATION_ALLOWED=0" "bootstrap contract runtime mutation disabled"
require_contains "$CONTRACT" "WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY=1" "bootstrap contract unknown review mode"

echo "PASS: VPS WorkRouter bootstrap contract checks"
