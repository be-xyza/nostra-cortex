#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WORKSPACE_ROOT="$DIR/../../.."

export VITE_CORTEX_GATEWAY_URL="http://localhost:3000"

echo "Pre-building cortex-gateway to observe any fatal errors without timeouts..."
(cd $WORKSPACE_ROOT && cargo build -p cortex-gateway)

echo "Running full unmocked E2E pipeline..."
python /Users/xaoj/.gemini/antigravity/skills/webapp-testing/scripts/with_server.py.md \
    --timeout 180 \
    --server "cd $WORKSPACE_ROOT && cargo run -p cortex-gateway" --port 3000 \
    --server "cd $DIR/.. && npm run dev -- --host 127.0.0.1 --port 5173" --port 5173 \
    -- python "$DIR/e2e_automation.py"

echo "E2E Run completed successfully"
