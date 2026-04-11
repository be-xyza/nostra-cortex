# Provider Runtime Boundary Implementation

Updated: 2026-04-10
Base: `origin/main`
Restore source: `codex/execution-canvas-runtime-recovery`
Planning reference: `codex/runtime-provider-boundary-review`

## Purpose

This branch is the clean implementation lane for the provider-runtime boundary slice after the repo clean-state migration merged.

## Owned Scope

- `cortex/apps/cortex-desktop/src/gateway/server.rs`
- `cortex/apps/cortex-eudaemon/src/gateway/**`
- `cortex/apps/cortex-eudaemon/src/services/provider_runtime/**`
- `cortex/apps/cortex-eudaemon/src/services/provider_probe.rs`
- `cortex/apps/cortex-eudaemon/src/gateway/session_auth.rs`
- `cortex/apps/cortex-eudaemon/tests/fixtures/gateway_baseline/**`
- `cortex/apps/cortex-git-adapter/**`

## Restore Rule

Only provider/runtime boundary files may be restored from the recovery anchor. Web shell, heap UI, Nostra extraction, generated artifacts, and local logs are out of scope.

## Immediate Next Step

Restore only the owned paths from `codex/execution-canvas-runtime-recovery`, run the branch-specific gateway/parity checks, and open the first recovery PR from this branch rather than from the salvage anchor.
