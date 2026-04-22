#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MODE="observe"

if [[ "${1:-}" == "--mode" ]]; then
  MODE="${2:-observe}"
elif [[ -n "${1:-}" ]]; then
  MODE="$1"
fi

case "$MODE" in
  observe|softgate|hardgate)
    ;;
  *)
    echo "FAIL: unsupported mode '$MODE' (expected observe|softgate|hardgate)" >&2
    exit 2
    ;;
esac

python3 "$ROOT_DIR/scripts/siq_tools.py" refresh --mode "$MODE"

if [[ -f "$ROOT_DIR/scripts/check_siq_artifact_consistency.sh" ]]; then
  bash "$ROOT_DIR/scripts/check_siq_artifact_consistency.sh" --mode "$MODE"
fi
