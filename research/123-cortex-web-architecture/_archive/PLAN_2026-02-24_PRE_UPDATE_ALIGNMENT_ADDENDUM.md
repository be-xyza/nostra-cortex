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
updated: "2026-02-22"
---

# Initiative 123: Cortex Web Architecture

## Objective
Deliver `cortex-web` as a first-class Cortex execution host, parity-aligned with `cortex-desktop`, while preserving host neutrality from Research 118.

## Scope
1. Canonical app root: `/Users/xaoj/ICP/cortex/apps/cortex-web`.
2. Runtime authority: host-neutral gateway API (`/api/system/*`, `/api/kg/spaces/:space_id/initiative-graph/*`).
3. Workbench parity: node/edge/hash/path/lens outputs match desktop for the same corpus and goal.
4. Governance parity: steward-gated mutations with approval envelope required in both hosts.

## Out of Scope
1. Replacing `cortex-desktop`.
2. Introducing a standalone DPub product.
3. Alternate graph logic in host UIs.

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

### Phase 3: Desktop UX Stabilization
- Remove nested left-panel scroll traps.
- Add sticky section headers in left/right panes.
- Surface build/mode identity chips from `/api/system/build`.

### Phase 4: Web Host Bringup
- Implement `cortex-web` React/Vite host and Workbench route.
- Consume shared gateway APIs and contracts.
- Enforce stewardship gating and parity checks.

## Exit Criteria
1. Desktop and web consume the same host-neutral API.
2. Same corpus yields same graph hash and recommended path in both hosts.
3. Steward-gated mutation policy enforced in both hosts.
4. Bootstrap/parity/contract checks pass on latest branch state.
