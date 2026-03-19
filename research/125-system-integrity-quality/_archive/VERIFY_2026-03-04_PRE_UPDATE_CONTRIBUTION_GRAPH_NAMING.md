# Verify: SIQ Program Operationalization (Initiative 125)

## Verification Checklist

### A. Portfolio + Governance
- [ ] `research/125-system-integrity-quality/*` artifacts exist.
- [ ] `research/RESEARCH_INITIATIVES_STATUS.md` includes 125.
- [ ] `AGENTS.md` includes SIQ command contract and canonical SIQ artifact paths.

### B. 121 Alignment
- [ ] Template residue removed from `research/121-cortex-memory-fs/` active root.
- [ ] `research/templates/` is canonical template location.
- [ ] 121 plan includes SIQ milestone advancement block clause.

### C. CI + Determinism
- [ ] SIQ observe job runs and uploads `logs/siq/*` artifacts.
- [ ] SIQ consistency check validates summary/run/projection linkage.
- [ ] Deterministic replay fingerprint check passes for same input snapshot.

### D. Host Intake (Read-Only)
- [ ] SIQ read-only gateway endpoints return valid payloads.
- [ ] `NOSTRA_SIQ_LOG_DIR` override works.
- [ ] `siq_service` typed fetch methods compile.

### E. Initiative-Graph Bridge
- [ ] Overview payload includes optional SIQ metadata fields.
- [ ] Existing initiative-graph payload contracts remain valid.

### F. No Mutation API Expansion
- [ ] No SIQ governance mutation endpoint introduced.

### G. CI Warning-Bypass Integrity Guard
- [ ] `scripts/check_ci_warning_bypass.sh --strict` exists and fails on suppression patterns in active workflows.
- [ ] `logs/alignment/ci_warning_bypass_latest.json` is generated with `status`, `violations[]`, and `scanned_files[]`.
- [ ] `test-suite.yml` runs CI warning-bypass check in blocking mode.
- [ ] `siq-weekly-drift.yml` runs CI warning-bypass check in blocking mode before SIQ observe checks.
- [ ] `shared/standards/alignment_contracts.toml` includes `ci_warning_bypass_contract` rule.

## Temporary Exceptions (Tracked with Expiry)

Canonical exception registry: `shared/standards/alignment_contract_exceptions.json`

CI warning-bypass policy exceptions:
- Scope key: `ci_warning_bypass_contract`
- Required fields: `id`, `owner`, `reason`, `expires_at`, `enabled`
- Optional selectors: `workflow_path`, `line`, `pattern`
- Expiry discipline: exception TTL must not exceed 30 days

| Ref | Class | Owner | Expires | Status | Notes |
|---|---|---|---|---|---|
| `scripts/check_docs_structure.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_reference_paths.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_reference_metadata_v2.py` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_reference_taxonomy_integrity.py` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_no_panic_paths.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_phase7_closeout_tasks.py` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_strict_warning_profile.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_cortex_ux_fixture_drift.py` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |
| `scripts/check_test_catalog_consistency.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by AGENTS + CI; keep tracked until restored. |
| `scripts/test_catalog_refresh.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by CI; keep tracked until restored. |
| `nostra/scripts/verify_compliance.sh` | missing_referenced_script | Systems Steward | 2026-03-31 | open | Referenced by existing CI lint step. |

## Commands

```bash
bash scripts/run_siq_checks.sh --mode observe
bash scripts/check_siq_artifact_consistency.sh --mode observe --check-deterministic
bash scripts/check_template_residue_active_initiatives.py --strict
bash scripts/check_gate_surface_integrity.sh --strict
bash scripts/check_ci_warning_bypass.sh --strict
bash scripts/check_alignment_contract_targets.sh
```
