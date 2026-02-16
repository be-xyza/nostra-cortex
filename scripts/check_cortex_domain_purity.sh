#!/bin/bash
set -euo pipefail

# Cortex Domain Purity Check
# Enforces the Runtime Purity Contract v1.3 for cortex-domain.
# Reference: research/118-cortex-runtime-extraction/PLAN.md

DOMAIN_CRATE="cortex-domain"
MANIFEST_PATH="nostra/Cargo.toml"
FAILURES=0

echo "=== Cortex Domain Purity Check ==="

# 1. WASM compilation check
echo "--- Check 1: wasm32 compilation ---"
if cargo check --manifest-path "$MANIFEST_PATH" --target wasm32-unknown-unknown -p "$DOMAIN_CRATE" >/dev/null 2>&1; then
  echo "PASS: $DOMAIN_CRATE compiles to wasm32-unknown-unknown"
else
  if ! cargo tree --manifest-path "$MANIFEST_PATH" -p "$DOMAIN_CRATE" >/dev/null 2>&1; then
     echo "SKIP: $DOMAIN_CRATE not found in workspace"
     exit 0
  else
     echo "FAIL: $DOMAIN_CRATE does not compile to wasm32-unknown-unknown"
     FAILURES=$((FAILURES + 1))
  fi
fi

# 2. Forbidden dependency check
echo "--- Check 2: Forbidden dependencies ---"
FORBIDDEN_DEPS="tokio|reqwest|ic-agent|candid|^log "
if cargo tree --manifest-path "$MANIFEST_PATH" -p "$DOMAIN_CRATE" 2>/dev/null | grep -qE "$FORBIDDEN_DEPS"; then
  echo "FAIL: $DOMAIN_CRATE has forbidden dependencies:"
  cargo tree --manifest-path "$MANIFEST_PATH" -p "$DOMAIN_CRATE" | grep -E "$FORBIDDEN_DEPS" || true
  FAILURES=$((FAILURES + 1))
else
  echo "PASS: No forbidden dependencies found"
fi

# 3. Forbidden direct runtime/time APIs
echo "--- Check 3: Forbidden direct runtime/time APIs ---"
if rg -n "SystemTime::now|Instant::now|Utc::now|chrono::Utc::now" "nostra/libraries/$DOMAIN_CRATE/src" >/dev/null 2>&1; then
  echo "FAIL: $DOMAIN_CRATE uses wall-clock APIs directly"
  rg -n "SystemTime::now|Instant::now|Utc::now|chrono::Utc::now" "nostra/libraries/$DOMAIN_CRATE/src" || true
  FAILURES=$((FAILURES + 1))
else
  echo "PASS: No wall-clock API usage found"
fi

# 4. Domain layer import boundary (when domain/ module exists)
DOMAIN_SRC=$(cargo metadata --manifest-path "$MANIFEST_PATH" --format-version 1 2>/dev/null \
  | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; matches=[p['manifest_path'] for p in pkgs if p['name']=='$DOMAIN_CRATE']; print(matches[0] if matches else '')" 2>/dev/null \
  | sed 's|/Cargo.toml|/src|' || echo "")

if [ -n "$DOMAIN_SRC" ] && [ -d "${DOMAIN_SRC}/domain" ]; then
  echo "--- Check 4: Domain layer imports ---"
  if grep -rq "crate::application\|crate::ports" "${DOMAIN_SRC}/domain/"; then
    echo "FAIL: domain/ imports from application/ or ports/"
    grep -rn "crate::application\|crate::ports" "${DOMAIN_SRC}/domain/" || true
    FAILURES=$((FAILURES + 1))
  else
    echo "PASS: domain/ has no outward imports"
  fi
fi

# Summary
echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "✅ All purity checks passed"
  exit 0
else
  echo "❌ $FAILURES purity check(s) failed"
  exit 1
fi
