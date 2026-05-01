#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_ROUTER_LOG_ROOT="${WORK_ROUTER_LOG_ROOT:-$ROOT_DIR/logs/work_router}"
VALIDATOR="$ROOT_DIR/scripts/validate_work_router_dispatch.py"
DRY_RUN="$ROOT_DIR/scripts/work_router_dispatch_dry_run.py"
APPLY_DECISION="$ROOT_DIR/scripts/work_router_apply_dispatch_decision.py"
GENERATE_HANDOFF="$ROOT_DIR/scripts/work_router_generate_handoff.py"
RENDER_MESSAGE="$ROOT_DIR/scripts/work_router_render_dispatch_message.py"
RECORD_RECEIPT="$ROOT_DIR/scripts/work_router_record_transport_receipt.py"
RUN_D1="$ROOT_DIR/scripts/work_router_run_d1.sh"
VALIDATE_RUN="$ROOT_DIR/scripts/validate_work_router_run.py"
LIST_PENDING="$ROOT_DIR/scripts/work_router_list_pending.py"
APPLY_RUN_DECISION="$ROOT_DIR/scripts/work_router_apply_run_decision.sh"
EXPORT_PENDING="$ROOT_DIR/scripts/work_router_export_pending_messages.py"
CLI_ADAPTER="$ROOT_DIR/scripts/work_router_cli_transport_adapter.py"
PARSE_REPLY="$ROOT_DIR/scripts/work_router_parse_transport_reply.py"
TELEGRAM_ADAPTER="$ROOT_DIR/scripts/work_router_telegram_transport_adapter.py"
COMMAND="$ROOT_DIR/scripts/work_router_command.py"
REVIEW_UNKNOWNS="$ROOT_DIR/scripts/work_router_review_unknowns.py"
APPLY_UNKNOWN_REVIEW="$ROOT_DIR/scripts/work_router_apply_unknown_review.py"
TELEGRAM_RECEIVE="$ROOT_DIR/scripts/work_router_telegram_receive_dry_run.py"
HERMES_GATEWAY_RECEIVE="$ROOT_DIR/scripts/work_router_hermes_gateway_receive_dry_run.py"
AGENT_OPERATING_MODEL_CHECK="$ROOT_DIR/scripts/check_agent_operating_model.sh"
TELEGRAM_UPDATE_APPROVE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/telegram_update_approve_reply.v1.json"
TELEGRAM_UPDATE_UNKNOWN="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/telegram_update_unknown_reply.v1.json"
HERMES_GATEWAY_APPROVE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/hermes_gateway_message_approve_reply.v1.json"
HERMES_GATEWAY_UNKNOWN="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/hermes_gateway_message_unknown_reply.v1.json"
ESCALATION_FIXTURE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/work_router_dispatch_level_escalation_rejected.v1.json"
STRUCTURAL_FIXTURE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/work_router_dispatch_structural_code_change_rejected.v1.json"
INTAKE_FIXTURE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/novel_task_intake_workflow_failure_capability_review.v1.json"
PATCH_PREP_INTAKE_FIXTURE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/novel_task_intake_hermescortexdev_low_risk_patch_prep.v1.json"
PATCH_PREP_DECISION="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/dispatch_decision_low_risk_patch_prep_approve.v1.json"
INVALID_DECISION="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/dispatch_decision_level_escalation_rejected.v1.json"
DRY_RUN_OUT="/tmp/work_router_dispatch_dry_run.out.json"
PATCH_PREP_DRY_RUN_OUT="/tmp/work_router_dispatch_patch_prep_dry_run.out.json"
PATCH_PREP_APPLIED_OUT="/tmp/work_router_dispatch_patch_prep_applied.out.json"
PATCH_PREP_HANDOFF_OUT="/tmp/work_router_dispatch_patch_prep.handoff.md"
PATCH_PREP_MESSAGE_OUT="/tmp/work_router_dispatch_patch_prep.message.txt"
PATCH_PREP_RECEIPT_OUT="/tmp/work_router_dispatch_patch_prep.receipt.json"
RUN_JSON_OUT="/tmp/work_router_run_path.txt"

