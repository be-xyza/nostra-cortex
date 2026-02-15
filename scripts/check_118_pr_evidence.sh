#!/bin/bash
set -euo pipefail

# check_118_pr_evidence.sh
# Usage:
#   bash scripts/check_118_pr_evidence.sh --pr-body-file /tmp/pr_body.md
#   bash scripts/check_118_pr_evidence.sh --pr-body-file /tmp/pr_body.md --require-freeze-evidence false --scope-includes-118 false
#
# Example local validation command:
#   bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md

print_usage() {
  cat <<'EOF'
Usage:
  check_118_pr_evidence.sh --pr-body-file <path> [--require-freeze-evidence true|false] [--scope-includes-118 true|false]

Options:
  --pr-body-file              Path to markdown file containing PR description/body.
  --require-freeze-evidence   Whether freeze evidence is required. Default: true.
  --scope-includes-118        Whether CI path matcher determined this PR is in 118 scope. Default: true.
EOF
}

PR_BODY_FILE=""
REQUIRE_FREEZE_EVIDENCE="true"
SCOPE_INCLUDES_118="true"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pr-body-file)
      PR_BODY_FILE="${2:-}"
      shift 2
      ;;
    --require-freeze-evidence)
      REQUIRE_FREEZE_EVIDENCE="${2:-}"
      shift 2
      ;;
    --scope-includes-118)
      SCOPE_INCLUDES_118="${2:-}"
      shift 2
      ;;
    -h|--help)
      print_usage
      exit 0
      ;;
    *)
      echo "FAIL: unknown argument: $1"
      print_usage
      exit 1
      ;;
  esac
done

if [[ -z "$PR_BODY_FILE" ]]; then
  echo "FAIL: --pr-body-file is required"
  print_usage
  exit 1
fi

if [[ ! -f "$PR_BODY_FILE" ]]; then
  echo "FAIL: PR body file not found: $PR_BODY_FILE"
  exit 1
fi

case "$REQUIRE_FREEZE_EVIDENCE" in
  true|false) ;;
  *)
    echo "FAIL: --require-freeze-evidence must be true or false"
    exit 1
    ;;
esac

case "$SCOPE_INCLUDES_118" in
  true|false) ;;
  *)
    echo "FAIL: --scope-includes-118 must be true or false"
    exit 1
    ;;
esac

declare -a FAILURES=()

has_out_of_scope_override() {
  grep -Eq '(^|[[:space:]`])118_SCOPE_APPLIES=no([[:space:]`]|$)' "$PR_BODY_FILE"
}

has_scope_marker_yes() {
  grep -Eq '(^|[[:space:]`])118_SCOPE_APPLIES=yes([[:space:]`]|$)' "$PR_BODY_FILE"
}

if has_out_of_scope_override; then
  if [[ "$SCOPE_INCLUDES_118" == "true" ]]; then
    FAILURES+=("118_SCOPE_APPLIES=no override is invalid because CI path matcher marked this PR as 118 scope")
  else
    echo "PASS: 118 scope override accepted for out-of-scope PR (118_SCOPE_APPLIES=no)"
    exit 0
  fi
fi

if [[ "$REQUIRE_FREEZE_EVIDENCE" == "false" ]]; then
  if [[ ${#FAILURES[@]} -gt 0 ]]; then
    for failure in "${FAILURES[@]}"; do
      echo "FAIL: $failure"
    done
    exit 1
  fi
  echo "PASS: freeze-gate evidence check skipped (--require-freeze-evidence=false)"
  exit 0
fi

if ! has_scope_marker_yes; then
  FAILURES+=("missing scope marker: 118_SCOPE_APPLIES=yes")
fi

if ! grep -Eq '\- \[[xX]\] `cortex-runtime-freeze-gates` is green on this PR head' "$PR_BODY_FILE"; then
  FAILURES+=("missing checked box for PR-head freeze gate")
fi

if ! grep -Eq '\- \[[xX]\] `cortex-runtime-freeze-gates` is green on latest `main`' "$PR_BODY_FILE"; then
  FAILURES+=("missing checked box for latest-main freeze gate")
fi

ACTION_URL_COUNT="$(
  grep -Eo 'https://github\.com/[^/[:space:]]+/[^/[:space:]]+/actions/runs/[0-9]+' "$PR_BODY_FILE" \
    | sort -u \
    | wc -l \
    | tr -d ' '
)"

if [[ "$ACTION_URL_COUNT" -ne 1 ]]; then
  FAILURES+=("expected exactly one GitHub Actions run URL, found $ACTION_URL_COUNT")
fi

COUNT_LINE="$(grep -Eo 'inventory=[0-9]+[[:space:]]+fixtures=[0-9]+[[:space:]]+exemptions=[0-9]+' "$PR_BODY_FILE" | head -n1 || true)"
if [[ -z "$COUNT_LINE" ]]; then
  FAILURES+=("missing inventory counts in format: inventory=<n> fixtures=<n> exemptions=<n>")
else
  INVENTORY="$(echo "$COUNT_LINE" | sed -E 's/.*inventory=([0-9]+).*/\1/')"
  FIXTURES="$(echo "$COUNT_LINE" | sed -E 's/.*fixtures=([0-9]+).*/\1/')"
  EXEMPTIONS="$(echo "$COUNT_LINE" | sed -E 's/.*exemptions=([0-9]+).*/\1/')"

  if [[ "$INVENTORY" -ne $((FIXTURES + EXEMPTIONS)) ]]; then
    FAILURES+=("inventory equation invalid: inventory=$INVENTORY fixtures=$FIXTURES exemptions=$EXEMPTIONS")
  fi

  if [[ "$EXEMPTIONS" -ne 0 ]]; then
    FAILURES+=("exemptions must be zero for default policy, found exemptions=$EXEMPTIONS")
  fi
fi

EVIDENCE_ATTACHED_LINE="$(grep -Eio 'Evidence files attached:[[:space:]]*(yes|no)' "$PR_BODY_FILE" | head -n1 || true)"
if [[ -z "$EVIDENCE_ATTACHED_LINE" ]]; then
  FAILURES+=("missing field: Evidence files attached: yes/no")
else
  EVIDENCE_ATTACHED_VALUE="$(echo "$EVIDENCE_ATTACHED_LINE" | sed -E 's/.*:[[:space:]]*(yes|no).*/\1/I' | tr '[:upper:]' '[:lower:]')"
  if [[ "$EVIDENCE_ATTACHED_VALUE" != "yes" ]]; then
    FAILURES+=("evidence files must be attached for 118 scope: set 'Evidence files attached: yes'")
  fi
fi

if [[ ${#FAILURES[@]} -gt 0 ]]; then
  for failure in "${FAILURES[@]}"; do
    echo "FAIL: $failure"
  done
  exit 1
fi

echo "PASS: Initiative 118 freeze-gate evidence is valid"
