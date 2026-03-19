# Decision Log — Initiative 123

## DEC-123-001: Canonical Initiative Numbering and Scope
**Date**: 2026-02-22  
**Status**: Approved

**Decision**: Canonicalize Cortex Web Architecture as initiative `123-cortex-web-architecture` and preserve `119-nostra-commons` lineage as completed.

**Rationale**:
1. Duplicate use of ID 119 created portfolio ambiguity and status-index conflicts.
2. Commons closure evidence is already attached to initiative 119 and must remain immutable.
3. Web host work is active and requires independent execution tracking.

**Implications**:
1. All new web-host architectural work tracks under initiative 123.
2. Portfolio status index must include 122 and 123 entries.
3. Old `119-cortex-web-architecture` folder path is deprecated.

## DEC-123-002: Dual-Host Runtime Contract
**Date**: 2026-02-22  
**Status**: Approved

**Decision**: Operate Desktop and Web as equal Cortex hosts behind a host-neutral Workbench API surface.

**Rationale**:
1. Research 118 requires host plurality and thin-host boundaries.
2. DPub Workbench behavior must be deterministic across hosts.
3. Governance controls must remain centralized and auditable.

**Implications**:
1. Runtime API contract remains stable under `/api/system/*` and `/api/kg/spaces/:space_id/initiative-graph/*`.
2. Host UIs consume shared contracts and artifacts only.
3. Steward approval envelope remains mandatory for mutating pipeline actions.
