---
id: "130"
name: "space-capability-graph-governance"
title: "Verify: Space Capability Graph Governance"
type: "verify"
project: "cortex"
status: active
created: "2026-03-03"
updated: "2026-03-03"
---

# Verification Checklist

## Domain
1. `SpaceRecord` legacy JSON loads without new fields.
2. `SpaceRecord` round-trip includes capability graph linkage fields.
3. `compile_navigation_plan` hash remains stable across reordered inputs.
4. Inactive node override removes entry from compiled plan.
5. Viewer context excludes operator-only routes.
6. `ContextualDeep` appears in contextual surfacing, not primary core.

## Runtime
1. `/api/system/capability-catalog` returns schema/version/hash and nodes/edges.
2. `/api/spaces/:space_id/capability-graph` bootstraps default graph for new space.
3. Non-steward graph PUT is rejected.
4. Steward graph PUT is accepted and emits graph hash.
5. `/api/spaces/:space_id/navigation-plan` returns deterministic `planHash` for same input.
6. Existing `/api/system/capability-graph` compatibility tests remain green.

## Governance Checks
1. `bash scripts/check_agent_preflight_contract.sh`
2. `bash scripts/check_dynamic_config_contract.sh`
3. `bash scripts/check_antigravity_rule_policy.sh`
