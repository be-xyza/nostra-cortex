#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="${NOSTRA_WORKSPACE_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
LABEL="org.nostra.cortex-gateway-dev"
PLIST_PATH="${HOME}/Library/LaunchAgents/${LABEL}.plist"
LOG_DIR="${WORKSPACE_ROOT}/logs/runtime"
OUT_LOG="${LOG_DIR}/cortex-gateway-dev.out.log"
ERR_LOG="${LOG_DIR}/cortex-gateway-dev.err.log"

mkdir -p "${HOME}/Library/LaunchAgents" "${LOG_DIR}"

cat > "${PLIST_PATH}" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${LABEL}</string>

  <key>ProgramArguments</key>
  <array>
    <string>/bin/zsh</string>
    <string>-lc</string>
    <string>cd '${WORKSPACE_ROOT}' && NOSTRA_WORKSPACE_ROOT='${WORKSPACE_ROOT}' CORTEX_GATEWAY_PORT='3000' CORTEX_GATEWAY_LEGACY_DISPATCH_MODE='http_loopback' bash '${WORKSPACE_ROOT}/scripts/run_cortex_gateway.sh' dev</string>
  </array>

  <key>WorkingDirectory</key>
  <string>${WORKSPACE_ROOT}</string>

  <key>RunAtLoad</key>
  <true/>

  <key>KeepAlive</key>
  <true/>

  <key>StandardOutPath</key>
  <string>${OUT_LOG}</string>

  <key>StandardErrorPath</key>
  <string>${ERR_LOG}</string>

  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
    <key>NOSTRA_WORKSPACE_ROOT</key>
    <string>${WORKSPACE_ROOT}</string>
    <key>CORTEX_GATEWAY_PORT</key>
    <string>3000</string>
    <key>CORTEX_GATEWAY_LEGACY_DISPATCH_MODE</key>
    <string>http_loopback</string>
  </dict>
</dict>
</plist>
PLIST

launchctl bootout "gui/$(id -u)/${LABEL}" >/dev/null 2>&1 || true
launchctl bootstrap "gui/$(id -u)" "${PLIST_PATH}"
launchctl enable "gui/$(id -u)/${LABEL}"
launchctl kickstart -k "gui/$(id -u)/${LABEL}"

echo "installed=${PLIST_PATH}"
echo "label=${LABEL}"
echo "stdout=${OUT_LOG}"
echo "stderr=${ERR_LOG}"
