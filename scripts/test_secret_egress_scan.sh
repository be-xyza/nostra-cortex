#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

LEAK_FILE="$TMP_DIR/leak.env"
SAFE_FILE="$TMP_DIR/safe.env"
SAMPLE_BODY="BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"
SAMPLE_VALUE="sk-or-v1-${SAMPLE_BODY}"

cat > "$LEAK_FILE" <<EOF
NOSTRA_LLM_API_KEY=$SAMPLE_VALUE
EOF

cat > "$SAFE_FILE" <<'EOF'
NOSTRA_LLM_API_KEY=<redacted>
EOF

set +e
leak_output="$(python3 "$ROOT_DIR/scripts/check_secret_egress.py" --paths "$LEAK_FILE" 2>&1)"
leak_status=$?
set -e

if [[ "$leak_status" -eq 0 ]]; then
  echo "FAIL: scanner did not fail on high-confidence credential"
  exit 1
fi

if grep -Fq "$SAMPLE_VALUE" <<<"$leak_output"; then
  echo "FAIL: scanner emitted raw secret"
  exit 1
fi

python3 "$ROOT_DIR/scripts/check_secret_egress.py" --paths "$SAFE_FILE" >/dev/null
(cd "$TMP_DIR" && python3 "$ROOT_DIR/scripts/check_secret_egress.py" --paths "$SAFE_FILE" >/dev/null)

echo "PASS: secret egress scanner blocks leaks without emitting raw secrets"
