---
id: "125"
name: "system-integrity-quality"
title: "Requirements: SIQ Program Operationalization"
type: "requirements"
project: "nostra"
status: active
authors:
  - "X"
tags:
  - "siq"
  - "quality-gates"
  - "governance"
created: "2026-02-23"
updated: "2026-02-23"
---

# Requirements: SIQ Program Operationalization

## Functional Requirements

### FR-1: SIQ Governance/Execution Contract Coverage
- FR-1.1 SIQ MUST evaluate governance evidence presence for all integrity-critical initiatives.
- FR-1.2 SIQ MUST fail with `missing_gate_evidence` when required evidence is absent.
- FR-1.3 SIQ MUST fail with `governance_contract_mismatch` when waiver owner/expiry is missing.

### FR-2: SIQ Host Parity Contract Coverage
- FR-2.1 SIQ MUST evaluate dual-host parity evidence linkage for 118/123 controls.
- FR-2.2 SIQ MUST fail with `parity_contract_drift` when parity contracts drift.

### FR-3: SIQ Graph Projection Contract
- FR-3.1 SIQ MUST emit deterministic projection JSON to `logs/siq/graph_projection_latest.json`.
- FR-3.2 Projection MUST include entity classes: `Initiative`, `Rule`, `GateRun`, `Violation`, `Evidence`, `Waiver`.
- FR-3.3 Projection MUST include required edges:
  - `initiative_has_rule`
  - `rule_has_run`
  - `run_emits_violation`
  - `violation_backed_by_evidence`
  - `initiative_has_waiver`

### FR-4: CI Rollout Behavior
- FR-4.1 CI MUST run SIQ in `observe` mode by default.
- FR-4.2 CI MUST support promotion to `softgate` based on documented stability criteria.
- FR-4.3 CI MUST run deterministic projection replay check on identical input.

### FR-5: Read-Only Host Intake
- FR-5.1 Cortex gateway MUST expose read-only SIQ endpoints.
- FR-5.2 SIQ intake MUST be filesystem-canonical (`logs/siq/*`) in this phase.
- FR-5.3 No SIQ mutation APIs may be introduced.

### FR-6: 121 Milestone Gate Coupling
- FR-6.1 Initiative 121 milestone advancement MUST fail when SIQ governance/parity contracts fail.

## Non-Functional Requirements

### NFR-1: Determinism
- NFR-1.1 SIQ graph projection fingerprint MUST be stable across repeated runs with identical inputs.

### NFR-2: Backward Compatibility
- NFR-2.1 Initiative-graph outputs MUST remain backward-compatible.
- NFR-2.2 New SIQ overview fields MUST be optional.

### NFR-3: Auditability
- NFR-3.1 SIQ artifacts MUST include stable run IDs and verifiable evidence paths.
- NFR-3.2 Exceptions/skips MUST include owner and expiry.
