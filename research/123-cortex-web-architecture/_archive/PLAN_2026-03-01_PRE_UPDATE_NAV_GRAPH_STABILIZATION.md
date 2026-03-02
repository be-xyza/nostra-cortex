---
id: "123"
name: "cortex-web-architecture"
title: "Cortex Web Architecture"
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
  - "web"
  - "workbench"
  - "dual-host"
  - "a2ui"
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Execution Hosts"
created: "2026-02-22"
updated: "2026-03-01"
---

# Initiative 123: Cortex Web Architecture

## Objective
Deliver `cortex-web` as the canonical interactive Cortex execution shell, with `cortex-desktop` operating as a headless daemon/gateway runtime per `DEC-123-004`.

## Scope
1. Canonical app root: `/Users/xaoj/ICP/cortex/apps/cortex-web`.
2. Runtime authority: `cortex-eudaemon` gateway APIs remain source-of-truth (`/api/system/*`, `/api/kg/spaces/:space_id/initiative-graph/*`).
3. Web shell responsibility: all interactive Workbench UX (A2UI surfaces, route rendering, action execution) is delivered in `cortex-web`.
4. Desktop shell responsibility: daemon, worker, and local gateway execution surfaces continue without a primary interactive UI mandate.
5. Governance parity: steward-gated mutations with approval envelope remain mandatory across runtime boundaries.

## Out of Scope
1. Reinstating Dioxus desktop UI as a primary execution shell.
2. Introducing a standalone DPub product.
3. Alternate graph logic in host-specific UIs.

## Delivery Phases

### Phase 0: Portfolio and Standards Integrity
- Renumber legacy `119-cortex-web-architecture` to `123-cortex-web-architecture`.
- Resolve status index conflicts and include 120/121/122/123 rows.
- Restore `docs/architecture/standards.md` as canonical standards authority index.

### Phase 1: Runtime/Gate Integrity
- Ensure `nostra/frontend/Cargo.toml` exists and workspace loads.
- Re-run bootstrap and protocol gate scripts.

### Phase 2: Host-Neutral Runtime Surface
- Add `cortex/apps/cortex-gateway` binary as shared runtime API entrypoint.
- Add `cortex/libraries/cortex-workbench-contracts` for shared Workbench contracts.

### Phase 3: Runtime Gateway Stabilization
- Keep daemon/gateway route contracts deterministic and parity-tested.
- Maintain inventory/fixture synchronization for active runtime endpoints.
- Preserve strict parity gating for contract tests and replay cases.

### Phase 4: Web Shell Bringup and Stabilization
- Implement `cortex-web` React/Vite host and Workbench route.
- Consume shared gateway APIs and contracts.
- Enforce stewardship gating and parity checks from the web shell.

## Exit Criteria
1. `cortex-web` consumes runtime contracts from `cortex-eudaemon` without host-specific API forks.
2. Gateway parity inventory, fixtures, and replay checks remain synchronized with `server.rs`.
3. Steward-gated mutation policy is enforced for web-initiated runtime actions.
4. Contract, build, and parity checks pass on the latest branch state.

## Alignment Addendum
1. Boundary: Nostra defines authority contracts and governance semantics; Cortex hosts remain execution adapters.
2. Parity: Desktop and web are required to consume shared runtime contracts, never host-specific forks.
3. Determinism: parity outcomes and graph outputs must remain reproducible for the same input corpus.
