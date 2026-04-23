---
id: "114-cortex-realtime-transport-and-ops-governance"
name: "cortex-realtime-transport-and-ops-governance"
title: "Phase 6: Cortex Realtime Transport Integration and Operator Realtime UX Governance"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "realtime", "streaming", "governance", "ux"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex Realtime Collaboration Runtime + Ops Governance"
created: "2026-02-09"
updated: "2026-02-09"
---

# 114 Plan: Cortex Realtime Transport Integration and Operator Realtime UX Governance

## Objective
Bind Cortex collaboration to `nostra_streaming` transport lifecycle, expose production realtime operator UX surfaces, and harden governance identity/decision-proof controls for privileged collaboration actions.

## Locked Decisions
1. Realtime transport uses streaming canister lifecycle methods from `nostra/streaming/streaming.did`.
2. CRDT state remains authoritative and workflow-engine persistence remains primary durability source.
3. HITL/steward governance remains mandatory for structural promotion and privileged collaboration mutations.

## Workstreams

### A. Streaming Transport Binding
- Add `StreamingTransport` abstraction with canister and loopback implementations.
- Use canonical channel key `cortex:artifact:<artifact_id>`.
- Enforce persist-first then publish transport envelope.
- Queue replay on transport failure and expose degraded telemetry.

### B. Gateway Realtime Session Plane
- Extend gateway websocket path to collaboration event routing (`/ws/cortex/collab`).
- Support subscribe/unsubscribe/op-applied/presence/conflict/replay status semantics.
- Enforce deterministic idempotency/order with `op_id` and `(lamport, op_id)` ordering.

### C. Operator Realtime UX Surfaces
- Upgrade Artifacts desktop lane with presence roster, conflict banner, replay/degraded indicator, and checkpoint recovery controls.
- Keep keyboard and accessibility-safe controls for all realtime operations.
- Preserve capability contract alignment with web consumer.

### D. Governance + Identity Hardening
- Require governance envelope and decision proof for publish and force-resolve controls.
- Enforce actor identity consistency across HTTP headers, websocket identity, and ledgers.
- Reject privileged mutations when governance envelope/signature/role is missing.

### E. Contract + Declaration Alignment
- Add streaming declarations under `nostra/src/declarations/streaming/`.
- Extend shared UX fixture with realtime collaboration endpoint contract.
- Keep frontend Cortex contract fixture tests parity-locked.

### F. Reliability + Rollout
- Add realtime telemetry: convergence p95, replay backlog depth, duplicate drop rate, degraded duration.
- Roll out behind feature flag `cortex_collab_realtime`.
- Canary: internal operators -> steward validation -> broad operator enablement.

## Deliverables
- Streaming transport manager and realtime gateway APIs/ws route.
- Realtime operator surfaces in desktop Artifacts lane.
- Governance envelope enforcement on privileged actions.
- Phase 6 initiative artifacts and architecture docs updates.
- Shared fixture and parity tests with realtime endpoint coverage.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifacts_publish_requires_steward_role`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifact_collab_force_resolve_requires_steward_and_records_governance`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml shared_contract_fixture_declares_phase6_realtime_endpoints`
- `cargo test --manifest-path nostra/frontend/Cargo.toml shared_contract_fixture_declares_phase6_realtime_collaboration_endpoints`

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