python3 "$VALIDATOR"
"$AGENT_OPERATING_MODEL_CHECK" >/tmp/agent_operating_model_check.out
python3 "$DRY_RUN" "$INTAKE_FIXTURE" --transport telegram --channel-ref operator-private --created-at 2026-04-30T12:30:00Z > "$DRY_RUN_OUT"
python3 "$VALIDATOR" "$DRY_RUN_OUT" >/tmp/work_router_dispatch_dry_run_validate.out
python3 "$DRY_RUN" "$PATCH_PREP_INTAKE_FIXTURE" --transport cli --channel-ref local-operator --created-at 2026-04-30T12:35:00Z > "$PATCH_PREP_DRY_RUN_OUT"
python3 "$VALIDATOR" "$PATCH_PREP_DRY_RUN_OUT" >/tmp/work_router_dispatch_patch_prep_dry_run_validate.out
python3 "$RENDER_MESSAGE" "$PATCH_PREP_DRY_RUN_OUT" > "$PATCH_PREP_MESSAGE_OUT"
python3 "$RECORD_RECEIPT" "$PATCH_PREP_DRY_RUN_OUT" --message-ref "cli:local-operator:dry-run-001" --status sent --recorded-at 2026-04-30T12:35:10Z > "$PATCH_PREP_RECEIPT_OUT"
python3 "$VALIDATOR" "$PATCH_PREP_RECEIPT_OUT" >/tmp/work_router_dispatch_receipt_validate.out
python3 "$APPLY_DECISION" "$PATCH_PREP_DRY_RUN_OUT" "$PATCH_PREP_DECISION" --created-at 2026-04-30T12:37:00Z > "$PATCH_PREP_APPLIED_OUT"
python3 "$VALIDATOR" "$PATCH_PREP_APPLIED_OUT" >/tmp/work_router_dispatch_patch_prep_applied_validate.out
python3 "$GENERATE_HANDOFF" "$PATCH_PREP_APPLIED_OUT" --output "$PATCH_PREP_HANDOFF_OUT"
"$RUN_D1" \
  --intake "$PATCH_PREP_INTAKE_FIXTURE" \
  --decision "$PATCH_PREP_DECISION" \
  --transport cli \
  --channel-ref local-operator \
  --run-id "check-work-router-d1" \
  > "$RUN_JSON_OUT"
python3 "$VALIDATE_RUN" "$(cat "$RUN_JSON_OUT")" >/tmp/work_router_run_validate.out
"$RUN_D1" \
  --intake "$PATCH_PREP_INTAKE_FIXTURE" \
  --transport cli \
  --channel-ref local-operator \
  --run-id "check-work-router-pending" \
  >/tmp/work_router_pending_path.txt
python3 "$LIST_PENDING" >/tmp/work_router_pending_list.json
python3 "$EXPORT_PENDING" --outbox /tmp/work_router_outbox --created-at 2026-04-30T12:38:00Z >/tmp/work_router_outbox_export.json
python3 "$PARSE_REPLY" \
  /tmp/work_router_outbox/check-work-router-pending.json \
  --reply approve \
  --decider-kind operator \
  --decider-id user \
  --decided-at 2026-04-30T12:39:00Z \
  --output /tmp/work_router_parsed_decision.json
python3 "$CLI_ADAPTER" --outbox /tmp/work_router_outbox --run-id check-work-router-pending --send --decision /tmp/work_router_parsed_decision.json >/tmp/work_router_cli_adapter.out
"$RUN_D1" \
  --intake "$PATCH_PREP_INTAKE_FIXTURE" \
  --transport telegram \
  --channel-ref operator-private \
  --run-id "check-work-router-telegram-dry-run" \
  >/tmp/work_router_telegram_path.txt
python3 "$EXPORT_PENDING" --outbox /tmp/work_router_telegram_outbox --created-at 2026-04-30T12:42:00Z >/tmp/work_router_telegram_outbox_export.json
python3 "$PARSE_REPLY" \
  /tmp/work_router_telegram_outbox/check-work-router-telegram-dry-run.json \
  --reply approve \
  --decider-kind operator \
  --decider-id user \
  --decided-at 2026-04-30T12:43:00Z \
  --output /tmp/work_router_telegram_decision.json
