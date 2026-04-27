---
id: "134"
name: "hybrid-workflow-authority-and-execution"
title: "Decision Log: Hybrid Workflow Authority and Execution"
type: "decision"
project: "nostra"
status: archived
authors:
  - "Codex"
created: "2026-03-11"
updated: "2026-04-27"
---

# Decision Log: Hybrid Workflow Authority and Execution

## DEC-134-001: 013 is a historical pattern source, not default canon
- **Decision**: Treat initiative 013 as historical architecture input whose decisions must be explicitly preserved, modified, superseded, or deferred.
- **Rationale**: Current runtime and canister code do not support assuming 013 remains the most optimal or most current architecture.

## DEC-134-002: Canonical workflow source is an internal definition artifact
- **Decision**: `WorkflowDefinitionV1` is the source of truth; Serverless Workflow is projection/interchange only.
- **Rationale**: Preserves deterministic compilation while avoiding direct lock-in to an external DSL as the internal IR.

## DEC-134-003: A2UI is projection, not definition primitive
- **Decision**: Human and evaluation nodes emit typed A2UI projection artifacts; canonical definitions store checkpoint contracts, not renderer payloads.
- **Rationale**: Keeps workflow semantics distinct from UI rendering concerns and better aligns with the Nostra/Cortex boundary.

## DEC-134-004: Hybrid authority/runtime split
- **Decision**: Nostra owns intents, definitions, governance, lineage, and outcomes; Cortex owns execution adapters, checkpoints, runtime trace, and operator UX.
- **Rationale**: Matches the platform/runtime boundary and avoids making a single canister or worker implementation architecturally primary by assumption.

## DEC-134-005: Canister execution adapter is explicit, not default
- **Decision**: `workflow_engine_canister_v1` is a supported execution adapter path only when explicitly requested. `local_durable_worker_v1` remains the default execution adapter.
- **Rationale**: The canister path is now implemented, statically verified, and live-validated (see DEC-134-008). The default remains local for operational simplicity; canister selection is explicit by design, not by uncertainty.
- **Consequences**:
  - Gateway users must explicitly set `adapter: "workflow_engine_canister_v1"` in the execution binding.
  - The default `local_durable_worker_v1` requires no network or canister deployment.

## DEC-134-006: ICP CLI is the current live validation path
- **Decision**: Phase 5 live canister validation should use the current ICP CLI path (`icp`). Legacy `dfx` support remains compatibility-only.
- **Rationale**: The active runtime direction is ICP CLI-first, and operator-facing Initiative 134 readiness should not depend on legacy `dfx` behavior.

## DEC-134-007: Phase 4 is complete; Phase 5 requires live validation
- **Decision**: Treat Phase 4 execution adapter hardening as complete for implementation, static verification, and main-tree promotion. Phase 5 requires live canister deployment and cross-adapter parity evidence.
- **Rationale**: Static checks alone are insufficient for a canister-backed adapter. Live gateway-to-canister start/signal/snapshot/cancel cycles must be demonstrated before parity can be claimed.

## DEC-134-008: Phase 5 cross-adapter parity is demonstrated
- **Decision**: Treat Phase 5 cross-adapter parity as complete. The `workflow_engine_canister_v1` adapter has been deployed to a live canister, exercised through the Cortex gateway, and shown to reach semantic parity with `local_durable_worker_v1` for supported workflow motifs.
- **Rationale**:
  - Live canister deployment via ICP CLI succeeded on 2026-04-26.
  - Direct canister lifecycle (compile, start, signal, snapshot, cancel) was exercised through `icp canister call`.
  - Gateway-mediated canister execution succeeded through `/api/cortex/workflow-instances` with adapter selection, registry persistence, and full read/write endpoint coverage.
  - Local-vs-canister parity was demonstrated: identical supported definitions reach matching semantic state (workflow_started, checkpoint_created, signal_received, resolved checkpoint, completed instance, completed outcome).
  - Unsupported node kinds (parallel, evaluation_gate) fail fast on both adapters.
  - The canister path is no longer optional until build/runtime validation is real. It is now a validated, selectable execution adapter.
- **Consequences**:
  - Future workflow definitions may explicitly select `workflow_engine_canister_v1` as the execution adapter.
  - `local_durable_worker_v1` remains the default adapter unless explicitly overridden.
  - The decision to keep canister as non-default is now operational preference, not architectural uncertainty.
  - Initiative 134 exit criteria are satisfied.

## DEC-134-009: Initiative 134 is complete and archived
- **Decision**: Mark Initiative 134 complete and archived as of 2026-04-27.
- **Rationale**:
  - The merged implementation in PR #52 establishes the canister-backed execution path, adapter-aware gateway/workbench routing, local parity remediation, typed frontend workflow-definition contract, and ICP CLI-first operator path.
  - Live validation demonstrated direct canister lifecycle calls and gateway-mediated canister execution, including signal, snapshot, trace, checkpoint, outcome, cancel, and fail-fast unsupported-node behavior.
  - CI passed before merge, including Static Analysis, Rust Unit & Integration Tests, Cortex Runtime Freeze Gates, ACP Gateway Integration, Motoko Canister Tests, Test Catalog Consistency, SIQ Observe, SIQ Softgate Promotion, Initiative 118 Evidence Gate, Offline Simulations Playwright, Vercel, and canister-ID checks.
  - Remaining work is future hardening or expansion, not a blocker to the Initiative 134 scope.
- **Consequences**:
  - Initiative 134 is no longer an active implementation initiative.
  - Future adapter expansion, default-adapter rollout, or broader canister node-kind support must be opened under separate governed work.
  - Hermes may treat 134 as a validated substrate for advisory review and cross-initiative observation.
