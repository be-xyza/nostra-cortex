#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
OUT_DIR="$ROOT_DIR/logs/testing"
OUT_FILE="$OUT_DIR/test_catalog_latest.json"
RUNS_DIR="$OUT_DIR/runs"
TMP_FILE="$(mktemp)"
CORTEX_DESKTOP_PATH_REL="${CORTEX_DESKTOP_DIR#"$ROOT_DIR"/}"

mkdir -p "$OUT_DIR" "$RUNS_DIR"
printf '[]\n' > "$TMP_FILE"

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
GIT_COMMIT="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
ARTIFACTS_TESTING_JSON="[\"$ROOT_DIR/test-results\",\"$ROOT_DIR/logs/testing\"]"
ARTIFACTS_PLAYWRIGHT_JSON="[\"$ROOT_DIR/test-results\",\"$ROOT_DIR/nostra/frontend/playwright-report\"]"
ARTIFACTS_MOTOKO_JSON="[\"$ROOT_DIR/logs/testing\"]"
ARTIFACTS_KNOWLEDGE_BENCH_JSON="[\"$ROOT_DIR/logs/knowledge/retrieval_benchmark_latest.json\"]"
ARTIFACTS_KNOWLEDGE_ROLLBACK_JSON="[\"$ROOT_DIR/logs/knowledge/rollback_drill_latest.json\"]"
ARTIFACTS_KNOWLEDGE_UI_SURFACE_JSON="[\"$ROOT_DIR/logs/knowledge/ui_surface_matrix_latest.json\"]"
ARTIFACTS_KNOWLEDGE_UI_INTERACTION_JSON="[\"$ROOT_DIR/logs/knowledge/ui_interaction_matrix_latest.json\"]"
ARTIFACTS_KNOWLEDGE_UI_PLAYWRIGHT_JSON="[\"$ROOT_DIR/logs/knowledge/ui_playwright_matrix_latest.json\",\"$ROOT_DIR/logs/knowledge/ui_playwright_report_latest.json\"]"
ARTIFACTS_THEME_JSON="[\"$ROOT_DIR/logs/testing/cortex_ui_theme_conformance_latest.json\"]"
ARTIFACTS_OFFLINE_SIM_JSON="[\"$ROOT_DIR/logs/testing/offline_projection_playwright_latest.json\",\"$ROOT_DIR/logs/testing/offline_projection_playwright_report_latest.json\"]"
ARTIFACTS_KNOWLEDGE_CONTRACT_JSON="[\"$ROOT_DIR/logs/knowledge/ui_contract_matrix_latest.json\"]"

append_entry() {
  local test_id="$1"
  local name="$2"
  local layer="$3"
  local stack="$4"
  local owner="$5"
  local path="$6"
  local command="$7"
  local gate_level="$8"
  local tags_csv="$9"
  local artifacts_json="${10}"

  jq \
    --arg test_id "$test_id" \
    --arg name "$name" \
    --arg layer "$layer" \
    --arg stack "$stack" \
    --arg owner "$owner" \
    --arg path "$path" \
    --arg command "$command" \
    --arg gate_level "$gate_level" \
    --arg tags_csv "$tags_csv" \
    --arg git_commit "$GIT_COMMIT" \
    --arg updated_at "$NOW_UTC" \
    --argjson artifacts "$artifacts_json" \
    '. += [{
      test_id: $test_id,
      name: $name,
      layer: $layer,
      stack: $stack,
      owner: $owner,
      path: $path,
      command: $command,
      artifacts: $artifacts,
      gate_level: $gate_level,
      destructive: false,
      tags: ($tags_csv | split(",") | map(select(length > 0))),
      last_seen_commit: $git_commit,
      updated_at: $updated_at
    }]' "$TMP_FILE" > "$TMP_FILE.next"
  mv "$TMP_FILE.next" "$TMP_FILE"
}

normalize_id() {
  local path="$1"
  echo "$path" | sed 's#^/##; s#[^A-Za-z0-9/_-]#_#g; s#/#:#g'
}