python3 "$TELEGRAM_ADAPTER" \
  --outbox /tmp/work_router_telegram_outbox \
  --run-id check-work-router-telegram-dry-run \
  --dry-run \
  --decision /tmp/work_router_telegram_decision.json \
  >/tmp/work_router_telegram_adapter.out
"$RUN_D1" \
  --intake "$PATCH_PREP_INTAKE_FIXTURE" \
  --transport telegram \
  --channel-ref 424242 \
  --run-id "check-work-router-telegram-receive" \
  >/tmp/work_router_telegram_receive_path.txt
rm -rf /tmp/work_router_telegram_receive_outbox
python3 "$EXPORT_PENDING" --outbox /tmp/work_router_telegram_receive_outbox --run-id check-work-router-telegram-receive --created-at 2026-04-30T12:46:00Z >/tmp/work_router_telegram_receive_export.json
rm -f "$WORK_ROUTER_LOG_ROOT/telegram_processed/1320001.json" "$WORK_ROUTER_LOG_ROOT/telegram_processed/1320002.json"
python3 "$TELEGRAM_RECEIVE" "$TELEGRAM_UPDATE_APPROVE" --outbox /tmp/work_router_telegram_receive_outbox --created-at 2026-04-30T12:47:00Z >/tmp/work_router_telegram_receive_approve.json
python3 "$VALIDATE_RUN" "$WORK_ROUTER_LOG_ROOT/runs/check-work-router-telegram-receive/run.json" >/tmp/work_router_telegram_receive_validate.out
python3 "$TELEGRAM_RECEIVE" "$TELEGRAM_UPDATE_APPROVE" --outbox /tmp/work_router_telegram_receive_outbox --created-at 2026-04-30T12:47:00Z >/tmp/work_router_telegram_receive_duplicate.json
python3 "$TELEGRAM_RECEIVE" "$TELEGRAM_UPDATE_UNKNOWN" --outbox /tmp/work_router_telegram_receive_outbox --created-at 2026-04-30T12:48:00Z >/tmp/work_router_telegram_receive_unknown.json
"$RUN_D1" \
  --intake "$PATCH_PREP_INTAKE_FIXTURE" \
  --transport hermes_gateway \
  --channel-ref operator-private \
  --run-id "check-work-router-hermes-gateway-receive" \
  >/tmp/work_router_hermes_gateway_receive_path.txt
rm -rf /tmp/work_router_hermes_gateway_outbox
python3 "$EXPORT_PENDING" --outbox /tmp/work_router_hermes_gateway_outbox --run-id check-work-router-hermes-gateway-receive --created-at 2026-05-01T01:00:00Z >/tmp/work_router_hermes_gateway_export.json
rm -f "$WORK_ROUTER_LOG_ROOT/hermes_gateway_processed/hermes-gateway-msg-approve-001.json" "$WORK_ROUTER_LOG_ROOT/hermes_gateway_processed/hermes-gateway-msg-unknown-001.json"
python3 "$HERMES_GATEWAY_RECEIVE" "$HERMES_GATEWAY_APPROVE" --outbox /tmp/work_router_hermes_gateway_outbox --created-at 2026-05-01T01:00:10Z >/tmp/work_router_hermes_gateway_approve.json
python3 "$VALIDATE_RUN" "$WORK_ROUTER_LOG_ROOT/runs/check-work-router-hermes-gateway-receive/run.json" >/tmp/work_router_hermes_gateway_receive_validate.out
python3 "$HERMES_GATEWAY_RECEIVE" "$HERMES_GATEWAY_APPROVE" --outbox /tmp/work_router_hermes_gateway_outbox --created-at 2026-05-01T01:00:10Z >/tmp/work_router_hermes_gateway_duplicate.json
python3 "$HERMES_GATEWAY_RECEIVE" "$HERMES_GATEWAY_UNKNOWN" --outbox /tmp/work_router_hermes_gateway_outbox --created-at 2026-05-01T01:01:00Z >/tmp/work_router_hermes_gateway_unknown.json
python3 "$COMMAND" --text pending --created-at 2026-04-30T12:44:00Z --json >/tmp/work_router_command_pending.json
python3 "$COMMAND" --text approved --created-at 2026-04-30T12:44:01Z --json >/tmp/work_router_command_approved.json
python3 "$COMMAND" --text "apprve please" --created-at 2026-04-30T12:44:02Z --json >/tmp/work_router_command_unknown.json
python3 "$REVIEW_UNKNOWNS" --unknown "$WORK_ROUTER_LOG_ROOT/unknown/20260430T124402Z-apprve-please.json" --out-dir /tmp/work_router_unknown_reviews --created-at 2026-04-30T12:45:00Z >/tmp/work_router_unknown_review_index.json
python3 "$APPLY_UNKNOWN_REVIEW" /tmp/work_router_unknown_reviews/20260430T124402Z-apprve-please.review.json >/tmp/work_router_unknown_review_apply.json
if ! grep -q '"runId": "check-work-router-pending"' /tmp/work_router_pending_list.json; then
  echo "FAIL: pending run was not listed" >&2
  cat /tmp/work_router_pending_list.json >&2 || true
  exit 1
