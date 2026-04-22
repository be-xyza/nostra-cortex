#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_DIR="$ROOT_DIR/cortex/apps/cortex-web"
BASE_URL="${BASE_URL:-http://127.0.0.1:4173}"

if ! command -v npx >/dev/null 2>&1; then
  echo "Missing required command: npx" >&2
  exit 1
fi

echo "== Running heap parity Playwright checks against ${BASE_URL}"
(
  cd "$WEB_DIR"
  BASE_URL="$BASE_URL" npx playwright test -c playwright.heap.config.ts tests/playwright/heap-parity.spec.ts --project=chromium
)
