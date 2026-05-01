#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_ROUTER_LOG_ROOT="${WORK_ROUTER_LOG_ROOT:-$ROOT_DIR/logs/work_router}"
INTAKE_PATH=""
DECISION_PATH=""
TRANSPORT="cli"
CHANNEL_REF="local-operator"
RUN_ID=""

usage() {
  cat <<'USAGE'
Usage: scripts/work_router_run_d1.sh --intake <NovelTaskIntakeV1.json> [options]

Options:
  --decision <DispatchDecisionV1.json>   Apply a decision and generate approved artifacts.
  --transport <kind>                     Dispatch transport kind (default: cli).
  --channel-ref <ref>                    Dispatch channel reference (default: local-operator).
  --run-id <id>                          Stable run id. Defaults to UTC timestamp + task slug.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --intake)
      INTAKE_PATH="$2"
      shift 2
      ;;
    --decision)
      DECISION_PATH="$2"
      shift 2
      ;;
    --transport)
      TRANSPORT="$2"
      shift 2
      ;;
    --channel-ref)
      CHANNEL_REF="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
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

if [[ -z "$INTAKE_PATH" ]]; then
  echo "error: --intake is required" >&2
  usage >&2
  exit 1
fi

INTAKE_ABS="$INTAKE_PATH"
if [[ "$INTAKE_ABS" != /* ]]; then
  INTAKE_ABS="$ROOT_DIR/$INTAKE_ABS"
fi

if [[ ! -f "$INTAKE_ABS" ]]; then
  echo "error: intake not found: $INTAKE_ABS" >&2
  exit 1
fi

TASK_ID="$(python3 - "$INTAKE_ABS" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["taskId"])
PY
)"

if [[ -z "$RUN_ID" ]]; then
  RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-${TASK_ID//[^A-Za-z0-9._:-]/-}"
fi

RUN_DIR="$WORK_ROUTER_LOG_ROOT/runs/$RUN_ID"
mkdir -p "$RUN_DIR"

STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
ROUTER_BUNDLE="$RUN_DIR/router_bundle.json"
MESSAGE="$RUN_DIR/message.txt"
RECEIPT="$RUN_DIR/receipt.json"
APPROVED_BUNDLE="$RUN_DIR/approved_bundle.json"
HANDOFF="$RUN_DIR/handoff.md"
RUN_JSON="$RUN_DIR/run.json"

python3 "$ROOT_DIR/scripts/work_router_dispatch_dry_run.py" \
  "$INTAKE_ABS" \
  --transport "$TRANSPORT" \
  --channel-ref "$CHANNEL_REF" \
  --created-at "$STARTED_AT" \
  > "$ROUTER_BUNDLE"

python3 "$ROOT_DIR/scripts/work_router_render_dispatch_message.py" "$ROUTER_BUNDLE" > "$MESSAGE"
python3 "$ROOT_DIR/scripts/work_router_record_transport_receipt.py" \
  "$ROUTER_BUNDLE" \
  --message-ref "${TRANSPORT}:${CHANNEL_REF}:${RUN_ID}" \
  --status sent \
  --recorded-at "$STARTED_AT" \
  > "$RECEIPT"

STATUS="pending_decision"
DECISION_VALUE="none"

if [[ -n "$DECISION_PATH" ]]; then
  DECISION_ABS="$DECISION_PATH"
  if [[ "$DECISION_ABS" != /* ]]; then
    DECISION_ABS="$ROOT_DIR/$DECISION_ABS"
  fi
  python3 "$ROOT_DIR/scripts/work_router_apply_dispatch_decision.py" \
    "$ROUTER_BUNDLE" \
    "$DECISION_ABS" \
    --created-at "$STARTED_AT" \
    > "$APPROVED_BUNDLE"
  DECISION_VALUE="$(python3 - "$DECISION_ABS" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["decision"])
PY
)"
  STATUS="decision_applied"
  if grep -q '"codeChangeRequest"' "$APPROVED_BUNDLE"; then
    python3 "$ROOT_DIR/scripts/work_router_generate_handoff.py" "$APPROVED_BUNDLE" --output "$HANDOFF"
    STATUS="handoff_generated"
  fi
fi

FINISHED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

python3 - "$RUN_JSON" "$RUN_ID" "$STATUS" "$STARTED_AT" "$FINISHED_AT" "$INTAKE_ABS" "${DECISION_PATH:-}" "$ROUTER_BUNDLE" "$MESSAGE" "$RECEIPT" "$APPROVED_BUNDLE" "$HANDOFF" "$TRANSPORT" "$DECISION_VALUE" <<'PY'
import json
import pathlib
import sys

(
    run_json,
    run_id,
    status,
    started_at,
    finished_at,
    intake,
    decision,
    router_bundle,
    message,
    receipt,
    approved_bundle,
    handoff,
    transport,
    decision_value,
) = sys.argv[1:]

router_payload = json.loads(pathlib.Path(router_bundle).read_text(encoding="utf-8"))
router = router_payload["workRouterDecision"]
request = router_payload["dispatchRequest"]

artifact_refs = {
    "routerBundle": router_bundle,
    "message": message,
    "receipt": receipt,
}
if pathlib.Path(approved_bundle).exists():
    artifact_refs["approvedBundle"] = approved_bundle
if pathlib.Path(handoff).exists():
    artifact_refs["handoff"] = handoff

input_refs = {"intake": intake}
if decision:
    input_refs["decision"] = decision

payload = {
    "schemaVersion": "1.0.0",
    "runId": run_id,
    "status": status,
    "startedAt": started_at,
    "finishedAt": finished_at,
    "inputRefs": input_refs,
    "artifactRefs": artifact_refs,
    "authority": {
        "maxLevel": request["authorityCeiling"],
        "mutationAllowed": False,
        "transportKind": transport,
    },
    "summary": {
        "taskRef": request["taskRef"],
        "route": router["recommendedRoute"],
        "riskLevel": request["riskLevel"],
        "decision": decision_value,
        "notes": "Local WorkRouter D1 run; no repo or runtime mutation authorized.",
    },
    "warnings": [],
}

pathlib.Path(run_json).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

python3 "$ROOT_DIR/scripts/validate_work_router_run.py" "$RUN_JSON" >/dev/null
mkdir -p "$WORK_ROUTER_LOG_ROOT"
cp "$RUN_JSON" "$WORK_ROUTER_LOG_ROOT/latest.json"

echo "$RUN_JSON"
