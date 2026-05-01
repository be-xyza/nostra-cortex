#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 "$ROOT_DIR/scripts/check_agent_runtime_boundaries.py" "$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_runtime_boundaries.v1.json"
