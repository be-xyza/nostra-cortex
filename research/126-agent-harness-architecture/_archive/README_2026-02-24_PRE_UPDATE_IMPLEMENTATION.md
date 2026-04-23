# Initiative 126: Agent Harness Architecture

Formalizes a lean, governed Agent Harness for Cortex that emits canonical `GlobalEvent` lifecycle records while preserving Nostra authority boundaries.

## Current Artifacts
- `PLAN.md`: architecture, scope cutline, and implementation phases
- `REQUIREMENTS.md`: functional/non-functional requirements for v1
- `DECISIONS.md`: architectural decisions and rationale
- `VERIFY.md`: verification objectives and evidence targets
- `_archive/`: pre-update snapshots

## Scope Relationship
- Extends `122-cortex-agent-runtime-kernel` with governance-grade lifecycle, authority, and replay contracts.
- Does not supersede 122 MVK loop design; it narrows and operationalizes it.
