#!/usr/bin/env bash
# DEPRECATED: This script has been renamed to run_cortex_daemon.sh.
# This shim exists for backward compatibility and will be removed in a future release.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[DEPRECATED] run_cortex_desktop.sh is deprecated. Use run_cortex_daemon.sh instead." >&2
exec "${SCRIPT_DIR}/run_cortex_daemon.sh" "$@"
