#!/usr/bin/env bash
set -euo pipefail

GATEWAY_BASE_URL="${NOSTRA_GATEWAY_URL:-http://127.0.0.1:3000}"
WORKER_BASE_URL="${NOSTRA_WORKER_LIVE_GENERATION_URL:-http://127.0.0.1:3003}"
EXPECTED_MODEL="${NOSTRA_EXPECTED_WORKER_GENERATION_MODEL:-~moonshotai/kimi-latest}"
SMOKE_EXPECTED="${NOSTRA_WORKER_GENERATION_SMOKE_EXPECTED:-kimi-worker-smoke-ok}"
GATEWAY_ROLE="${NOSTRA_GATEWAY_OPERATOR_ROLE:-operator}"
GATEWAY_PRINCIPAL="${NOSTRA_GATEWAY_OPERATOR_PRINCIPAL:-}"

gateway_curl_args=(-H "x-cortex-role: $GATEWAY_ROLE")
if [[ -n "$GATEWAY_PRINCIPAL" ]]; then
  gateway_curl_args+=(-H "x-ic-principal: $GATEWAY_PRINCIPAL")
fi

json_get() {
  local url="$1"
  curl -fsS "$url"
}

gateway_get() {
  local url="$1"
  curl -fsS "${gateway_curl_args[@]}" "$url"
}

json_post() {
  local url="$1"
  local payload="$2"
  curl -fsS -X POST "$url" -H "Content-Type: application/json" --data "$payload"
}

gateway_post() {
  local url="$1"
  local payload="$2"
  curl -fsS "${gateway_curl_args[@]}" -X POST "$url" -H "Content-Type: application/json" --data "$payload"
}

health="$(gateway_get "$GATEWAY_BASE_URL/api/system/worker-generation/health")"
model="$(printf '%s' "$health" | jq -r '.generationModel // .generation_model // empty')"
if [[ "$model" != "$EXPECTED_MODEL" ]]; then
  echo "FAIL: worker generation model mismatch: expected $EXPECTED_MODEL got ${model:-<missing>}" >&2
  exit 1
fi

direct_health="$(json_get "$WORKER_BASE_URL/health/model")"
direct_model="$(printf '%s' "$direct_health" | jq -r '.generation_model // .generationModel // empty')"
if [[ "$direct_model" != "$EXPECTED_MODEL" ]]; then
  echo "FAIL: direct worker generation model mismatch: expected $EXPECTED_MODEL got ${direct_model:-<missing>}" >&2
  exit 1
fi

payload="$(jq -nc --arg expected "$SMOKE_EXPECTED" '{
  question: ("Say exactly: " + $expected),
  contexts: [("chunk-id: smoke-test\nThe required answer is " + $expected + ".")],
  strictGrounding: true
}')"
response="$(gateway_post "$GATEWAY_BASE_URL/api/system/worker-generation/grounded" "$payload")"
answer="$(printf '%s' "$response" | jq -r '.answer // empty')"
if [[ "$answer" != *"$SMOKE_EXPECTED"* ]]; then
  echo "FAIL: worker generation smoke answer mismatch: expected token $SMOKE_EXPECTED got ${answer:-<missing>}" >&2
  exit 1
fi

echo "PASS: worker generation gateway smoke"
echo "model=$model"
