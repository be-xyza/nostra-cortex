#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TMP_DIR="$(mktemp -d)"
TEST_INITIATIVE="138-protected-resources-and-secret-egress"
TEST_SUBDIR="_tmp_secret_scan_promotion_test"
DEST_DIR="$ROOT_DIR/research/$TEST_INITIATIVE/$TEST_SUBDIR"
trap 'rm -rf "$TMP_DIR" "$DEST_DIR"' EXIT

SAFE_ARTIFACT="$TMP_DIR/safe-evidence.txt"
LEAK_ARTIFACT="$TMP_DIR/leaked-evidence.txt"
FAKE_SECRET="sk-or-v1-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"

cat > "$SAFE_ARTIFACT" <<'EOF'
Redacted evidence: provider key present=true fingerprint=sha256:example
EOF

cat > "$LEAK_ARTIFACT" <<EOF
provider_key=$FAKE_SECRET
EOF

bash "$ROOT_DIR/scripts/promote_evidence_artifact.sh" \
  --source "$SAFE_ARTIFACT" \
  --initiative "$TEST_INITIATIVE" \
  --subdir "$TEST_SUBDIR" >/dev/null

set +e
leak_output="$(bash "$ROOT_DIR/scripts/promote_evidence_artifact.sh" \
  --source "$LEAK_ARTIFACT" \
  --initiative "$TEST_INITIATIVE" \
  --subdir "$TEST_SUBDIR" 2>&1)"
leak_status=$?
set -e

if [[ "$leak_status" -eq 0 ]]; then
  echo "FAIL: leaked evidence artifact was promoted"
  exit 1
fi

if grep -Fq "$FAKE_SECRET" <<<"$leak_output"; then
  echo "FAIL: promotion scanner emitted raw secret"
  exit 1
fi

echo "PASS: evidence promotion blocks secret-bearing artifacts"