fi
if ! grep -q "check-work-router-pending.json" /tmp/work_router_outbox_export.json; then
  echo "FAIL: pending run was not exported to transport outbox" >&2
  cat /tmp/work_router_outbox_export.json >&2 || true
  exit 1
fi
if ! grep -q '"appliedRun"' /tmp/work_router_cli_adapter.out; then
  echo "FAIL: CLI adapter did not apply decision" >&2
  cat /tmp/work_router_cli_adapter.out >&2 || true
  exit 1
fi
if ! grep -q '"kind": "decision_alias"' /tmp/work_router_command_approved.json; then
  echo "FAIL: approved alias was not parsed as decision_alias" >&2
  cat /tmp/work_router_command_approved.json >&2 || true
  exit 1
fi
if ! grep -q '"kind": "unknown"' /tmp/work_router_command_unknown.json; then
  echo "FAIL: unknown command was not captured as unknown" >&2
  cat /tmp/work_router_command_unknown.json >&2 || true
  exit 1
fi
if ! grep -q '"unknownRef"' /tmp/work_router_command_unknown.json; then
  echo "FAIL: unknown command did not record unknownRef" >&2
  cat /tmp/work_router_command_unknown.json >&2 || true
  exit 1
fi
if ! grep -q '"canonical": "approve"' /tmp/work_router_unknown_review_apply.json; then
  echo "FAIL: unknown route review did not propose approve mapping" >&2
  cat /tmp/work_router_unknown_review_apply.json >&2 || true
  exit 1
fi
python3 "$VALIDATE_RUN" "$WORK_ROUTER_LOG_ROOT/runs/check-work-router-pending/run.json" >/tmp/work_router_pending_applied_validate.out
python3 "$VALIDATE_RUN" "$WORK_ROUTER_LOG_ROOT/runs/check-work-router-telegram-dry-run/run.json" >/tmp/work_router_telegram_dry_run_validate.out
if ! grep -q '"status": "decision_applied"' /tmp/work_router_telegram_receive_approve.json; then
  echo "FAIL: telegram receive fixture did not apply approval decision" >&2
  cat /tmp/work_router_telegram_receive_approve.json >&2 || true
  exit 1
fi
if ! grep -q '"status": "skipped_duplicate"' /tmp/work_router_telegram_receive_duplicate.json; then
  echo "FAIL: telegram duplicate update was not skipped" >&2
  cat /tmp/work_router_telegram_receive_duplicate.json >&2 || true
  exit 1
fi
if ! grep -q '"status": "unknown_recorded"' /tmp/work_router_telegram_receive_unknown.json; then
  echo "FAIL: telegram unknown update was not recorded" >&2
  cat /tmp/work_router_telegram_receive_unknown.json >&2 || true
  exit 1
fi
if ! grep -q '"status": "decision_applied"' /tmp/work_router_hermes_gateway_approve.json; then
  echo "FAIL: Hermes Gateway fixture did not apply approval decision" >&2
  cat /tmp/work_router_hermes_gateway_approve.json >&2 || true
  exit 1
