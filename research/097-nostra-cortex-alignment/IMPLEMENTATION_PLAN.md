---
id: "097-nostra-cortex-alignment-implementation"
name: "nostra-cortex-alignment-implementation"
title: "Implementation Plan: Flow Graph + Cortex Workbench MVP"
type: "implementation-plan"
project: "nostra"
status: active
authors:
  - "User"
  - "Codex"
tags: [alignment, workbench, flow-graph]
created: "2026-02-04"
updated: "2026-02-05"
---

# Implementation Plan: Flow Graph + Cortex Workbench MVP

## Objectives
- Implement the minimal workflow subset for deterministic flow graph derivation (DEC-008).
- Include lineage edges in the graph (DEC-009) and persist layout as Nostra contributions (DEC-010).
- Deliver a Cortex Workbench MVP with Flow Graph, Traces, and Logs views.

## Implementation Strategy
This implementation strategy completes alignment by reconciling all initiatives against canonical standards and principles while finishing the Workbench/Flow Graph deliverables.

### Workstreams
- **Inventory & Resolution**: Build the cross-initiative resolution matrix and map every initiative to plan/spec status, principle coverage, and canonical references.
- **Contract & Type Alignment**: Keep `.did` contracts authoritative and re-sync bindings/types after changes.
- **Workbench Foundations**: Deliver flow graph derivation + layout persistence + UI wiring.
- **Snapshot Contracts**: Ensure 080/057/036 adopt the `SnapshotManifest` + `DocsBundle` contract.
- **Verification & Closeout**: Confirm alignment artifacts, test coverage, and status truth.

### Governance Gates
- Any rename/merge/archive/scope change is recommendation-only until explicitly approved.
- Every cross-initiative change logs a decision and updates the resolution matrix.

## Scope
- Covers Flow Graph derivation, layout persistence, and Workbench MVP UI.
- Excludes full DSL coverage, advanced analytics, and non-core step types.

## Deliverables
- Flow Graph derivation module with stable hashing and deterministic ordering.
- `FlowLayout` contribution contract + storage + cache invalidation.
- Layout history (retention cap) with commit metadata and handle/collapsed group persistence.
- Workbench MVP UI (Flow Graph, Traces, Logs) wired via `ic-agent`.
- Updated Candid contracts + Rust bindings + frontend types.
- Verification notes recorded in `VERIFICATION_SNAPSHOT.md`.
- `RESOLUTION_MATRIX.md` with cross-initiative alignment status and exceptions.

## Proposed File Locations (MVP)
- `nostra/backend/workflow_engine/src/flow_graph/` for derivation logic and lineage edge join.
- `nostra/backend/workflow_engine/candid/workflow_engine.did` (or a new `flow_graph.did`) for `get_flow_graph`, `get_flow_layout`, `set_flow_layout`.
- `nostra/backend/**/types.mo` for `FlowGraph`/`FlowLayout` types aligned to `.did` contracts.
- `nostra/src/declarations/**` and `nostra/frontend/src/types.rs` for Rust bindings and frontend types.
- `nostra/frontend/src/views/workbench/` for Flow Graph, Traces, Logs views.
- `nostra/frontend/src/services/flow_graph.rs` (or equivalent) for `ic-agent` calls.
- `shared/` for any new standard schemas that must be referenced by multiple canisters.

## Phases

### Phase A: Contracts + Types
- Define `FlowGraph`, `FlowNode`, `FlowEdge`, `FlowLayout`, and `FlowTraceRef` (minimal MVP shape).
- Add Candid interface(s) for flow graph queries and layout updates.
- Sync Motoko domain types and Rust bindings with the `.did` contracts.
- Define layout event payloads for cache invalidation (Event standard in `shared/`).

### Phase B: Flow Graph Derivation
- Implement deterministic graph derivation from workflow steps using the minimal subset.
- Add lineage edges by joining workflow step ownership with Nostra contribution lineage.
- Provide stable hash of graph definition for cache and UI diffing.
- Implement space-scoped access controls; no cross-space graph leakage.

### Phase C: Layout Persistence + Cache
- Store layout as a `FlowLayout` contribution in Nostra.
- Append layout history with a fixed retention cap and commit metadata.
- Add Cortex cache with invalidation on layout update events.
- Enforce versioning and optimistic concurrency (last-write-wins with revision checks).

### Phase D: Workbench MVP UI
- Flow Graph view with trace overlay, layout editing, and provenance display.
- Layout history panel with preview/apply controls and handle/collapsed group editors.
- Traces view with filterable runs, status, duration, and step-level breakdown.
- Logs view connected to Log Registry with space-aware filters.
- Ensure no direct DOM manipulation outside `dioxus::eval` bridges.

### Phase E: Verification + Documentation
- Add verification snapshot entries (doc updates + build checks as available).
- Document usage in Workbench spec and update any dependent research plans.

### Phase F: Cross-Initiative Resolution & Closeout
- Build and maintain `RESOLUTION_MATRIX.md` with plan/spec presence, status alignment, and cross-link checks.
- Resolve conflicts between initiative plans/specs and canonical standards (`docs/architecture/standards.md`, constitutions).
- Record exceptions in `DECISIONS.md` and keep unresolved items as recommendations only.
- Reconcile status indices and ensure `research/README.md` reflects `PLAN.md` truth.

## Acceptance Criteria
- Graph derivation is deterministic and stable across builds.
- Lineage edges are visible and distinguishable from emit/subscribe edges.
- Layout persists as a Nostra contribution and survives reloads.
- Workbench views load for a space with minimal subset workflows.
- All contracts synced with `.did` and bound in Rust + frontend types.
- `RESOLUTION_MATRIX.md` covers all initiatives listed in `research/RESEARCH_INITIATIVES_STATUS.md`.

## Risks & Mitigations
- Risk: Lineage join increases query complexity.
- Mitigation: Cache contribution lineage by space and recompute on updates only.
- Risk: Layout updates race with graph updates.
- Mitigation: Revision-based writes + cache invalidation events.
- Risk: UI performance for dense graphs.
- Mitigation: Layout caching + viewport culling + async data fetch.

## Verification (Planned)
- `python3 scripts/generate_compliance_matrix.py`
- `python3 scripts/check_alignment_addendum.py`
- `dfx build` (when code changes land)
- Manual: confirm Workbench views render with sample workflows and logs.
