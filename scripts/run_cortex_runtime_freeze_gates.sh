#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/logs/testing/freeze_gates"
mkdir -p "$LOG_DIR"

run_step() {
  local name="$1"
  shift
  echo "==> $name"
  "$@" 2>&1 | tee "$LOG_DIR/${name}.log"
}

run_step terminology ./scripts/check_nostra_cortex_terminology.sh
run_step domain_purity ./scripts/check_cortex_domain_purity.sh
run_step runtime_purity ./scripts/check_cortex_runtime_purity.sh
run_step gateway_inventory_sync ./scripts/check_gateway_parity_inventory_sync.sh
run_step domain_wasm cargo check --manifest-path nostra/Cargo.toml --target wasm32-unknown-unknown -p cortex-domain
run_step runtime_wasm cargo check --manifest-path nostra/Cargo.toml --target wasm32-unknown-unknown -p cortex-runtime --no-default-features
run_step gateway_parity cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --test gateway_parity
run_step acp_matrix cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 runtime_v0_matches_legacy_for_all_update_kinds
run_step acp_cloud_event cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 cloud_event_parity_allows_timestamp_format_differences_only
run_step shadow_rejects_drift cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_rejects_non_allowed_drift
run_step shadow_allows_timestamp cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_allows_timestamp_only_drift

echo "✅ Cortex runtime freeze gates passed"
