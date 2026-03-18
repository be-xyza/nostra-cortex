# Initiative 118 - Remote Governance Ledger

Date: 2026-02-17
Status: active

This ledger records remote CI evidence links and steward authorization references
required under ADR-017 for merge and operational closure evidence.

## Records

| Phase/Slice | PR URL / Authorization Ref | PR-head Freeze Gate | Latest Main Freeze Gate | Steward Record | Status |
|---|---|---|---|---|---|
| Phase 2 PR-1 (`acp_meta_policy`) | https://github.com/be-xyza/cortex-dev/pull/2 | https://github.com/be-xyza/cortex-dev/actions/runs/22048766714 | https://github.com/be-xyza/cortex-dev/actions/runs/22048828212 | Merged PR steward authorization | closed |
| Phase 2 PR-2..PR-7 (per-slice backlog) | Scope authorization recorded in evidence template (`tests/fixtures/pr_evidence/phase2_slice_template.md`) | https://github.com/be-xyza/cortex-dev/actions/runs/22048766714 | https://github.com/be-xyza/cortex-dev/actions/runs/22048828212 | Steward: Systems Steward, authorized_on: 2026-02-16 | recorded |
| Phase 3 completion evidence merge discipline | https://github.com/be-xyza/cortex-dev/pull/2 | https://github.com/be-xyza/cortex-dev/actions/runs/22048766714 | https://github.com/be-xyza/cortex-dev/actions/runs/22048828212 | ADR-017 stewardship retained | recorded |
| Phase 5 operational closure | https://github.com/be-xyza/cortex-dev/pull/2 | https://github.com/be-xyza/cortex-dev/actions/runs/22048766714 | https://github.com/be-xyza/cortex-dev/actions/runs/22048828212 | Steward merge authorization attached | closed |

## Notes

1. This ledger is the canonical cross-slice reference for remote governance attachments cited by:
   - `PHASE_2_COMPLETION_EVIDENCE_2026-02-16.md`
   - `PHASE_3_COMPLETION_EVIDENCE_2026-02-16.md`
   - `PHASE_5_OPERATIONAL_CLOSURE_EVIDENCE_2026-02-16.md`
   - `GSMS_PREREQ_GATE_2026-02-16.md`
2. If new PR slices are added, append rows here and update evidence artifacts with the new references.