fi
if ! grep -q '"status": "skipped_duplicate"' /tmp/work_router_hermes_gateway_duplicate.json; then
  echo "FAIL: Hermes Gateway duplicate message was not skipped" >&2
  cat /tmp/work_router_hermes_gateway_duplicate.json >&2 || true
  exit 1
fi
if ! grep -q '"status": "unknown_recorded"' /tmp/work_router_hermes_gateway_unknown.json; then
  echo "FAIL: Hermes Gateway unknown message was not recorded" >&2
  cat /tmp/work_router_hermes_gateway_unknown.json >&2 || true
  exit 1
fi

if grep -q '"codeChangeRequestId"' "$PATCH_PREP_DRY_RUN_OUT"; then
  echo "FAIL: patch-prep dry run emitted CodeChangeRequestV1 before approval" >&2
  cat "$PATCH_PREP_DRY_RUN_OUT" >&2 || true
  exit 1
fi

if ! grep -q "Reply with one decision" "$PATCH_PREP_MESSAGE_OUT"; then
  echo "FAIL: rendered dispatch message missing reply instruction" >&2
  cat "$PATCH_PREP_MESSAGE_OUT" >&2 || true
  exit 1
fi

if ! grep -q '"codeChangeRequestId"' "$PATCH_PREP_APPLIED_OUT"; then
  echo "FAIL: approved patch-prep decision did not emit CodeChangeRequestV1" >&2
  cat "$PATCH_PREP_APPLIED_OUT" >&2 || true
  exit 1
fi

for required_section in \
  "## Summary" \
  "## Recommended Patch Plan" \
  "## Likely Files" \
  "## Verification Commands" \
  "## Risk Notes" \
  "## Acceptance Criteria" \
  "## Forbidden Actions Confirmed"
do
  if ! grep -q "$required_section" "$PATCH_PREP_HANDOFF_OUT"; then
    echo "FAIL: generated handoff missing section: $required_section" >&2
    cat "$PATCH_PREP_HANDOFF_OUT" >&2 || true
    exit 1
  fi
done

if python3 "$APPLY_DECISION" "$PATCH_PREP_DRY_RUN_OUT" "$INVALID_DECISION" >/tmp/work_router_dispatch_invalid_decision.out 2>/tmp/work_router_dispatch_invalid_decision.err; then
  echo "FAIL: invalid dispatch decision escalation unexpectedly passed" >&2
  cat /tmp/work_router_dispatch_invalid_decision.out >&2 || true
  exit 1
fi

if ! grep -q "exceeds authority ceiling" /tmp/work_router_dispatch_invalid_decision.err; then
  echo "FAIL: invalid decision escalation failed for unexpected reason" >&2
  cat /tmp/work_router_dispatch_invalid_decision.err >&2 || true
  exit 1
fi

if python3 "$VALIDATOR" "$ESCALATION_FIXTURE" >/tmp/work_router_dispatch_escalation.out 2>/tmp/work_router_dispatch_escalation.err; then
  echo "FAIL: invalid dispatch escalation fixture unexpectedly passed" >&2
  cat /tmp/work_router_dispatch_escalation.out >&2 || true
  exit 1
fi

if ! grep -q "exceeds authority ceiling" /tmp/work_router_dispatch_escalation.err; then
  echo "FAIL: dispatch escalation fixture failed for unexpected reason" >&2
  cat /tmp/work_router_dispatch_escalation.err >&2 || true
  exit 1
fi

if python3 "$VALIDATOR" "$STRUCTURAL_FIXTURE" >/tmp/work_router_dispatch_structural.out 2>/tmp/work_router_dispatch_structural.err; then
  echo "FAIL: invalid structural dispatch fixture unexpectedly passed" >&2
  cat /tmp/work_router_dispatch_structural.out >&2 || true
  exit 1
fi

if ! grep -q "high or structural risk cannot route to code_change_candidate" /tmp/work_router_dispatch_structural.err; then
  echo "FAIL: structural dispatch fixture failed for unexpected reason" >&2
  cat /tmp/work_router_dispatch_structural.err >&2 || true
  exit 1
fi

echo "PASS: WorkRouter dispatch fixtures and authority checks"
