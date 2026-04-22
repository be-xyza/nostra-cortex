#!/usr/bin/env bash
set -euo pipefail

GATEWAY_BASE="${1:-http://127.0.0.1:3000}"
SPACE_ID="${2:-nostra-governance-v0}"
ACTOR_ID="${ACTOR_ID:-smoke:contributions-cockpit}"
ACTOR_ROLE="${ACTOR_ROLE:-operator}"

WORKBENCH_URL="${GATEWAY_BASE}/api/system/ux/workbench?route=%2Fcontributions&space_id=${SPACE_ID}"
AGENT_RUNS_URL="${GATEWAY_BASE}/api/system/agents/runs?space_id=${SPACE_ID}"
GRAPH_RUNS_URL="${GATEWAY_BASE}/api/kg/spaces/${SPACE_ID}/contribution-graph/runs?limit=10"

BUILD_JSON="$(curl -sS "${GATEWAY_BASE}/api/system/build")"
READY_JSON="$(curl -sS "${GATEWAY_BASE}/api/system/ready")"
STATUS_JSON="$(curl -sS "${GATEWAY_BASE}/api/system/status")"
WORKBENCH_JSON="$(curl -sS -H "x-cortex-actor: ${ACTOR_ID}" -H "x-cortex-role: ${ACTOR_ROLE}" "${WORKBENCH_URL}")"
AGENT_RUNS_JSON="$(curl -sS -H "x-cortex-actor: ${ACTOR_ID}" -H "x-cortex-role: ${ACTOR_ROLE}" "${AGENT_RUNS_URL}")"
GRAPH_RUNS_JSON="$(curl -sS "${GRAPH_RUNS_URL}")"

TMP_BUILD="$(mktemp)"
TMP_READY="$(mktemp)"
TMP_STATUS="$(mktemp)"
TMP_WORKBENCH="$(mktemp)"
TMP_AGENT_RUNS="$(mktemp)"
TMP_GRAPH_RUNS="$(mktemp)"

cleanup() {
  rm -f "$TMP_BUILD" "$TMP_READY" "$TMP_STATUS" "$TMP_WORKBENCH" "$TMP_AGENT_RUNS" "$TMP_GRAPH_RUNS"
}
trap cleanup EXIT

printf '%s' "$BUILD_JSON" > "$TMP_BUILD"
printf '%s' "$READY_JSON" > "$TMP_READY"
printf '%s' "$STATUS_JSON" > "$TMP_STATUS"
printf '%s' "$WORKBENCH_JSON" > "$TMP_WORKBENCH"
printf '%s' "$AGENT_RUNS_JSON" > "$TMP_AGENT_RUNS"
printf '%s' "$GRAPH_RUNS_JSON" > "$TMP_GRAPH_RUNS"

python3 - <<'EOF' "$TMP_BUILD" "$TMP_READY" "$TMP_STATUS" "$TMP_WORKBENCH" "$TMP_AGENT_RUNS" "$TMP_GRAPH_RUNS" "$SPACE_ID"
import json
import sys

build_path, ready_path, status_path, workbench_path, agent_runs_path, graph_runs_path, space_id = sys.argv[1:8]

with open(build_path, "r", encoding="utf-8") as handle:
    build = json.load(handle)
with open(ready_path, "r", encoding="utf-8") as handle:
    ready = json.load(handle)
with open(status_path, "r", encoding="utf-8") as handle:
    status = json.load(handle)
with open(workbench_path, "r", encoding="utf-8") as handle:
    workbench = json.load(handle)
with open(agent_runs_path, "r", encoding="utf-8") as handle:
    agent_runs = json.load(handle)
with open(graph_runs_path, "r", encoding="utf-8") as handle:
    graph_runs = json.load(handle)

def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    sys.exit(1)

def require_key(obj, key: str, label: str) -> None:
    if key not in obj:
        fail(f'{label} missing key "{key}"')

for key in ("buildId", "buildTimeUtc", "gatewayDispatchMode"):
    require_key(build, key, "build")

for key in ("ready", "gateway_port", "icp_network_healthy"):
    require_key(ready, key, "ready")
require_key(status, "icp_cli_running", "status")

icp_network_healthy = ready["icp_network_healthy"]
icp_cli_running = status["icp_cli_running"]

if not isinstance(workbench, dict):
    fail("workbench response is not a JSON object")
surface_id = workbench.get("surfaceId", "")
if not isinstance(surface_id, str) or "contributions" not in surface_id:
    fail("workbench surface is not the contributions cockpit")

workbench_text = json.dumps(workbench).lower()
if "under construction" in workbench_text:
    fail("workbench surface still contains placeholder text")
if "contributions cockpit" not in workbench_text:
    fail("workbench surface does not expose the contributions cockpit heading")

agent_runs_summary = None
if isinstance(agent_runs, list):
    agent_runs_summary = str(len(agent_runs))
elif (
    isinstance(agent_runs, dict)
    and agent_runs.get("errorCode") == "ACTOR_IDENTITY_UNVERIFIED"
):
    agent_runs_summary = "auth_locked"
else:
    fail("agent runs payload is neither an array nor an auth-locked gateway error")

if not isinstance(graph_runs, list):
    fail("graph runs payload is not an array")

print("PASS: contributions cockpit smoke")
print(
    f"  space_id={space_id} ready={ready['ready']} icp_network_healthy={icp_network_healthy} icp_cli_running={icp_cli_running}"
)
print(
    f"  surface_id={surface_id} agent_runs={agent_runs_summary} graph_runs={len(graph_runs)}"
)
print(f"  build={build['buildId']} dispatch={build['gatewayDispatchMode']}")
EOF
