# Initiative 118 — Phase 3 Execution Plan (Governance, Workflow, Streaming)

Date: 2026-02-16
Scope: Phase 3 in `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/PLAN.md`

## 1) Entry Status: Blocker Check

Phase 3 kickoff is unblocked in local workspace as of 2026-02-16.

Validated now:

1. `bash /Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh`
   - PASS: `inventory=123 fixtures=123 exemptions=0`
2. `bash /Users/xaoj/ICP/scripts/run_cortex_runtime_freeze_gates.sh`
   - PASS: terminology, domain/runtime purity, wasm checks, gateway parity, ACP matrix/cloud parity, shadow strictness
3. `bash /Users/xaoj/ICP/scripts/check_118_pr_evidence.sh --pr-body-file /Users/xaoj/ICP/tests/fixtures/pr_evidence/valid.md`
   - PASS
4. `bash /Users/xaoj/ICP/scripts/check_118_pr_evidence.sh --pr-body-file /Users/xaoj/ICP/tests/fixtures/pr_evidence/phase2_slice_template.md`
   - PASS
5. `bash /Users/xaoj/ICP/tests/scripts/test_check_118_pr_evidence.sh`
   - PASS (negative fixtures fail as expected)

Operational constraints (not blockers):
- ADR-017 evidence discipline remains required for each PR.
- Steward authorization records are still required for remote merge workflow.

## 2) Phase 3 Objectives

Extract and split these targets while preserving Runtime Purity Contract v1.3:

1. `governance_client.rs` -> runtime governance orchestration + IC adapter calls
2. `workflow_engine_client.rs` -> runtime workflow orchestration + IC adapter calls
3. `workflow_service.rs` -> runtime workflow service (`std::env` and wall-clock removed)
4. `workflow_executor.rs` -> runtime executor logic (host transport/process concerns removed)
5. `streaming_transport.rs` -> 4-way split:
   - domain protocol types
   - runtime orchestration
   - IC adapter transport
   - host process management
6. `dfx_client.rs` -> `cortex-ic-adapter` DFX wrapper

## 3) Phase 3 PR Slice Plan

### PR-1: Contract and Type Foundation
Deliverables:
1. Add runtime ports in `cortex-runtime`:
   - `GovernanceAdapter`
   - `WorkflowAdapter`
   - `StreamingTransportAdapter`
2. Add domain streaming protocol types:
   - `/Users/xaoj/ICP/cortex/libraries/cortex-domain/src/streaming/types.rs`
3. Register modules in crate roots; keep desktop behavior unchanged.

Exit gate:
1. `cargo check -p cortex-domain --target wasm32-unknown-unknown`
2. `cargo check -p cortex-runtime --target wasm32-unknown-unknown --no-default-features`
3. `bash /Users/xaoj/ICP/scripts/run_cortex_runtime_freeze_gates.sh`

### PR-2: Governance + Workflow IC Adapter Split
Deliverables:
1. Move IC-specific call logic from:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/governance_client.rs`
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/workflow_engine_client.rs`
2. Implement adapter-side clients in:
   - `/Users/xaoj/ICP/cortex/libraries/cortex-ic-adapter/src/governance.rs`
   - `/Users/xaoj/ICP/cortex/libraries/cortex-ic-adapter/src/workflow.rs`
3. Leave host shim in desktop; runtime depends only on traits.

Exit gate:
1. Runtime purity checks show zero `candid`/`ic-agent` usage in runtime crate.
2. Existing desktop integration tests remain green.

### PR-3: Workflow Service + Executor Runtime Extraction
Deliverables:
1. Extract runtime orchestration from:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/workflow_service.rs`
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/workflow_executor.rs`
2. Replace `std::env` with `RuntimeConfig`.
3. Replace `SystemTime::now()` with `TimeProvider`.
4. Replace direct host logging with `LogAdapter`.
5. Keep host-only transport/UI wiring in desktop.

Exit gate:
1. No forbidden runtime APIs in extracted modules.
2. Workflow behaviors match pre-extraction responses for catalog/control endpoints.

### PR-4: Streaming 4-Way Split
Deliverables:
1. Move protocol structures to domain module.
2. Move replay/degrade/ack orchestration to runtime module.
3. Move IC transport code (`ic-agent`, `candid`) to adapter module.
4. Keep `Command`/filesystem/process management host-side.
5. Remove `OnceLock` from extracted runtime path by injecting transport handles.

Exit gate:
1. Runtime has zero `std::process::Command`.
2. Runtime has zero `ic-agent`/`candid`.
3. Streaming unit tests pass with mock `StreamingTransportAdapter`.

### PR-5: DFX Adapter Finalization + Integration
Deliverables:
1. Move DFX command wrapper from desktop service to adapter crate.
2. Keep runtime free of command execution and filesystem coupling.
3. Wire desktop to runtime + adapter composition path.

Exit gate:
1. Freeze/evidence gates green.
2. End-to-end smoke path for governance/workflow/streaming is green under legacy parity expectations.

## 4) Phase 3 Definition of Done

Phase 3 is complete when all are true:

1. Runtime crate has no `ic-agent`, `candid`, `std::process::Command`, `std::env`, direct wall-clock APIs, or host logging macros in extracted paths.
2. Streaming split is complete with domain/runtime/adapter/host boundaries enforced.
3. New adapter traits are in use and runtime remains substrate-neutral.
4. Freeze/evidence gates pass and parity inventory remains locked at zero exemptions.
5. Completion evidence artifact is recorded:
   - `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/PHASE_3_COMPLETION_EVIDENCE_2026-02-16.md` (or current completion date).

## 5) Risk Controls and Rollback

1. Keep legacy desktop path available per PR until parity proves green.
2. Scope each PR to one extraction concern; do not combine governance/workflow/streaming in a single merge.
3. If parity regresses, disable new path and keep failing fixtures as evidence.
4. Require PR evidence body fields per ADR-017 contract before merge.
