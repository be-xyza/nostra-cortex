#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

ENV_FILE="$TMP_DIR/runtime.env"
SAMPLE_BODY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
SAMPLE_VALUE="sk-or-v1-${SAMPLE_BODY}"

cat > "$ENV_FILE" <<EOF
NOSTRA_LLM_API_KEY=$SAMPLE_VALUE
NOSTRA_PUBLIC_MODE=enabled
NOSTRA_WEBHOOK_SECRET=short
EOF

output="$(bash "$ROOT_DIR/scripts/inspect_runtime_config_redacted.sh" --env-file "$ENV_FILE")"

if grep -Fq "$SAMPLE_VALUE" <<<"$output"; then
  echo "FAIL: redacted inspection emitted raw secret"
  exit 1
fi

if ! grep -Fq '"name": "NOSTRA_LLM_API_KEY"' <<<"$output"; then
  echo "FAIL: redacted inspection missing secret key metadata"
  exit 1
fi

if ! grep -Fq '"fingerprint": "sha256:' <<<"$output"; then
  echo "FAIL: redacted inspection missing fingerprint"
  exit 1
fi

echo "PASS: runtime config redaction does not emit raw secrets"
