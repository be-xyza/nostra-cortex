# Verification Checklist — Initiative 123

## Portfolio Integrity
- [x] `RESEARCH_INITIATIVES_STATUS.md` has no conflict markers.
- [x] `119-nostra-commons` remains completed.
- [x] `123-cortex-web-architecture` is indexed and active.

## Runtime and Contract Parity (Latest Run: 2026-03-01)
- [x] `bash scripts/check_gateway_parity_inventory_sync.sh` passes with strict zero exemptions.
- [x] `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml --test gateway_parity` passes (`6 passed`).
- [ ] `bash scripts/check_cortex_dual_host_parity.sh` is currently blocked unless gateway is running on `127.0.0.1:3000`.

## Governance
- [ ] Mutating pipeline action blocked without approval envelope.
- [ ] Mutating pipeline action succeeds with steward envelope.

## Cortex-Web Stabilization Evidence (Latest Run: 2026-03-01)
- [x] `npm run check` in `cortex/apps/cortex-web`.
- [x] `npm run test:spatial` in `cortex/apps/cortex-web` (`10/10` pass).
- [x] `npm run build` in `cortex/apps/cortex-web`.
- [x] Spatial event contract lock enforces the 7 approved event types (`spatialEventContract.test.ts`).
- [x] Gateway inventory includes `GET /api/system/capability-graph` and `GET /api/system/ux/workbench`.
