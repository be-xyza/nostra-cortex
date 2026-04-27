# Regression Test Coverage

**Date**: 2026-04-26

## Added Tests

- `cargo test -p cortex-runtime workflow::local_durable_worker`
  - Verifies internal snapshot materialization when no external bridge snapshot exists.
- `cargo test -p cortex-eudaemon workflow_instance_start_materializes_local_snapshot_without_bridge_snapshot`
  - Verifies local instance start returns waiting_checkpoint with trace/checkpoints instead of queued/empty.

## Existing Tests Kept Passing

- `cargo test -p nostra_workflow_engine`
- `cargo test -p nostra_backend`
- `cargo test -p cortex-ic-adapter`
- `cargo test -p cortex-eudaemon workflow_binding_accepts_canister_adapter`
- `cargo test -p cortex-eudaemon workflow_instance_start_returns_queued_instance_without_snapshot`
- `npm run check`
- `npm run build`
- `npm run test:workflow`
