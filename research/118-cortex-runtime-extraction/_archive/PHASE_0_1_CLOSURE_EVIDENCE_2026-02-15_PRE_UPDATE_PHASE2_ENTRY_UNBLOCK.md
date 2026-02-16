# Phase 0/1 Closure Evidence (2026-02-15)

## Scope
This artifact records the gate run used to close Initiative 118 Phase 0 and Phase 1 (event-engine slice only).

## Commands (single-run gate set)

```bash
./scripts/check_gateway_parity_inventory_sync.sh
./scripts/check_nostra_cortex_terminology.sh
./scripts/check_cortex_domain_purity.sh
./scripts/check_cortex_runtime_purity.sh
cargo check --manifest-path nostra/Cargo.toml --target wasm32-unknown-unknown -p cortex-domain
cargo check --manifest-path nostra/Cargo.toml --target wasm32-unknown-unknown -p cortex-runtime --no-default-features
cargo test --manifest-path nostra/Cargo.toml -p cortex-domain
cargo test --manifest-path nostra/Cargo.toml -p cortex-runtime
cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --test gateway_parity
cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 runtime_v0_matches_legacy_for_all_update_kinds
cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 cloud_event_parity_allows_timestamp_format_differences_only
cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_rejects_non_allowed_drift
cargo test --manifest-path nostra/Cargo.toml -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_allows_timestamp_only_drift
bash scripts/run_cortex_runtime_freeze_gates.sh
```

## Results
- PASS: `check_nostra_cortex_terminology.sh`
- PASS: `check_gateway_parity_inventory_sync.sh`
- PASS: `check_cortex_domain_purity.sh`
- PASS: `check_cortex_runtime_purity.sh`
- PASS: `cargo check --target wasm32-unknown-unknown -p cortex-domain`
- PASS: `cargo check --target wasm32-unknown-unknown -p cortex-runtime --no-default-features`
- PASS: `cargo test -p cortex-domain`
- PASS: `cargo test -p cortex-runtime`
- PASS: `cargo test -p cortex-desktop --test gateway_parity` (6 tests passed)
- PASS: `cargo test -p cortex-desktop --features cortex_runtime_v0 runtime_v0_matches_legacy_for_all_update_kinds`
- PASS: `cargo test -p cortex-desktop --features cortex_runtime_v0 cloud_event_parity_allows_timestamp_format_differences_only`
- PASS: `cargo test -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_rejects_non_allowed_drift`
- PASS: `cargo test -p cortex-desktop --features cortex_runtime_v0 shadow_projection_matcher_allows_timestamp_only_drift`
- PASS: `run_cortex_runtime_freeze_gates.sh`

## Inventory Lock Evidence
- Inventory source: `nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline/endpoint_inventory.tsv`
- Inventory mirror: `nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline/endpoint_inventory.json`
- Fixture directory: `nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline/parity_cases`
- Exemptions: `nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline/approved_exemptions.json`
- Lock counts:
  - `inventory_count = 123`
  - `fixture_count = 123`
  - `approved_exemptions_count = 0`
  - `inventory_count == fixture_count + approved_exemptions_count` ✅

## Hybrid Evidence Gate Addendum (2026-02-15)

### Commands

```bash
bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md
bash tests/scripts/test_check_118_pr_evidence.sh
bash scripts/check_gateway_parity_inventory_sync.sh
bash scripts/run_cortex_runtime_freeze_gates.sh
```

### Results
- PASS: `check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md`
- PASS: `tests/scripts/test_check_118_pr_evidence.sh`
  - `valid.md` passes
  - `missing_url.md` fails as expected
  - `bad_counts.md` fails as expected
  - `nonzero_exemptions.md` fails as expected
- PASS: `check_gateway_parity_inventory_sync.sh`
  - `inventory=123 fixtures=123 exemptions=0`
- PASS: `run_cortex_runtime_freeze_gates.sh`

### CI Control Activation
- New CI job: `initiative-118-evidence-gate`
- Existing gate retained: `cortex-runtime-freeze-gates`
- Hybrid merge protocol evidence fields validated by script contract:
  - `118_SCOPE_APPLIES`
  - freeze run URL
  - inventory counts
  - evidence attachment flag

## Post-Merge Compatibility Addendum (2026-02-15)

### Production Run Evidence
- PASS: GitHub workflow run on commit `2af443a` for PR branch
  `codex/initiative-118-hybrid-enforcement` prior to merge.

### Compatibility Constraints Applied
- Artifact action deprecation remediation:
  - `actions/upload-artifact` migrated from `@v3` to `@v4`.
- Workflow compatibility guardrails for this repository snapshot:
  - Unsupported `hashFiles(...)` expressions removed from job-level conditionals.
  - Non-applicable jobs explicitly disabled in this snapshot where required project paths do not exist.
- `initiative-118-evidence-gate` remained active for pull request validation.
