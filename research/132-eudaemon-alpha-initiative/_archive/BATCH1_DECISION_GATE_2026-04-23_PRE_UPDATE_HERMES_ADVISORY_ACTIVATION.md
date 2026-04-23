# Initiative 132 Batch 1 Decision Gate

**Date**: 2026-04-01
**Outcome**: `Batch 1 Accepted; Batch 2 Deferred`

## Decision

Batch 1 provider-runtime extraction is accepted as current-stage implementation progress. The provider-admin discovery, record-shaping, and auth-binding helper paths now sit behind narrower `provider_runtime` and `provider_admin` boundaries, and the governed parity checks are green after the move.

Initiative 132 should not start Batch 2 ACP / terminal execution-control extraction in the same stage pass by default.

## Why

1. The provider-runtime boundary is materially narrower than it was at the start of Batch 1:
   - live-provider discovery aggregation and remote SSH host probe logic now live in `provider_runtime::discovery`
   - provider record shaping and state-upsert helpers now live in `gateway::provider_admin`
   - auth-binding probe-key resolution and runtime-provider binding fallback now live in `gateway::provider_admin::state`
2. The operator-facing contract stayed intact:
   - split operator inventory reads still match the aggregate provider surface
   - non-executable providers are still rejected from execution-binding writes
   - `gateway_parity` is green after the extraction
3. The current stage rule still applies:
   - Batch 2 should not begin merely because Batch 1 stayed green
   - ACP remains the next seam, but it is not blocking the provider-runtime move that just landed

## Answers Required By The Gate

### Is provider runtime now ready for deeper extraction?

Yes, within the validated limits of the current stage.

The gateway no longer owns the remote runtime-host discovery pipeline or the provider-admin auth-binding helper path, which means the remaining provider-runtime logic can now be evaluated from a much smaller app-layer surface.

### Did ACP contract or terminal execution concerns become blocking during Batch 1?

No.

The provider-runtime work stayed isolated enough that ACP does not need to move in order to preserve the current extraction.

### Did workbench or heap projection issues surface that block the next stage decision?

No new workbench or heap blockers surfaced in this Batch 1 slice.

The previously known heap/workbench compatibility cleanup remains real, but it did not regress or become newly coupled to the provider-runtime extraction.

## Evidence

### Targeted Verification

- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib resolve_provider_probe_api_key_prefers_explicit_key`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib resolve_provider_probe_api_key_uses_provider_binding_when_requested`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib parse_remote_ollama_probe_output_reports_missing_runtime`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib remote_discovery_record_from_runtime_host_uses_host_scoped_provider_identity`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib system_providers_aggregate_matches_split_operator_reads`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib put_system_provider_binding_rejects_non_executable_provider`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test gateway_parity`
  - passed

### Governance / Contract Checks

- `bash /Users/xaoj/ICP/scripts/check_dynamic_config_contract.sh`
  - passed
- `python3 /Users/xaoj/ICP/scripts/check_research_portfolio_consistency.py`
  - passed
- `bash /Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh --repo-contract`
  - passed

## Next Step For This Stage

Record Batch 1 as materially advanced, keep Batch 2 deferred by default, and only begin ACP / terminal execution-control extraction as its own governed batch rather than smuggling it into the provider-runtime closeout.
