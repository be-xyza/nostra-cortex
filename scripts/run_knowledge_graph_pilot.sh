#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

cargo run \
  --manifest-path "$ROOT_DIR/cortex/apps/cortex-eudaemon/Cargo.toml" \
  --no-default-features \
  --features knowledge-graph-tests \
  --bin knowledge_graph_pilot_runner
