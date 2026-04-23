---
id: "130"
name: "space-capability-graph-governance"
title: "Requirements: Space Capability Graph Governance"
type: "requirements"
project: "cortex"
status: active
created: "2026-03-03"
updated: "2026-03-03"
---

# Requirements

## Functional
1. The runtime must expose a platform-level capability catalog endpoint.
2. The runtime must expose per-space capability graph read endpoint.
3. The runtime must expose per-space capability graph write endpoint gated to steward role.
4. Space graph writes must require lineage reference metadata.
5. The runtime must compile per-space navigation plans from catalog + overlay + context.
6. Role filtering must occur before ranking/surfacing.

## Data Model
1. Platform catalog nodes must include surfacing/frequency enrichments.
2. Space overlays must support active/inactive state and local role/surfacing overrides.
3. Space registry records must optionally track graph URI/version/hash.

## Compatibility
1. Existing `PlatformCapabilityGraph` domain references must continue compiling via alias.
2. Existing `/api/system/capability-graph` payload and tests must remain valid.

## Determinism
1. Equivalent semantic input sets must yield identical `plan_hash`.
2. Reordered source arrays must not change plan ordering/hash.

## Governance
1. Structural space graph mutations are steward-gated.
2. Structural mutations require lineage reference in payload.

## Verification
1. Unit tests cover compatibility, determinism, role filtering, surfacing buckets, and steward gate behavior.
2. Runtime capability graph compatibility tests remain green.
