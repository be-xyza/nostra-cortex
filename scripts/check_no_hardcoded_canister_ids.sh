#!/usr/bin/env bash
set -euo pipefail

# This script checks for hardcoded IC Principals (canister IDs) in the Rust codebase.
# They should be managed via ConfigService and environment variables.

echo "🔍 Checking for hardcoded canister IDs..."

# Pattern: Principal::from_text("...")
# We ignore .did files and tests if they are in test blocks
EXIT_CODE=0

# Search for common canister ID patterns (base32-ish, 27 chars)
# This is a heuristic but effective for standard IC IDs
HARDCODED=$(grep -rE "Principal::from_text\(\"[a-z0-9-]{27,}\"\)" nostra/worker/src --exclude-dir=tests || true)

if [ -n "$HARDCODED" ]; then
    echo "❌ ERROR: Found hardcoded canister IDs in the following files:"
    echo "$HARDCODED"
    echo ""
    echo "Please use ConfigService::get().get_canister_id(\"...\") instead."
    EXIT_CODE=1
else
    echo "✅ No hardcoded canister IDs found in worker source."
fi

exit $EXIT_CODE
