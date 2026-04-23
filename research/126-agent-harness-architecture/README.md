# Initiative 126: Agent Harness Architecture

Formalizes a lean, governed Agent Harness for Cortex that emits canonical `GlobalEvent` lifecycle records while preserving Nostra authority boundaries.

## Status
- **State**: Complete (2026-02-24)
- **Scope**: Gateway-first v1 (`L0-L2` implemented, `L3/L4` fail-closed)
- **Delivery mode**: Local JSONL durable lifecycle events with optional remote sink

## Current Artifacts
- `PLAN.md`: architecture, scope cutline, and implementation phases
- `REQUIREMENTS.md`: functional/non-functional requirements for v1
- `DECISIONS.md`: architectural decisions and rationale
- `VERIFY.md`: verification objectives and evidence targets
- `_archive/`: pre-update snapshots

## Scope Relationship
- Extends `122-cortex-agent-runtime-kernel` with governance-grade lifecycle, authority, and replay contracts.
- Does not supersede 122 MVK loop design; it narrows and operationalizes it.

## Closeout Notes
- Verification evidence and command results are recorded in `VERIFY.md`.
- Accepted implementation decisions are recorded in `DECISIONS.md`.
- Residual risk is limited to an unrelated `cortex-domain` branding test include-path issue outside 126 implementation scope.
