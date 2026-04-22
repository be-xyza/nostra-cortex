#!/usr/bin/env bash
set -euo pipefail

GATEWAY_BASE="${1:-http://127.0.0.1:3000}"
SPACE_ID="${2:-nostra-governance-v0}"

BUILD_JSON="$(curl -sS "${GATEWAY_BASE}/api/system/build")"
READY_JSON="$(curl -sS "${GATEWAY_BASE}/api/system/ready")"
OVERVIEW_JSON="$(curl -sS "${GATEWAY_BASE}/api/kg/spaces/${SPACE_ID}/contribution-graph/overview")"

TMP_BUILD="$(mktemp)"
TMP_READY="$(mktemp)"
TMP_OVERVIEW="$(mktemp)"
printf '%s' "$BUILD_JSON" > "$TMP_BUILD"
printf '%s' "$READY_JSON" > "$TMP_READY"
printf '%s' "$OVERVIEW_JSON" > "$TMP_OVERVIEW"

node -e '
const fs = require("fs");
const build = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
const ready = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const overview = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));

const req = (obj, key) => {
  if (!(key in obj)) {
    console.error(`FAIL: missing key "${key}"`);
    process.exit(1);
  }
};

["buildId", "buildTimeUtc", "gatewayDispatchMode", "gatewayPort", "workspaceRoot"].forEach((k) => req(build, k));
["ready", "gateway_port"].forEach((k) => req(ready, k));
req(ready, "icp_network_healthy");
["latestGraphMetrics", "latestPathSummary", "health"].forEach((k) => req(overview, k));

const metrics = overview.latestGraphMetrics || {};
const goals = (((overview.latestPathSummary || {}).goals) || []);
const rec = goals[0]?.recommendedPath || "n/a";

console.log(`PASS: dual-host gateway baseline`);
console.log(`  build=${build.buildId} mode=${build.gatewayDispatchMode} port=${build.gatewayPort}`);
const icpHealth = ready.icp_network_healthy;
console.log(`  ready=${ready.ready} icp=${icpHealth} unresolvedRefs=${metrics.unresolvedRefs ?? "n/a"}`);
console.log(`  graph hash=${metrics.hash ?? "n/a"} nodes=${metrics.nodes ?? "n/a"} edges=${metrics.edges ?? "n/a"}`);
console.log(`  recommended_path=${rec}`);
' "$TMP_BUILD" "$TMP_READY" "$TMP_OVERVIEW"

rm -f "$TMP_BUILD" "$TMP_READY" "$TMP_OVERVIEW"
