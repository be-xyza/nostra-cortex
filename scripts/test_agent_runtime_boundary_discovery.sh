#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT_DIR/scripts/check_agent_runtime_boundaries.py"
VALIDATOR="$ROOT_DIR/scripts/validate_agent_operating_model.py"
HARNESS="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_harness_registry.v1.json"
PROFILES="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_profiles.v1.json"
GATES="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_promotion_gates.v1.json"
RUN_EVIDENCE="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_run_evidence.v1.json"
ROADMAP="$ROOT_DIR/research/132-eudaemon-alpha-initiative/examples/agent_activation_roadmap.v1.json"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

write_profile() {
  local profile_dir="$1"
  local disabled="$2"
  local cli_tool="$3"
  mkdir -p "$profile_dir"
  cat >"$profile_dir/config.yaml" <<EOF
model:
  provider: openrouter
agent:
  disabled_toolsets:
$disabled
delegation:
  orchestrator_enabled: false
platform_toolsets:
  cli:
$cli_tool
EOF
  cat >"$profile_dir/SOUL.md" <<'EOF'
Codex CLI invocation is forbidden.
Terminal, shell, Python/code execution is forbidden.
Codex or the operator applies patches.
EOF
}

write_boundary() {
  local path="$1"
  local profiles_root="$2"
  local systemd_root="$3"
  local boundaries="$4"
  cat >"$path" <<EOF
{
  "schemaVersion": "1.0.0",
  "boundarySetId": "agent-runtime-boundaries:test",
  "authorityMode": "recommendation_only",
  "generatedAt": "2026-05-01T00:00:00Z",
  "discovery": {
    "hermesProfilesRoot": "$profiles_root",
    "projectHermesProfilePrefixes": ["hermes"],
    "ignoredHermesProfiles": [],
    "systemdRoot": "$systemd_root",
    "agentLikeSystemdServices": ["cortex-worker.service", "cortex-workrouter.service", "cortex-newagent.service"],
    "ignoredSystemdServices": ["cortex-gateway.service", "cortex-icp-network.service"]
  },
  "boundaries": [
    $boundaries
  ]
}
EOF
}

hermes_boundary='{
      "agentId": "hermesbounded",
      "runtimeKind": "hermes_profile",
      "defaultProfileDir": "'"$tmpdir"'/profiles/hermesbounded",
      "configFile": "config.yaml",
      "charterFile": "SOUL.md",
      "requiredDisabledToolsets": ["terminal", "code_execution"],
      "forbiddenCliToolsets": ["terminal", "code_execution"],
      "forbiddenModelProviders": ["codex", "openai-codex"],
      "requiredConfigAssertions": [{"path": "delegation.orchestrator_enabled", "equals": false}],
      "requiredCharterPhrases": ["codex cli invocation", "terminal, shell, python/code execution", "codex or the operator applies patches"]
    }'

service_boundary='{
      "agentId": "workrouter",
      "runtimeKind": "systemd_service",
      "servicePath": "'"$tmpdir"'/systemd/cortex-workrouter.service",
      "requiredContains": ["Environment=WORK_ROUTER_MAX_DISPATCH_LEVEL=D1"],
      "forbiddenContains": ["git push"]
    }'

mkdir -p "$tmpdir/profiles" "$tmpdir/systemd"
write_profile "$tmpdir/profiles/hermesbounded" "  - terminal"$'\n'"  - code_execution" "  - file"
cat >"$tmpdir/systemd/cortex-workrouter.service" <<'EOF'
[Service]
Environment=WORK_ROUTER_MAX_DISPATCH_LEVEL=D1
ExecStart=/srv/repo/scripts/work_router_service_stub.sh
EOF

write_boundary "$tmpdir/pass.json" "$tmpdir/profiles" "$tmpdir/systemd" "$hermes_boundary,$service_boundary"
python3 "$CHECK" "$tmpdir/pass.json" >/tmp/agent_runtime_boundary_positive.out