rust_command_for() {
  local path="$1"
  if [[ "$path" == nostra/worker/* ]]; then
    echo "cargo test --manifest-path nostra/worker/Cargo.toml --lib -- --skip skills::hrm_scheduler::tests::test_hrm_demo_execution"
  elif [[ "$path" == "$CORTEX_DESKTOP_PATH_REL"/* ]]; then
    echo "cargo test --manifest-path $CORTEX_DESKTOP_PATH_REL/Cargo.toml"
  elif [[ "$path" == cortex/apps/cortex-worker/* ]]; then
    echo "cargo test --manifest-path cortex/apps/cortex-worker/Cargo.toml"
  elif [[ "$path" == nostra/backend/workflow_engine/* ]]; then
    echo "cargo test --manifest-path nostra/backend/workflow_engine/Cargo.toml"
  elif [[ "$path" == nostra/backend/governance/* ]]; then
    echo "cargo test --manifest-path nostra/backend/governance/Cargo.toml"
  elif [[ "$path" == nostra/frontend/src/* ]]; then
    echo "cargo test --manifest-path nostra/frontend/Cargo.toml"
  else
    echo "cargo test"
  fi
}

rust_owner_for() {
  local path="$1"
  if [[ "$path" == nostra/frontend/src/* ]]; then
    echo "UX Steward"
  else
    echo "Systems Steward"
  fi
}

rust_layer_for() {
  local path="$1"
  if [[ "$path" == *"benchmark"* ]]; then
    echo "L4_BENCHMARK"
  elif [[ "$path" == *"drill"* ]]; then
    echo "L5_DRILL"
  elif [[ "$path" == *"/tests/"* || "$path" == *"_test.rs" || "$path" == *"integration"* ]]; then
    echo "L2_INTEGRATION"
  else
    echo "L1_UNIT"
  fi
}

rust_gate_for() {
  local path="$1"
  if [[ "$path" == nostra/worker/* || "$path" == nostra/backend/workflow_engine/* || "$path" == "$CORTEX_DESKTOP_PATH_REL"/tests/* ]]; then
    echo "release_blocker"
  else
    echo "informational"
  fi
}

while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  [[ "$rel" == *"/_archive/"* ]] && continue
  if grep -Eq '^[[:space:]]*#\[(tokio::)?test\]' "$file"; then
    layer="$(rust_layer_for "$rel")"
    gate="$(rust_gate_for "$rel")"
    owner="$(rust_owner_for "$rel")"
    command="$(rust_command_for "$rel")"
    test_id="rust:$(normalize_id "$rel")"
    name="$(basename "$rel")"
    append_entry "$test_id" "$name" "$layer" "rust" "$owner" "$rel" "$command" "$gate" "rust,$layer" "$ARTIFACTS_TESTING_JSON"
  fi
done < <(find \
  "$ROOT_DIR/nostra/worker" \
  "$ROOT_DIR/nostra/backend" \
  "$CORTEX_DESKTOP_DIR" \
  "$ROOT_DIR/cortex/apps/cortex-worker" \
  "$ROOT_DIR/nostra/frontend/src" \
  -type f -name '*.rs' 2>/dev/null)

while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  test_id="ts:$(normalize_id "$rel")"
  name="$(basename "$rel")"
  if [[ "$rel" == tests/* ]]; then
    command="npm run test:e2e -- $rel"
  else
    command="cd nostra/frontend/e2e && npx playwright test $(basename "$rel")"
  fi
  append_entry "$test_id" "$name" "L3_E2E" "ts" "UX Steward" "$rel" "$command" "informational" "ts,L3_E2E" "$ARTIFACTS_PLAYWRIGHT_JSON"
done < <(find \
  "$ROOT_DIR/tests" \
  "$ROOT_DIR/nostra/frontend/e2e" \
  -type f \( -name '*.spec.ts' -o -name '*.test.ts' -o -name '*.test.tsx' -o -name '*.test.js' \) 2>/dev/null)

while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  gate="informational"
  test_id="motoko:$(normalize_id "$rel")"
  name="$(basename "$rel")"
  append_entry "$test_id" "$name" "L2_INTEGRATION" "motoko" "Systems Steward" "$rel" "cd nostra && mops test" "$gate" "motoko,L2_INTEGRATION" "$ARTIFACTS_MOTOKO_JSON"
done < <(find \
  "$ROOT_DIR/nostra/backend" \
  "$ROOT_DIR/nostra/labs" \
  "$ROOT_DIR/nostra/registry" \
  -type f -name '*test*.mo' 2>/dev/null)

append_entry \
  "mixed:scripts:knowledge-closeout-benchmark.sh" \
  "knowledge-closeout-benchmark.sh" \
  "L4_BENCHMARK" \
  "mixed" \
  "Systems Steward" \
  "scripts/knowledge-closeout-benchmark.sh" \
  "bash scripts/knowledge-closeout-benchmark.sh" \
  "release_blocker" \
  "knowledge,L4_BENCHMARK" \
  "$ARTIFACTS_KNOWLEDGE_BENCH_JSON"

append_entry \
  "mixed:scripts:knowledge-shadow-rollback-drill.sh" \
  "knowledge-shadow-rollback-drill.sh" \
  "L5_DRILL" \
  "mixed" \
  "Systems Steward" \
  "scripts/knowledge-shadow-rollback-drill.sh" \
  "bash scripts/knowledge-shadow-rollback-drill.sh" \
  "informational" \
  "knowledge,L5_DRILL" \
  "$ARTIFACTS_KNOWLEDGE_ROLLBACK_JSON"

append_entry \
  "mixed:scripts:check-semantic-search-ui-surfaces.sh" \
  "check-semantic-search-ui-surfaces.sh" \
  "L2_INTEGRATION" \
  "mixed" \
  "UX Steward" \
  "scripts/check_semantic_search_ui_surfaces.sh" \
  "bash scripts/check_semantic_search_ui_surfaces.sh" \
  "release_blocker" \
  "knowledge,ui,L2_INTEGRATION" \
  "$ARTIFACTS_KNOWLEDGE_UI_SURFACE_JSON"

append_entry \
  "mixed:scripts:check-semantic-search-ui-interactions.sh" \
  "check-semantic-search-ui-interactions.sh" \
  "L2_INTEGRATION" \
  "mixed" \
  "UX Steward" \
  "scripts/check_semantic_search_ui_interactions.sh" \
  "bash scripts/check_semantic_search_ui_interactions.sh" \
  "release_blocker" \
  "knowledge,ui,L2_INTEGRATION" \
  "$ARTIFACTS_KNOWLEDGE_UI_INTERACTION_JSON"

append_entry \
  "mixed:scripts:check-semantic-search-ui-playwright.sh" \
  "check-semantic-search-ui-playwright.sh" \
  "L3_E2E" \
  "mixed" \
  "UX Steward" \
  "scripts/check_semantic_search_ui_playwright.sh" \
  "bash scripts/check_semantic_search_ui_playwright.sh" \
  "release_blocker" \
  "knowledge,ui,L3_E2E" \
  "$ARTIFACTS_KNOWLEDGE_UI_PLAYWRIGHT_JSON"

append_entry \
  "mixed:scripts:check-cortex-ui-theme-conformance.sh" \
  "check-cortex-ui-theme-conformance.sh" \
  "L2_INTEGRATION" \
  "mixed" \
  "UX Steward" \
  "scripts/check_cortex_ui_theme_conformance.sh" \
  "bash scripts/check_cortex_ui_theme_conformance.sh" \
  "release_blocker" \
  "cortex,ui,theme,L2_INTEGRATION" \
  "$ARTIFACTS_THEME_JSON"

append_entry \
  "mixed:scripts:check-offline-simulation-playwright.sh" \
  "check-offline-simulation-playwright.sh" \
  "L3_E2E" \
  "mixed" \
  "UX Steward" \
  "scripts/check_offline_simulation_playwright.sh" \
  "bash scripts/check_offline_simulation_playwright.sh" \
  "release_blocker" \
  "offline,blackwell,projection_contract,L3_E2E" \
  "$ARTIFACTS_OFFLINE_SIM_JSON"

append_entry \
  "mixed:scripts:check-semantic-search-worker-contracts.sh" \
  "check-semantic-search-worker-contracts.sh" \
  "L2_INTEGRATION" \
  "mixed" \
  "Systems Steward" \
  "scripts/check_semantic_search_worker_contracts.sh" \
  "bash scripts/check_semantic_search_worker_contracts.sh" \
  "release_blocker" \
  "knowledge,contracts,L2_INTEGRATION" \
  "$ARTIFACTS_KNOWLEDGE_CONTRACT_JSON"

jq \
  --arg schema_version "1.0.0" \
  --arg generated_at "$NOW_UTC" \
  '{
    schema_version: $schema_version,
    generated_at: $generated_at,
    tests: (sort_by(.test_id) | unique_by(.test_id))
  }' "$TMP_FILE" > "$OUT_FILE"

rm -f "$TMP_FILE"
echo "Wrote test catalog: $OUT_FILE"
