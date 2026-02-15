#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/check_118_pr_evidence.sh"
FIXTURE_DIR="$ROOT_DIR/tests/fixtures/pr_evidence"

expect_pass() {
  local name="$1"
  shift
  if "$@"; then
    echo "PASS: $name"
  else
    echo "FAIL: expected pass: $name"
    exit 1
  fi
}

expect_fail() {
  local name="$1"
  shift
  if "$@"; then
    echo "FAIL: expected failure: $name"
    exit 1
  else
    echo "PASS: $name (failed as expected)"
  fi
}

expect_pass "valid fixture" \
  bash "$CHECK_SCRIPT" --pr-body-file "$FIXTURE_DIR/valid.md"

expect_fail "missing url fixture" \
  bash "$CHECK_SCRIPT" --pr-body-file "$FIXTURE_DIR/missing_url.md"

expect_fail "bad counts fixture" \
  bash "$CHECK_SCRIPT" --pr-body-file "$FIXTURE_DIR/bad_counts.md"

expect_fail "nonzero exemptions fixture" \
  bash "$CHECK_SCRIPT" --pr-body-file "$FIXTURE_DIR/nonzero_exemptions.md"

echo "PASS: check_118_pr_evidence fixtures validated"
