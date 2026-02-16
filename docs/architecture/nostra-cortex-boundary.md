# Nostra-Cortex Boundary Contract

## Purpose
This document defines the canonical boundary to prevent terminology and architecture drift.

## Canonical Definitions
- Nostra = platform authority (what exists)
- Cortex = execution runtime (how work runs)
- Nostra Cortex = product umbrella (external-facing only)
- `Nostra`: Platform authority layer. Defines canonical entities, contribution model, governance, schemas, and policy constraints. Nostra defines what exists.
- `Cortex`: Execution runtime layer. Executes workflows, agents, orchestration, and runtime services. Cortex defines how work runs.
- `Nostra Cortex`: Product umbrella label for external-facing references only.

## Authority Model
- Canonical truth for governance and platform state lives in Nostra protocol authority surfaces (canisters and contracts).
- Cortex may enforce and validate locally for UX/runtime behavior, but cannot override protocol authority.

## Naming Rules
- Platform crates and services: `nostra-*`
- Execution crates and services: `cortex-*`
- Avoid slash-separated Nostra and Cortex naming in canonical docs. Use explicit phrasing:
  - `Nostra platform`
  - `Cortex runtime`
  - `Nostra Cortex` (umbrella only)

## Documentation Rules
- Canonical docs must include boundary-consistent wording:
  - `/Users/xaoj/ICP/AGENTS.md`
  - `/Users/xaoj/ICP/nostra/spec.md`
  - `/Users/xaoj/ICP/research/README.md`
  - `/Users/xaoj/ICP/docs/reference/README.md`
- New architecture or protocol docs should reference this file when defining system layers.

## Enforcement
- CI terminology lint (`scripts/check_nostra_cortex_terminology.sh`) must pass for canonical docs.
