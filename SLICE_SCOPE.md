# Runtime Provider Boundary Review

Updated: 2026-04-10
Base: `codex/execution-canvas-runtime-recovery`

## Purpose

This branch isolates provider-runtime, session-auth, and gateway-boundary changes from the execution recovery anchor so they can be reviewed without web-shell or Nostra extraction churn.

## Primary Scope

- `cortex/apps/cortex-desktop/src/gateway/server.rs`
- `cortex/apps/cortex-eudaemon/src/gateway/**`
- `cortex/apps/cortex-eudaemon/src/services/provider_runtime/**`
- `cortex/apps/cortex-eudaemon/src/services/provider_probe.rs`
- `cortex/apps/cortex-eudaemon/src/gateway/session_auth.rs`
- `cortex/apps/cortex-eudaemon/tests/fixtures/gateway_baseline/**`
- `cortex/apps/cortex-git-adapter/**`

## Exclusions

- `cortex/apps/cortex-web/**`
- heap and execution-canvas UI changes
- `nostra/**`, `libraries/**`, and extraction/declaration churn
- generated runtime logs under app-local `logs/`

## Review Goal

Land a provider/runtime boundary slice that keeps operator-only execution surfaces explicit, preserves parity evidence, and excludes unrelated presentation work.
