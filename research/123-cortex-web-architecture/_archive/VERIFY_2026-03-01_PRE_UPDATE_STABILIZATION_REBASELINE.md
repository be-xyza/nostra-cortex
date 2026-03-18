# Verification Checklist — Initiative 123

## Portfolio Integrity
- [x] `RESEARCH_INITIATIVES_STATUS.md` has no conflict markers.
- [x] `119-nostra-commons` remains completed.
- [x] `123-cortex-web-architecture` is indexed and active.

## Runtime and Host Parity
- [x] Gateway `/api/system/build` and `/api/system/ready` return healthy payloads.
- [x] Dual-host baseline parity script passes (`PASS: dual-host gateway baseline`).
- [x] Recommended path reported as `goal_dependency_closure` for default parity check corpus.

## Governance
- [ ] Mutating pipeline action blocked without approval envelope.
- [ ] Mutating pipeline action succeeds with steward envelope.

## SpatialPlane Phase 5 Gate Evidence (2026-02-22)
- [x] `npm run check` in `cortex/apps/cortex-web`.
- [x] `npm run test:spatial` in `cortex/apps/cortex-web` (5/5 pass).
- [x] `npm run build` in `cortex/apps/cortex-web`.
- [x] `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::spatial_experiment` (endpoint tests pass).
- [x] `bash /Users/xaoj/ICP/scripts/check_cortex_dual_host_parity.sh` (pass).
