#!/usr/bin/env bash
set -euo pipefail

workspace_root="${NOSTRA_WORKSPACE_ROOT:-$(git rev-parse --show-toplevel)}"
host_kind="${CORTEX_ACP_HOST_KIND:-eudaemon}"
gateway_port="${CORTEX_GATEWAY_PORT:-4943}"
acp_pilot="${CORTEX_ACP_PILOT:-1}"

case "${host_kind}" in
  eudaemon)
    cd "${workspace_root}/cortex/apps/cortex-eudaemon"
    exec env \
      NOSTRA_WORKSPACE_ROOT="${workspace_root}" \
      CORTEX_ACP_PILOT="${acp_pilot}" \
      CORTEX_GATEWAY_PORT="${gateway_port}" \
      cargo run \
        --manifest-path "${workspace_root}/cortex/Cargo.toml" \
        -p cortex-eudaemon \
        --bin gateway_server
    ;;
  desktop)
    echo "cortex-desktop is not packaged as a runnable gateway binary in this branch yet" >&2
    exit 1
    ;;
  *)
    echo "unsupported CORTEX_ACP_HOST_KIND: ${host_kind}" >&2
    exit 1
    ;;
esac