mkdir -p "$tmpdir/profiles/hermesrogue"
write_profile "$tmpdir/profiles/hermesrogue" "  - terminal"$'\n'"  - code_execution" "  - file"
if python3 "$CHECK" "$tmpdir/pass.json" >/tmp/agent_runtime_boundary_unregistered_profile.out 2>/tmp/agent_runtime_boundary_unregistered_profile.err; then
  echo "FAIL: unregistered Hermes profile unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "discovered project Hermes profile without runtime boundary" /tmp/agent_runtime_boundary_unregistered_profile.err; then
  echo "FAIL: unregistered Hermes profile failed for unexpected reason" >&2
  cat /tmp/agent_runtime_boundary_unregistered_profile.err >&2 || true
  exit 1
fi
rm -rf "$tmpdir/profiles/hermesrogue"

cat >"$tmpdir/systemd/cortex-newagent.service" <<'EOF'
[Service]
ExecStart=/srv/repo/scripts/new_agent.sh
EOF
if python3 "$CHECK" "$tmpdir/pass.json" >/tmp/agent_runtime_boundary_unregistered_service.out 2>/tmp/agent_runtime_boundary_unregistered_service.err; then
  echo "FAIL: unregistered Cortex service unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "discovered agent-like systemd service without runtime boundary" /tmp/agent_runtime_boundary_unregistered_service.err; then
  echo "FAIL: unregistered Cortex service failed for unexpected reason" >&2
  cat /tmp/agent_runtime_boundary_unregistered_service.err >&2 || true
  exit 1
fi
rm -f "$tmpdir/systemd/cortex-newagent.service"

write_profile "$tmpdir/profiles/hermesbounded" "  - terminal" "  - code_execution"
if python3 "$CHECK" "$tmpdir/pass.json" >/tmp/agent_runtime_boundary_forbidden_tool.out 2>/tmp/agent_runtime_boundary_forbidden_tool.err; then
  echo "FAIL: forbidden Hermes toolset unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "must disable toolsets" /tmp/agent_runtime_boundary_forbidden_tool.err; then
  echo "FAIL: forbidden Hermes toolset failed for unexpected reason" >&2
  cat /tmp/agent_runtime_boundary_forbidden_tool.err >&2 || true
  exit 1
fi

cat >"$tmpdir/profile_missing_runtime_boundary.json" <<'EOF'
{
  "schemaVersion": "1.0.0",
  "registryId": "agent-profile-registry:invalid:missing-runtime-boundary",
  "authorityMode": "recommendation_only",
  "generatedAt": "2026-05-01T00:00:00Z",
  "profiles": [
    {
      "agentId": "invalid-active-agent",
      "harnessId": "native-workrouter",
      "spaceBinding": { "mode": "operator_only" },
      "purpose": "Invalid active profile missing runtime boundary.",
      "authority": {
        "maxDispatchLevel": "D1",
        "sourceMutationAllowed": false,
        "runtimeMutationAllowed": false
      },
      "allowedOutputs": ["status_digest"],
      "forbiddenActions": ["source_mutation", "runtime_mutation", "commit", "push", "pull_request", "deploy", "canister_call", "graph_mutation", "provider_topology_exposure", "authority_escalation"],
      "transportBindings": ["cli"],
      "memoryPolicy": "none",
      "healthStatus": "healthy",
      "lifecycleState": "active"
    }
  ]
}
EOF

if python3 "$VALIDATOR" "$HARNESS" "$tmpdir/profile_missing_runtime_boundary.json" "$GATES" "$RUN_EVIDENCE" "$ROADMAP" >/tmp/agent_profile_missing_runtime_boundary.out 2>/tmp/agent_profile_missing_runtime_boundary.err; then
  echo "FAIL: missing runtimeBoundary profile unexpectedly passed" >&2
  exit 1
fi
if ! grep -q "runtimeBoundary" /tmp/agent_profile_missing_runtime_boundary.err; then
  echo "FAIL: missing runtimeBoundary profile failed for unexpected reason" >&2
  cat /tmp/agent_profile_missing_runtime_boundary.err >&2 || true
  exit 1
fi

echo "PASS: agent runtime boundary discovery negative checks"
