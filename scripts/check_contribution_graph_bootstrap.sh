#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_MANIFEST="$ROOT_DIR/nostra/extraction/Cargo.toml"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/.cache/cargo-target/root-workspace}"
export CARGO_TARGET_DIR
CLI_BIN="$CARGO_TARGET_DIR/debug/nostra-contribution-cli"
MODE="validate-only"

if [[ "${1:-}" == "--materialize" ]]; then
  MODE="materialize"
elif [[ "${1:-}" == "--validate-only" || -z "${1:-}" ]]; then
  MODE="validate-only"
else
  echo "FAIL: unsupported mode '$1' (expected --validate-only or --materialize)" >&2
  exit 2
fi

read_hash() {
  node -e "const fs=require('fs');const p='$ROOT_DIR/research/000-contribution-graph/contribution_graph.json';const g=JSON.parse(fs.readFileSync(p,'utf8'));process.stdout.write(g.graph_root_hash);"
}

echo "[contribution-graph] building CLI binary..."
cargo build --offline --manifest-path "$CLI_MANIFEST" --bin nostra-contribution-cli >/dev/null

echo "[contribution-graph] running CLI validate..."
"$CLI_BIN" validate --root "$ROOT_DIR" >/dev/null

if [[ "$MODE" == "validate-only" ]]; then
  echo "[contribution-graph] PASS (validate-only)"
  exit 0
fi

echo "[contribution-graph] generating graph (run #1)..."
"$CLI_BIN" ingest --root "$ROOT_DIR" >/dev/null
HASH1="$(read_hash)"

echo "[contribution-graph] generating graph (run #2)..."
"$CLI_BIN" ingest --root "$ROOT_DIR" >/dev/null
HASH2="$(read_hash)"

if [[ "$HASH1" != "$HASH2" ]]; then
  echo "[contribution-graph] deterministic hash mismatch: $HASH1 vs $HASH2"
  exit 1
fi

echo "[contribution-graph] running doctor report..."
"$CLI_BIN" doctor --root "$ROOT_DIR" >/dev/null

echo "[contribution-graph] validating explain-path contract..."
"$CLI_BIN" explain-path --goal stable-cortex-domain --root "$ROOT_DIR" >/dev/null

echo "[contribution-graph] generating baseline simulation output..."
"$CLI_BIN" simulate --scenario "$ROOT_DIR/research/000-contribution-graph/scenarios/accelerate_118.yaml" --root "$ROOT_DIR" >/dev/null

echo "[contribution-graph] publishing default edition..."
"$CLI_BIN" publish-edition --version v0.2.0 --root "$ROOT_DIR" >/dev/null

for required in \
  "$ROOT_DIR/research/000-contribution-graph/dpub.json" \
  "$ROOT_DIR/research/000-contribution-graph/contribution_graph.json" \
  "$ROOT_DIR/research/000-contribution-graph/path_assessment.json" \
  "$ROOT_DIR/research/000-contribution-graph/doctor_report.json" \
  "$ROOT_DIR/research/000-contribution-graph/simulations/accelerate_118.json" \
  "$ROOT_DIR/research/000-contribution-graph/editions/v0.2.0/edition_manifest.json" \
  "$ROOT_DIR/research/000-contribution-graph/editions/v0.2.0/snapshot.json"; do
  if [[ ! -f "$required" ]]; then
    echo "[contribution-graph] missing required artifact: $required"
    exit 1
  fi
done

if [[ -f "$ROOT_DIR/research/000-contribution-graph/editions/v0.1.0/snapshot.json" ]]; then
  echo "[contribution-graph] validating edition diff contract..."
  "$CLI_BIN" diff-edition --from v0.1.0 --to v0.2.0 --root "$ROOT_DIR" >/dev/null
fi

echo "[contribution-graph] PASS root_hash=$HASH2"
