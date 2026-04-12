# Cortex Terminal Host Plan

## Purpose

Define the promotion path for `a2ui-terminal` from experiment to operator-grade Cortex host adapter without duplicating [cortex-web](/Users/xaoj/ICP/cortex/apps/cortex-web).

## Status

- Authority mode: recommendation-only for promotion decisions
- Current implementation status: experiment contract probe
- Intended execution layer: Cortex host adapter
- Intended audience: operators and stewards, not general end users

## Core Decision

`a2ui-terminal` is a Cortex host adapter candidate.

It is not:
- a Nostra platform primitive
- a replacement for [cortex-web](/Users/xaoj/ICP/cortex/apps/cortex-web)
- a new authority surface
- a justification for terminal-only payload schemas

## Architectural Alignment

This host plan follows:
- [docs/architecture/nostra-cortex-boundary.md](/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md)
- [docs/architecture/standards.md](/Users/xaoj/ICP/docs/architecture/standards.md)
- [research/118-cortex-runtime-extraction/RESEARCH.md](/Users/xaoj/ICP/research/118-cortex-runtime-extraction/RESEARCH.md)
- [research/124-agui-heap-mode/PLAN.md](/Users/xaoj/ICP/research/124-agui-heap-mode/PLAN.md)
- [research/132-eudaemon-alpha-initiative/PLAN.md](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PLAN.md)

Key rule:
- runtime logic stays host-neutral
- terminal rendering is a host concern
- richer artifact, workflow, spatial, and media surfaces remain in `cortex-web`

## Supported Scope

### Terminal-first
- small A2UI approval flows
- note and task review
- pointer preview
- compact structured-data summary
- explicit web handoff actions

### Terminal summary plus web handoff
- structured data needing drill-down
- charts
- telemetry
- media
- workflow-linked artifacts

### Web-required
- workflow artifact inspector
- spatial/tldraw surfaces
- capability maps and matrices
- schema editing
- contribution graph exploration
- evaluator DAG views
- unsupported widget families

## Workstreams

### Phase 1: Charter and shared contract
- publish this host plan
- publish the shared terminal/web surface contract
- keep the experiment as the validation harness

### Phase 2: Real payload ingestion
- accept real heap and workflow payload envelopes
- avoid fixture-only validation
- keep the payload envelope aligned with web/runtime contracts
- validate terminal-renderable documents against `terminal_document_v1`

### Phase 3: Runtime-backed host slice
- integrate the adapter through existing terminal host infrastructure
- preferred host entry points:
  - [nostra/apps/cortex-desktop/src/services/terminal_service.rs](/Users/xaoj/ICP/nostra/apps/cortex-desktop/src/services/terminal_service.rs)
  - [cortex/apps/cortex-eudaemon/src/services/terminal_service.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/terminal_service.rs)
- keep the ACP terminal API shape unchanged and prove the first slice through `terminal/create`, `terminal/output`, and `terminal/wait_for_exit`
- current live proof is ACP-compatible and exercised through `cortex-eudaemon`; desktop remains the intended promotion target, but this branch does not yet package `cortex-desktop` as a runnable gateway binary

### Phase 4: Promotion gate
- benchmark one terminal-safe operator flow against the equivalent `cortex-web` flow
- promote only if terminal materially improves operator speed or simplicity

## Handoff Rules

- artifact drill-down opens `cortex-web` via `/explore?artifact_id=<id>`
- workflow drill-down opens `cortex-web` via `/workflows?node_id=<gateway_api_path>`
- terminal host should hand off to rendered workbench surfaces, never raw API paths alone

## Deliverables

- shared surface classification contract
- shared workbench handoff contract
- strict `terminal_document_v1` validation boundary
- one runtime-backed terminal host slice
- one benchmark report
- one go/no-go decision on whether to promote to a first-class CLI/TUI app

## Current Command Contract

Use these operator-side entry points while the experiment remains the validation harness:

- `bash scripts/run_a2ui_terminal_payload.sh --fixture=terminal-approval`
- `bash scripts/plan_a2ui_terminal_payload.sh --payload-file=cortex/experiments/a2ui-terminal/examples/workflow_handoff_envelope.json`
- `bash scripts/benchmark_a2ui_terminal.sh`
- `bash scripts/check_a2ui_terminal_desktop_acp_smoke.sh`

The ACP smoke wrapper targets any compatible Cortex ACP gateway host. In this branch, the live proof was run against `cortex-eudaemon` while the desktop crate remains service-wired but not yet exposed as a runnable gateway binary.

## Naming Recommendation

If promoted, prefer:
- `cortex-tui`
- `cortex-operator-cli`

Avoid naming that implies end-user parity with `cortex-web`.
