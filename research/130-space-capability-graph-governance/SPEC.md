---
id: "130"
name: "space-capability-graph-governance"
title: "Spec: Space Capability Graph Governance"
type: "spec"
project: "cortex"
status: active
created: "2026-03-03"
updated: "2026-03-03"
---

# Specification

## Domain Types
1. `PlatformCapabilityCatalog` is canonical global catalog (`nodes`, `edges`, version/hash metadata).
2. `PlatformCapabilityGraph` is compatibility alias to `PlatformCapabilityCatalog` for transition window.
3. `SpaceCapabilityGraph` stores space overlay metadata and node overrides.
4. `SpaceCapabilityNodeOverride` carries local activation and override controls.
5. `CompilationContext` + `CompiledNavigationPlan` + `CompiledSurfacingPlan` define deterministic compile outputs.

## Placement Semantics
1. `PrimaryCore` -> primary top nav.
2. `Secondary` -> grouped secondary navigation.
3. `ContextualDeep` -> contextual-only surfacing.
4. `Hidden` -> excluded from default nav surfacing.

## Frequency Weights
1. `Continuous` > `Daily` > `AdHoc` > `Rare`.
2. Frequency contributes ranking only after role gates are satisfied.

## API Endpoints
1. `GET /api/system/capability-catalog`
2. `GET /api/spaces/:space_id/capability-graph`
3. `PUT /api/spaces/:space_id/capability-graph` (steward role required)
4. `GET /api/spaces/:space_id/navigation-plan?actor_role=&intent=&density=`

## Steward Gate Contract
1. Non-steward PUT returns `403` with `STEWARD_ROLE_REQUIRED`.
2. Missing `lineage_ref` returns `400` with `LINEAGE_REF_REQUIRED`.

## Persistence Contract
1. Space graph persisted to `_spaces/<space_id>/capability_graph.json`.
2. Space registry stores graph URI/version/hash when available.
