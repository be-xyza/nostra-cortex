# Asset Restoration Manifest (2026-02-15)

## Purpose
Restore Initiative 118 runtime/parity assets required to run freeze governance gates from tracked repository content.

## Restored Paths

- `/.github/workflows/test-suite.yml`
  - Re-enabled disabled CI jobs and added preflight guards.
  - Kept required 118 gate job names unchanged.

- `/nostra/Cargo.toml`
  - Added workspace manifest expected by freeze scripts (`--manifest-path nostra/Cargo.toml`).

- `/nostra/apps/cortex-desktop/Cargo.toml`
  - Added package manifest with `cortex_runtime_v0` feature and dependencies for ACP/runtime parity tests.

- `/nostra/apps/cortex-desktop/src/gateway/server.rs`
  - Canonical source for gateway endpoint inventory lock.

- `/nostra/apps/cortex-desktop/src/lib.rs`
- `/nostra/apps/cortex-desktop/src/services/mod.rs`
- `/nostra/apps/cortex-desktop/src/services/acp_*.rs`
- `/nostra/apps/cortex-desktop/src/services/file_system_service.rs`
- `/nostra/apps/cortex-desktop/src/services/terminal_service.rs`
- `/nostra/apps/cortex-desktop/src/services/artifact_collab_crdt.rs`
  - Restored ACP/runtime service code required by freeze-gate parity tests and PR-1 extraction target continuity.

- `/nostra/apps/cortex-desktop/tests/gateway_parity.rs`
- `/nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline/**`
  - Restored inventory lock + parity fixture baseline.

- `/nostra/libraries/cortex-domain/**`
- `/nostra/libraries/cortex-runtime/**`
- `/nostra/libraries/cortex-ic-adapter/**`
- `/nostra/libraries/nostra-cloudevents/**`
  - Restored Phase 0 crate foundations and required runtime dependency.

- `/scripts/check_gateway_parity_inventory_sync.sh`
- `/scripts/run_cortex_runtime_freeze_gates.sh`
- `/scripts/check_cortex_domain_purity.sh`
- `/scripts/check_cortex_runtime_purity.sh`
- `/scripts/check_nostra_cortex_terminology.sh`
  - Restored Initiative 118 gate runner/check scripts.

- `/AGENTS.md`
- `/nostra/spec.md`
- `/research/README.md`
- `/docs/reference/README.md`
- `/docs/architecture/nostra-cortex-boundary.md`
  - Restored canonical terminology/layer-boundary documents referenced by gate scripts.

## Exclusions

- Local caches, tooling state, and runtime artifacts (e.g. `.cache`, `.cargo-target`, `node_modules`, `.agent`, logs) were not imported.
- No deploy/state mutation artifacts were included.
