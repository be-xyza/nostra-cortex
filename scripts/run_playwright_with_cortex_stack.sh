#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
FRONTEND_DIR="$ROOT_DIR/nostra/frontend"
GATEWAY_MANIFEST="$CORTEX_DESKTOP_MANIFEST"

port_in_use() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
  else
    ss -ltn "( sport = :$port )" 2>/dev/null | tail -n +2 | grep -q .
  fi
}

pick_free_port() {
  local candidate="$1"
  while port_in_use "$candidate"; do
    candidate=$((candidate + 1))
  done
  printf '%s' "$candidate"
}

GATEWAY_PORT="$(pick_free_port "${CORTEX_GATEWAY_PORT:-3001}")"
FRONTEND_PORT="$(pick_free_port "${NOSTRA_UI_TEST_PORT:-3011}")"
if [[ "$FRONTEND_PORT" == "$GATEWAY_PORT" ]]; then
  FRONTEND_PORT="$(pick_free_port "$((GATEWAY_PORT + 1))")"
fi
BASE_URL="${BASE_URL:-http://127.0.0.1:${FRONTEND_PORT}}"
PLAYWRIGHT_WORKERS="${PLAYWRIGHT_WORKERS:-1}"

mkdir -p "$ROOT_DIR/.cache/reports"
GATEWAY_LOG="$ROOT_DIR/.cache/reports/cortex_gateway_for_playwright.log"
FRONTEND_LOG="$ROOT_DIR/.cache/reports/dx_serve_for_playwright.log"

GATEWAY_PID=""
FRONTEND_PID=""

cleanup() {
  if [[ -n "$FRONTEND_PID" ]] && kill -0 "$FRONTEND_PID" >/dev/null 2>&1; then
    kill "$FRONTEND_PID" >/dev/null 2>&1 || true
    wait "$FRONTEND_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$GATEWAY_PID" ]] && kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
    kill "$GATEWAY_PID" >/dev/null 2>&1 || true
    wait "$GATEWAY_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "== Starting Cortex Gateway on port ${GATEWAY_PORT}"
(
  cd "$ROOT_DIR"
  CORTEX_GATEWAY_PORT="$GATEWAY_PORT" cargo run \
    --quiet \
    --manifest-path "$GATEWAY_MANIFEST" \
    --bin gateway_server
) >"$GATEWAY_LOG" 2>&1 &
GATEWAY_PID=$!

gateway_ready=false
for _ in $(seq 1 120); do
  if curl -fsS "http://127.0.0.1:${GATEWAY_PORT}/api/health" >/dev/null 2>&1; then
    gateway_ready=true
    break
  fi
  if ! kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if [[ "$gateway_ready" != "true" ]]; then
  echo "Gateway failed to become ready on port ${GATEWAY_PORT}" >&2
  tail -n 120 "$GATEWAY_LOG" >&2 || true
  exit 1
fi
if ! kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
  echo "Gateway process exited unexpectedly on port ${GATEWAY_PORT}" >&2
  tail -n 120 "$GATEWAY_LOG" >&2 || true
  exit 1
fi

echo "== Starting frontend on ${BASE_URL}"
(
  cd "$FRONTEND_DIR"
  TERM=xterm-256color COLORTERM=truecolor dx serve \
    --platform web \
    --open false \
    --watch false \
    --hot-reload false \
    --addr 127.0.0.1 \
    --port "$FRONTEND_PORT"
) >"$FRONTEND_LOG" 2>&1 &
FRONTEND_PID=$!

frontend_ready=false
for _ in $(seq 1 120); do
  if curl -fsS "$BASE_URL" >/dev/null 2>&1; then
    frontend_ready=true
    break
  fi
  if ! kill -0 "$FRONTEND_PID" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if [[ "$frontend_ready" != "true" ]]; then
  echo "Frontend failed to become ready at ${BASE_URL}" >&2
  tail -n 120 "$FRONTEND_LOG" >&2 || true
  exit 1
fi
if ! kill -0 "$FRONTEND_PID" >/dev/null 2>&1; then
  echo "Frontend process exited unexpectedly before Playwright startup" >&2
  tail -n 120 "$FRONTEND_LOG" >&2 || true
  exit 1
fi

# `dx serve` may be reachable before the first wasm bundle is fully primed.
sleep 5
curl -fsS "${BASE_URL}/?e2e=1" >/dev/null 2>&1 || true

echo "== Running Playwright"
cd "$ROOT_DIR"
BASE_URL="$BASE_URL" npx playwright test --workers "$PLAYWRIGHT_WORKERS" "$@"
