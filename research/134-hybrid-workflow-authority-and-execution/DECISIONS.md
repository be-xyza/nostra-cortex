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
updated: "2026-03-11"
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
