#!/bin/bash
set -euo pipefail

RUNTIME_CRATE="cortex-runtime"
MANIFEST_PATH="nostra/Cargo.toml"
FAILURES=0

echo "=== Cortex Runtime Purity Check ==="

echo "--- Check 1: wasm32 no-default-features compilation ---"
if cargo check --manifest-path "$MANIFEST_PATH" --target wasm32-unknown-unknown -p "$RUNTIME_CRATE" --no-default-features >/dev/null 2>&1; then
  echo "PASS: $RUNTIME_CRATE compiles for wasm32-unknown-unknown (--no-default-features)"
else
  if ! cargo tree --manifest-path "$MANIFEST_PATH" -p "$RUNTIME_CRATE" >/dev/null 2>&1; then
    echo "SKIP: $RUNTIME_CRATE not found in workspace"
    exit 0
  fi
  echo "FAIL: $RUNTIME_CRATE failed wasm32 no-default-features compilation"
  FAILURES=$((FAILURES + 1))
fi

echo "--- Check 2: Forbidden dependencies ---"
FORBIDDEN_DEPS="tokio|reqwest|ic-agent|candid"
if cargo tree --manifest-path "$MANIFEST_PATH" -p "$RUNTIME_CRATE" 2>/dev/null | grep -qE "$FORBIDDEN_DEPS"; then
  echo "FAIL: forbidden dependencies detected in $RUNTIME_CRATE"
  cargo tree --manifest-path "$MANIFEST_PATH" -p "$RUNTIME_CRATE" | grep -E "$FORBIDDEN_DEPS" || true
  FAILURES=$((FAILURES + 1))
else
  echo "PASS: no forbidden dependencies"
fi

echo "--- Check 3: Forbidden direct runtime APIs ---"
RUNTIME_SRC="nostra/libraries/cortex-runtime/src"
if rg -n "SystemTime::now|Instant::now|Utc::now|chrono::Utc::now|println!|eprintln!|tracing::" "$RUNTIME_SRC" >/dev/null 2>&1; then
  echo "FAIL: direct side-effect/time APIs found in cortex-runtime"
  rg -n "SystemTime::now|Instant::now|Utc::now|chrono::Utc::now|println!|eprintln!|tracing::" "$RUNTIME_SRC" || true
  FAILURES=$((FAILURES + 1))
else
  echo "PASS: runtime uses injected ports for time/logging"
fi

echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "✅ Cortex runtime purity checks passed"
  exit 0
else
  echo "❌ $FAILURES cortex runtime purity check(s) failed"
  exit 1
fi
