#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_ROUTER_LOG_ROOT="${WORK_ROUTER_LOG_ROOT:-$ROOT_DIR/logs/work_router}"
RUN_ID=""
DECISION_PATH=""

usage() {
  cat <<'USAGE'
Usage: scripts/work_router_apply_run_decision.sh --run-id <id> --decision <DispatchDecisionV1.json>

Applies a structured dispatch decision to an existing pending WorkRouter run.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --decision)
      DECISION_PATH="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$RUN_ID" || -z "$DECISION_PATH" ]]; then
  echo "error: --run-id and --decision are required" >&2
  usage >&2
  exit 1
fi

RUN_DIR="$WORK_ROUTER_LOG_ROOT/runs/$RUN_ID"
RUN_JSON="$RUN_DIR/run.json"
ROUTER_BUNDLE="$RUN_DIR/router_bundle.json"
APPROVED_BUNDLE="$RUN_DIR/approved_bundle.json"
HANDOFF="$RUN_DIR/handoff.md"

if [[ ! -f "$RUN_JSON" ]]; then
  echo "error: run not found: $RUN_JSON" >&2
  exit 1
fi

DECISION_ABS="$DECISION_PATH"
if [[ "$DECISION_ABS" != /* ]]; then
  DECISION_ABS="$ROOT_DIR/$DECISION_ABS"
fi

STATUS="$(python3 - "$RUN_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["status"])
PY
)"

if [[ "$STATUS" != "pending_decision" ]]; then
  echo "error: run is not pending_decision: $STATUS" >&2
  exit 1
fi

CREATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 "$ROOT_DIR/scripts/work_router_apply_dispatch_decision.py" \
  "$ROUTER_BUNDLE" \
  "$DECISION_ABS" \
  --created-at "$CREATED_AT" \
  > "$APPROVED_BUNDLE"

DECISION_VALUE="$(python3 - "$DECISION_ABS" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["decision"])
PY
)"

NEW_STATUS="decision_applied"
if grep -q '"codeChangeRequest"' "$APPROVED_BUNDLE"; then
  python3 "$ROOT_DIR/scripts/work_router_generate_handoff.py" "$APPROVED_BUNDLE" --output "$HANDOFF"
  NEW_STATUS="handoff_generated"
fi

FINISHED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 - "$RUN_JSON" "$DECISION_ABS" "$APPROVED_BUNDLE" "$HANDOFF" "$NEW_STATUS" "$DECISION_VALUE" "$FINISHED_AT" <<'PY'
import json
import pathlib
import sys

run_json, decision, approved_bundle, handoff, status, decision_value, finished_at = sys.argv[1:]
path = pathlib.Path(run_json)
payload = json.loads(path.read_text(encoding="utf-8"))
payload["status"] = status
payload["finishedAt"] = finished_at
payload["inputRefs"]["decision"] = decision
payload["artifactRefs"]["approvedBundle"] = approved_bundle
if pathlib.Path(handoff).exists():
    payload["artifactRefs"]["handoff"] = handoff
payload["summary"]["decision"] = decision_value
path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

python3 "$ROOT_DIR/scripts/validate_work_router_run.py" "$RUN_JSON" >/dev/null
mkdir -p "$WORK_ROUTER_LOG_ROOT"
cp "$RUN_JSON" "$WORK_ROUTER_LOG_ROOT/latest.json"
echo "$RUN_JSON"
