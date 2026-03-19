# Initiative 118 — Phase 2 Completion Evidence (Local)

Date: 2026-02-16

## Scope Closed

Phase 2 policy/collaboration extraction slices are implemented in workspace:

1. `artifact_collab_crdt.rs` extracted to `cortex/libraries/cortex-domain/src/collaboration/crdt.rs`
2. ACP operation contract split to `cortex/libraries/cortex-runtime/src/policy/adapter.rs`
3. Session store extraction to `cortex/libraries/cortex-runtime/src/policy/sessions.rs`
4. Permission ledger extraction to `cortex/libraries/cortex-runtime/src/policy/permissions.rs`
5. Metrics extraction to `cortex/libraries/cortex-runtime/src/policy/metrics.rs`
6. Protocol split:
   - JSON-RPC/domain types in `cortex/libraries/cortex-domain/src/policy/types.rs`
   - Runtime orchestration/dispatch in `cortex/libraries/cortex-runtime/src/policy/protocol.rs`
   - Desktop host shim remains in `cortex/apps/cortex-desktop/src/services/acp_protocol.rs`

## Determinism and Purity Checks

- Domain/runtime policy paths contain no extracted `OnceLock`, `std::env`, `std::fs`, or wall-clock APIs.
- Runtime protocol orchestration is port-driven (`SessionStorePort`, `PermissionLedgerPort`, `AcpProtocolHost`, `OperationAdapter`).

## Local Validation Commands

Executed on 2026-02-16 and green:

1. `bash /Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh`
   - PASS: `inventory=123 fixtures=123 exemptions=0`
2. `bash /Users/xaoj/ICP/scripts/run_cortex_runtime_freeze_gates.sh`
   - PASS: terminology/domain/runtime purity, wasm checks, parity replay, ACP matrix, cloud-event parity, shadow strictness
3. `bash /Users/xaoj/ICP/scripts/check_118_pr_evidence.sh --pr-body-file /Users/xaoj/ICP/tests/fixtures/pr_evidence/valid.md`
   - PASS
4. `bash /Users/xaoj/ICP/scripts/check_118_pr_evidence.sh --pr-body-file /Users/xaoj/ICP/tests/fixtures/pr_evidence/phase2_slice_template.md`
   - PASS
5. `bash /Users/xaoj/ICP/tests/scripts/test_check_118_pr_evidence.sh`
   - PASS (negative fixtures fail as expected)

## PR Evidence Contract Staging

- Template staged: `tests/fixtures/pr_evidence/phase2_slice_template.md`
- Fixture harness updated to enforce template validity:
  `tests/scripts/test_check_118_pr_evidence.sh`

## Governance Notes

- Required-check ruleset path remains preferred.
- If rulesets are unavailable, interim steward-only merge + mandatory evidence bundle remains valid per ADR-017 policy.
- Per-slice steward authorization records for PR-2 through PR-7 are now tracked in:
  `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/REMOTE_GOVERNANCE_LEDGER_2026-02-17.md`
