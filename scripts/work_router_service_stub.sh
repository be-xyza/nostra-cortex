#!/usr/bin/env bash
set -euo pipefail

if [[ "${WORK_ROUTER_MAX_DISPATCH_LEVEL:-}" != "D1" ]]; then
  echo "cortex-workrouter refused to start: WORK_ROUTER_MAX_DISPATCH_LEVEL must be D1" >&2
  exit 1
fi

for flag in WORK_ROUTER_SOURCE_MUTATION_ALLOWED WORK_ROUTER_RUNTIME_MUTATION_ALLOWED WORK_ROUTER_LIVE_TRANSPORT_ENABLED; do
  if [[ "${!flag:-}" != "0" ]]; then
    echo "cortex-workrouter refused to start: $flag must be 0 for v1 bootstrap" >&2
    exit 1
  fi
done

if [[ "${WORK_ROUTER_REQUIRE_REQUEST_ID:-}" != "1" ]]; then
  echo "cortex-workrouter refused to start: WORK_ROUTER_REQUIRE_REQUEST_ID must be 1" >&2
  exit 1
fi

if [[ "${WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY:-}" != "1" ]]; then
  echo "cortex-workrouter refused to start: WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY must be 1" >&2
  exit 1
fi

if [[ "${WORK_ROUTER_TRANSPORTS_ENABLED:-}" != "cli" ]]; then
  echo "cortex-workrouter refused to start: bootstrap transports must be cli only" >&2
  exit 1
fi

echo "cortex-workrouter observe service starting"
exec python3 "$(dirname "$0")/work_router_observe_service.py"
