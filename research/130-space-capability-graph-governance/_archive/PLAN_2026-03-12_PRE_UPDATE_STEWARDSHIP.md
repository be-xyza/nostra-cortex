---
id: "130"
name: "space-capability-graph-governance"
title: "Space Capability Graph Governance"
type: "plan"
project: "cortex"
status: active
portfolio_role: anchor
authority_mode: recommendation_only
execution_plane: "cortex"
authors:
  - "X"
tags:
  - "cortex"
  - "ux"
  - "navigation"
  - "governance"
  - "spaces"
created: "2026-03-03"
updated: "2026-03-03"
---

# Initiative 130: Space Capability Graph Governance

## Objective
Introduce a two-layer capability model where the platform maintains a canonical global capability catalog and each space maintains a steward-governed capability overlay used to compile deterministic navigation/surfacing plans.

## Scope
1. Add domain contracts for `PlatformCapabilityCatalog`, `SpaceCapabilityGraph`, and deterministic compilation outputs.
2. Extend space registry records with capability graph linkage metadata.
3. Add additive gateway APIs for catalog, per-space graph read/write, and per-space compiled navigation plans.
4. Preserve existing `/api/system/capability-graph` behavior for backwards compatibility.
5. Add steward gating and lineage requirements for structural space graph mutations.

## Out of Scope
1. Moving capability graph authority to canister-backed storage in this phase.
2. Removing legacy `/api/system/capability-graph` payload contracts.
3. Host-specific divergent compilers.

## Delivery Phases

### Phase A: Domain Contracts
- Add catalog and overlay types.
- Add surfacing/frequency/entity/placement enrichments.
- Keep compatibility alias for `PlatformCapabilityGraph`.

### Phase B: Space Registry Linkage
- Extend `SpaceRecord` with optional graph URI/version/hash fields.
- Preserve old JSON compatibility.

### Phase C: Deterministic Compiler
- Compile catalog + overlay + context into deterministic `CompiledNavigationPlan`.
- Enforce role gating before placement ranking.

### Phase D: Runtime APIs
- `GET /api/system/capability-catalog`
- `GET /api/spaces/:space_id/capability-graph`
- `PUT /api/spaces/:space_id/capability-graph` (steward only)
- `GET /api/spaces/:space_id/navigation-plan`

### Phase E: Web Consumption
- Add web contract interfaces for new payloads.
- Read navigation plan in shell layout while retaining layout fallback.

### Phase F: Governance and Evidence
- Log architectural decision in root decision log.
- Link Initiative 130 from Initiative 123 parity hardening scope.

## Exit Criteria
1. Space capability graph can be created/read/updated with steward gating.
2. Navigation plan hash is deterministic for equivalent inputs.
3. Existing system capability graph API remains backward compatible.
4. Runtime and domain tests pass for new contracts and behavior.
