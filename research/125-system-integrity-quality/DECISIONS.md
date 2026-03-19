---
id: "125"
name: "system-integrity-quality"
title: "Decisions: SIQ Program Operationalization"
type: "decision"
project: "nostra"
status: active
authors:
  - "X"
tags:
  - "siq"
  - "adr"
created: "2026-02-23"
updated: "2026-03-10"
---

# Decision Log: SIQ Program Operationalization

## ADR-125-001: SIQ Is a Continuous Program, Not a Point-in-Time Catalog
**Date**: 2026-02-23
**Status**: Accepted

**Decision**: SIQ is formalized as a continuous portfolio-quality system with artifacts in `logs/siq/*` and governance in initiative 125.

**Rationale**:
- Quality/integrity drift is continuous.
- Portfolio-level controls span initiatives and hosts.
- A catalog artifact is necessary but insufficient without recurring gates.

**Implications**:
- SIQ becomes a standing control plane with recurring CI checks.
- Initiative 125 owns policy, rollout, and exception governance.

## ADR-125-002: Contract-First, Read-Only Host Intake
**Date**: 2026-02-23
**Status**: Accepted

**Decision**: Implement read-only SIQ host intake endpoints now; defer mutation/governance product features.

**Rationale**:
- Stabilizes interfaces before UX productization.
- Keeps governance authority in existing contracts and evidence paths.
- Minimizes blast radius.

## ADR-125-003: Observe-to-Softgate Promotion Policy
**Date**: 2026-02-23
**Status**: Accepted

**Decision**:
1. Start SIQ CI in `observe`.
2. Promote to enforced `softgate` only after:
   - 5 consecutive green observe runs on `main`.
   - 0 unresolved P0 failures across those runs.
3. Keep P1 advisory until hardgate ADR.

## ADR-125-004: Waiver Governance Contract
**Date**: 2026-02-23
**Status**: Accepted

**Decision**: Every SIQ waiver must include owner + expiry and follow review cadence.

**Policy**:
- Required fields: `waiver_id`, `initiative_id`, `owner`, `expires_at`.
- Missing owner/expiry is a governance contract failure.
- Review cadence: weekly during observe/softgate rollout.

## ADR-125-005: Contribution-Graph SIQ Bridge Must Stay Backward-Compatible
**Date**: 2026-02-23
**Status**: Accepted

**Decision**: SIQ fields exposed via contribution-graph overview are optional metadata only. Any retained `initiative-graph` alias is legacy compatibility and must not be treated as navigation authority.

**Fields**:
- `siq_run_id`
- `siq_graph_fingerprint`
- `siq_overall_verdict`

## ADR-125-006: CI Warning Suppression Bypass Is Prohibited in Active Workflows
**Date**: 2026-02-24
**Status**: Accepted

**Decision**: Active CI workflows under `.github/workflows/` must not bypass warning policy using suppression flags (`-A warnings`, `-Awarnings`) in Rust command paths.

**Control**:
- Scanner command: `bash scripts/check_ci_warning_bypass.sh --strict`
- Report artifact: `logs/alignment/ci_warning_bypass_latest.json`
- Alignment rule: `ci_warning_bypass_contract` (P0, `softgate`)

**Allowed Exception Path**:
- Canonical registry only: `shared/standards/alignment_contract_exceptions.json`
- Mandatory fields: `id`, `owner`, `reason`, `expires_at`, `enabled`
- Optional selectors: `workflow_path`, `line`, `pattern`
- Maximum exception TTL: 30 days

**Rationale**:
- Warning budgets and strict profiles are trust controls; suppression bypass invalidates those guarantees.
- Blocking enforcement in both PR CI and weekly drift closes regression vectors.
