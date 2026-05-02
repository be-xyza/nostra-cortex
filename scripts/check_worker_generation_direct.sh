#!/usr/bin/env bash
set -euo pipefail

WORKER_BASE_URL="${NOSTRA_WORKER_LIVE_GENERATION_URL:-http://127.0.0.1:3003}"
EXPECTED_MODEL="${NOSTRA_EXPECTED_WORKER_GENERATION_MODEL:-~moonshotai/kimi-latest}"
SMOKE_EXPECTED="${NOSTRA_WORKER_GENERATION_SMOKE_EXPECTED:-worker-generation-smoke-ok}"

json_field() {
  local field="$1"
  python3 -c '
import json
import sys

field = sys.argv[1]
payload = json.load(sys.stdin)
print(payload.get(field) or "")
' "$field"
}

payload() {
  python3 - "$SMOKE_EXPECTED" <<'PY'
import json
import sys

expected = sys.argv[1]
print(json.dumps({
    "question": f"Say exactly: {expected}",
    "contexts": [f"chunk-id: smoke-test\nThe required answer is {expected}."],
    "strictGrounding": True,
}))
PY
}

health="$(curl -fsS "$WORKER_BASE_URL/health/model")"
model="$(printf '%s' "$health" | json_field generation_model)"
if [[ -z "$model" ]]; then
  model="$(printf '%s' "$health" | json_field generationModel)"
fi

if [[ "$model" != "$EXPECTED_MODEL" ]]; then
  echo "FAIL: direct worker generation model mismatch: expected $EXPECTED_MODEL got ${model:-<missing>}" >&2
  exit 1
fi

response="$(
  curl -fsS -X POST "$WORKER_BASE_URL/generation/grounded" \
    -H "Content-Type: application/json" \
    --data "$(payload)"
)"
answer="$(printf '%s' "$response" | json_field answer)"
response_model="$(printf '%s' "$response" | json_field model)"

if [[ "$response_model" != "$EXPECTED_MODEL" ]]; then
  echo "FAIL: direct worker generation response model mismatch: expected $EXPECTED_MODEL got ${response_model:-<missing>}" >&2
  exit 1
fi

if [[ "$answer" != *"$SMOKE_EXPECTED"* ]]; then
  echo "FAIL: direct worker generation smoke answer mismatch: expected token $SMOKE_EXPECTED got ${answer:-<missing>}" >&2
  exit 1
fi

echo "PASS: direct worker generation smoke"
echo "model=$response_model"
