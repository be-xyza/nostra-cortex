#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

cargo build \
  --manifest-path "$ROOT_DIR/cortex/Cargo.toml" \
  -p cortex-eudaemon \
  --lib \
  --bins
