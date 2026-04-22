#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ONTOLOGY_JSONLD_GENERATOR="$ROOT_DIR/scripts/generate_core_ontology_jsonld.py"
python3 "$ROOT_DIR/scripts/validate_knowledge_graph_contracts.py" --root "$ROOT_DIR"

python3 "$ONTOLOGY_JSONLD_GENERATOR" --check
