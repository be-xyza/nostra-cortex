#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATOR="$ROOT_DIR/scripts/validate_agent_operating_model.py"
HARNESS="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_harness_registry.v1.json"
PROFILES="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_profiles.v1.json"
GATES="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_promotion_gates.v1.json"
RUN_EVIDENCE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_run_evidence.v1.json"
ACTIVATION_ROADMAP="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_activation_roadmap.v1.json"
HERMES_GATEWAY_ADAPTER="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/dispatch_transport_adapter_hermes_gateway_candidate.v1.json"
TRANSPORT_ADAPTER_SCHEMA="$ROOT_DIR/research/132-eudaemon-alpha-initiative/schemas/DispatchTransportAdapterV1.schema.json"
VPS_WORKROUTER_BOOTSTRAP_CHECK="$ROOT_DIR/scripts/check_vps_workrouter_bootstrap.sh"
HERMESCORTEXDEV_BOUNDARY_CHECK="$ROOT_DIR/scripts/check_hermescortexdev_boundary.sh"
AGENT_RUNTIME_BOUNDARY_CHECK="$ROOT_DIR/scripts/check_agent_runtime_boundaries.py"
AGENT_RUNTIME_BOUNDARY_DISCOVERY_TEST="$ROOT_DIR/scripts/test_agent_runtime_boundary_discovery.sh"
INVALID_HARNESS="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/agent_harness_active_without_evidence_rejected.v1.json"
INVALID_PROFILE_MISSING_AUTH="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/agent_profile_missing_authority_ceiling_rejected.v1.json"
INVALID_PROFILE_MUTATION="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/invalid/agent_profile_mutation_d0_d1_rejected.v1.json"

python3 "$VALIDATOR" "$HARNESS" "$PROFILES" "$GATES" "$RUN_EVIDENCE" "$ACTIVATION_ROADMAP"
"$VPS_WORKROUTER_BOOTSTRAP_CHECK" >/tmp/vps_workrouter_bootstrap_check.out
"$HERMESCORTEXDEV_BOUNDARY_CHECK" >/tmp/hermescortexdev_boundary_check.out
python3 "$AGENT_RUNTIME_BOUNDARY_CHECK" >/tmp/agent_runtime_boundary_check.out
"$AGENT_RUNTIME_BOUNDARY_DISCOVERY_TEST" >/tmp/agent_runtime_boundary_discovery_test.out
python3 - "$TRANSPORT_ADAPTER_SCHEMA" "$HERMES_GATEWAY_ADAPTER" <<'PY'
import json
import sys
import jsonschema
schema_path, fixture_path = sys.argv[1], sys.argv[2]
with open(schema_path, encoding="utf-8") as handle:
    schema = json.load(handle)
with open(fixture_path, encoding="utf-8") as handle:
    fixture = json.load(handle)
jsonschema.validate(fixture, schema)
if fixture["transportKind"] != "hermes_gateway":
    raise SystemExit("Hermes Gateway adapter fixture must use transportKind=hermes_gateway")
if fixture["authority"]["maxLevel"] not in ("D0", "D1") or fixture["authority"]["mutationAllowed"]:
    raise SystemExit("Hermes Gateway adapter must remain D0-D1 non-mutating in v1")
PY

if python3 "$VALIDATOR" "$INVALID_HARNESS" "$PROFILES" "$GATES" >/tmp/agent_model_invalid_harness.out 2>/tmp/agent_model_invalid_harness.err; then
  echo "FAIL: active harness without evidence unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "requires evaluationEvidenceRefs" /tmp/agent_model_invalid_harness.err; then
  echo "FAIL: active harness fixture failed for unexpected reason" >&2
  cat /tmp/agent_model_invalid_harness.err >&2 || true
  exit 1
fi

if python3 "$VALIDATOR" "$HARNESS" "$INVALID_PROFILE_MISSING_AUTH" "$GATES" >/tmp/agent_model_invalid_profile_missing.out 2>/tmp/agent_model_invalid_profile_missing.err; then
  echo "FAIL: profile missing authority ceiling unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "maxDispatchLevel" /tmp/agent_model_invalid_profile_missing.err; then
  echo "FAIL: missing authority ceiling fixture failed for unexpected reason" >&2
  cat /tmp/agent_model_invalid_profile_missing.err >&2 || true
  exit 1
fi

if python3 "$VALIDATOR" "$HARNESS" "$INVALID_PROFILE_MUTATION" "$GATES" >/tmp/agent_model_invalid_profile_mutation.out 2>/tmp/agent_model_invalid_profile_mutation.err; then
  echo "FAIL: D1 mutating profile unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "cannot allow source or runtime mutation" /tmp/agent_model_invalid_profile_mutation.err; then
  echo "FAIL: D1 mutating profile fixture failed for unexpected reason" >&2
  cat /tmp/agent_model_invalid_profile_mutation.err >&2 || true
  exit 1
fi

echo "PASS: agent operating model checks"
