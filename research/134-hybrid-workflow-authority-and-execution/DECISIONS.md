---
id: "134"
name: "hybrid-workflow-authority-and-execution"
title: "Decision Log: Hybrid Workflow Authority and Execution"
type: "decision"
project: "nostra"
status: draft
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
- **Decision**: `workflow_engine_canister_v1` is a supported execution adapter path only when explicitly requested. `local_durable_worker_v1` remains the default execution adapter until live parity is demonstrated.
- **Rationale**: The canister path is now implemented and statically verified, but live ICP CLI deployment and gateway-to-canister execution parity have not yet been completed.

## DEC-134-006: ICP CLI is the current live validation path
- **Decision**: Phase 5 live canister validation should use the current ICP CLI path (`icp`). Legacy `dfx` support remains compatibility-only.
- **Rationale**: The active runtime direction is ICP CLI-first, and operator-facing Initiative 134 readiness should not depend on legacy `dfx` behavior.

## DEC-134-007: Phase 4 is complete; Phase 5 remains gated
- **Decision**: Treat Phase 4 execution adapter hardening as complete for implementation, static verification, and main-tree promotion. Treat Phase 5 as open until a live canister deployment and cross-adapter parity test pass.
- **Rationale**: Main-tree checks pass for the implemented adapter path, but no live gateway-to-canister start/signal/snapshot/cancel cycle has been executed against a deployed canister.
