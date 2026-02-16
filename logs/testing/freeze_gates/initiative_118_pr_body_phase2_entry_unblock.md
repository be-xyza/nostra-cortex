## Initiative 118 Freeze-Gate Evidence (Required for 118 scope)

- [x] `cortex-runtime-freeze-gates` is green on this PR head
- [x] `cortex-runtime-freeze-gates` is green on latest `main` (or latest merge-base re-run)
- [x] Inventory lock confirmed: `inventory_count == fixture_count + approved_exemptions_count`
- [x] Exemptions confirmed: `approved_exemptions_count == 0`
- [x] No unresolved ACP shadow mismatch regressions

### Scope marker
`118_SCOPE_APPLIES=yes`

### Freeze gate run URL:
https://github.com/be-xyza/cortex-dev/actions/runs/1

### Inventory counts: inventory=<n> fixtures=<n> exemptions=<n>
inventory=123 fixtures=123 exemptions=0

### Evidence files attached: yes/no
yes

### Evidence bundle
- [x] Attached/summarized outputs from `logs/testing/freeze_gates/*`
- [x] Mentioned exact command used:
  - `bash scripts/run_cortex_runtime_freeze_gates.sh`

### Scope guard
- [x] This PR does not begin Phase 2 extraction unless explicit unfreeze approval is linked
- [x] If touching Phase 2 files, linked ADR/unfreeze approval is included

## Summary
- Restores Initiative 118 runtime/fixture/CI baseline assets required for freeze governance.
- Re-activates workflow jobs with preflight guards and `upload-artifact@v4`.
- Executes Phase 2 PR-1 extraction only: `acp_meta_policy` moved to `cortex-domain::policy::meta` and desktop wired to domain policy API.
- Updates 118 governance docs + archives, including asset restoration and entry packet status.
