---
id: "125"
name: "system-integrity-quality"
title: "System Integrity + Quality (SIQ) Program Operationalization"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authority_mode: recommendation_only
execution_plane: "nostra-cortex"
authors:
  - "X"
tags:
  - "siq"
  - "governance"
  - "ci"
  - "quality"
  - "initiative-graph"
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Portfolio Integrity"
created: "2026-02-23"
updated: "2026-02-24"
---

# Initiative 125: SIQ Program Operationalization

## Objective
Operationalize SIQ from local contract checks into portfolio governance, CI enforcement (`observe -> softgate`), and read-only host intake for Cortex and initiative-graph consumers.

## Scope
1. Integrity set coverage and closure for `097/099/101/103/105/118/121/123`.
2. CI enforcement staged from observe to softgate without introducing SIQ mutation APIs.
3. Read-only SIQ gateway/service APIs in Cortex Desktop.
4. Optional initiative-graph SIQ intake with deterministic ordering and backward-compatible fields.
5. Residue/drift prevention: template residue, gate-surface script-reference integrity, and weekly drift checks.

## Out of Scope
1. SIQ governance mutation APIs.
2. Full in-app SIQ governance product UX.
3. Hardgate promotion for P1 (deferred to explicit ADR).

## Cross-Initiative Closure Matrix
| Initiative | Coverage Rule Groups | Owner | Evidence Mode | Blocking Relationship |
|---|---|---|---|---|
| 097 | governance_execution, graph_projection | Systems Steward | required | feeds 121 |
| 099 | governance_execution, graph_projection | Systems Steward | required | feeds 121 |
| 101 | governance_execution, graph_projection | Systems Steward | required | feeds 121 |
| 103 | governance_execution, graph_projection | Systems Steward | required | feeds 121 |
| 105 | governance_execution, graph_projection | Systems Steward | required | feeds 121 |
| 118 | governance_execution, host_parity, graph_projection | Systems Steward | required | feeds 121/123 |
| 121 | governance_execution, host_parity, graph_projection | Systems Steward | required | milestone-blocked by SIQ |
| 123 | governance_execution, host_parity, graph_projection | Systems Steward | required | feeds 121 |

## Workstreams

### A. Governance and Portfolio Formalization
- Create 125 artifact set (`PLAN/REQUIREMENTS/DECISIONS/VERIFY/RESEARCH`).
- Register 125 in `RESEARCH_INITIATIVES_STATUS.md`.
- Extend AGENTS with SIQ command contract and canonical SIQ artifacts.

### B. 121 Cleanup + Consumer Gate Alignment
- Remove template residue from `research/121-cortex-memory-fs/` via archive-first.
- Move templates to canonical `research/templates/`.
- Keep `INTEGRITY_DEPENDENCIES.md` as blocking contract for 121.
- Add 121 milestone gate clause: SIQ governance/parity failure blocks advancement.

### C. CI Integration (Observe)
- Add SIQ observe job in `.github/workflows/test-suite.yml`.
- Run `bash scripts/run_siq_checks.sh --mode observe`.
- Upload `logs/siq/*` artifacts.
- Add SIQ artifact consistency script and deterministic replay fingerprint check.

### D. CI Promotion (Softgate)
- Softgate enablement criteria:
  1. 5 consecutive green observe runs on `main`.
  2. 0 unresolved P0 failures across those runs.
- When criteria pass, enforce `softgate` failure on `overall_verdict=not-ready`.
- Keep P1 advisory until explicit hardgate ADR.

### E. Read-Only Cortex Gateway Intake
- Add filesystem-backed SIQ readers and endpoints:
  - `GET /api/system/siq/coverage`
  - `GET /api/system/siq/dependency-closure`
  - `GET /api/system/siq/gates/latest`
  - `GET /api/system/siq/graph-projection`
  - `GET /api/system/siq/runs`
  - `GET /api/system/siq/runs/:run_id`
  - `GET /api/system/siq/health`
- Add `NOSTRA_SIQ_LOG_DIR` override (default: `/Users/xaoj/ICP/logs/siq`).
- Add typed SIQ service methods in Cortex Desktop.

### F. Initiative-Graph Intake Bridge
- Optionally consume SIQ projection artifact for overview metadata.
- Deterministic handling for SIQ ingest material:
  - stable sort by `id` and `edge_id`
  - deterministic null/optional handling
- Keep initiative-graph contract backward-compatible (optional SIQ fields only).

### G. Drift and Residue Prevention
- Weekly SIQ drift check in CI schedule.
- Add template-residue detector for active initiative folders.
- Add gate-surface integrity check for missing referenced scripts.
- Track temporary exceptions in `VERIFY.md` with explicit expiry.

### H. CI Warning-Bypass Integrity Guard
- Add deterministic scanner command: `bash scripts/check_ci_warning_bypass.sh --strict`.
- Block warning-suppression bypass patterns in active workflows:
  - `RUSTFLAGS`/`RUSTDOCFLAGS` with `-A warnings`
  - `cargo clippy -- -A warnings`
  - rust-related cargo commands with `-A warnings`
- Emit scanner artifact at `logs/alignment/ci_warning_bypass_latest.json`.
- Register alignment contract rule `ci_warning_bypass_contract` as P0 softgate.
- Keep exceptions canonical in `shared/standards/alignment_contract_exceptions.json` with owner + reason + expiry.

## Exit Criteria
1. 125 artifacts exist and are index-registered.
2. AGENTS includes SIQ command + artifact contract.
3. 121 template residue removed and archived.
4. CI SIQ observe is running with artifacts and consistency checks.
5. Softgate path is implemented and can enforce P0 failures after promotion criteria.
6. SIQ read-only gateway endpoints return valid payloads from filesystem artifacts.
7. Initiative-graph can consume SIQ metadata without contract regression.
8. No SIQ governance mutation endpoint introduced.
9. CI warning-bypass scanner is enforced in active workflows and registered as alignment contract P0 control.

## Alignment Addendum
1. Boundary: SIQ remains evidence/gate orchestration for Nostra-Cortex contracts, not a new governance authority plane.
2. Parity: host-facing SIQ intake stays read-only and schema-first to prevent mutation drift.
3. Determinism: SIQ artifacts and projection fingerprints remain reproducible and lineage-linked.
