# Cortex Web Heap Shell Review

Updated: 2026-04-10
Base: `codex/execution-canvas-runtime-recovery`

## Purpose

This branch isolates the web execution shell, heap presentation, and related shell-navigation changes from the execution recovery anchor.

## Primary Scope

- `cortex/apps/cortex-web/**`
- `cortex/apps/cortex-desktop/src/services/heap_mapper.rs`
- `cortex/apps/cortex-eudaemon/src/services/heap_mapper.rs`
- `cortex/apps/cortex-eudaemon/src/workflows/spatial_synthesis.rs`

## Exclusions

- provider-runtime and gateway-admin backend changes
- `nostra/**`, `libraries/**`, declarations, and extraction churn
- app-local generated logs and replay runtime spill

## Review Goal

Turn the current salvage-heavy web delta into a UI-focused branch where shell, heap, conversation, and provider-dashboard surfaces can be reviewed as one coherent workbench slice.
