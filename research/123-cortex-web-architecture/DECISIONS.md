# Decision Log — Initiative 123

## DEC-123-005: Canonical Web Chat Surface Uses Runtime-First Dispatch and Heap-Backed Conversation Projection
**Date**: 2026-03-28  
**Status**: Approved

**Decision**: Standardize the `cortex-web` conversation surface so `/ws/chat` is a thin transport, the gateway resolves canonical heap context and persisted thread history, generation dispatches to the provider-runtime Responses path first, and conversation history persists as heap-backed block content projected back to the web host through canonical chat routes.

**Rationale**:
1. The prior prototype shape split authority across browser `localStorage`, ad hoc thread UI state, and a text-only generation path.
2. Host-neutrality requires the runtime to own context resolution, persistence, and generation routing rather than letting `cortex-web` become a second conversation authority.
3. Shared A2UI rendering and heap-backed projections align chat with the same execution-layer contracts used elsewhere in the Workbench.

**Implications**:
1. Canonical chat browse/hydrate routes are:
   - `GET /api/cortex/chat/conversations`
   - `GET /api/cortex/chat/conversations/:thread_id`
2. Persisted conversation content uses canonical heap-backed block forms (`conversation_message`, `conversation_a2ui`, `conversation_pointer`).
3. `/conversations` remains temporary only as a projection UI over canonical server-backed state; browser `localStorage` is cache, not authority.
4. `workflow-engine.process_message` may remain available only as compatibility fallback while provider-runtime is the primary generator path.

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
1. Runtime API contract remains stable under `/api/system/*` and `/api/kg/spaces/:space_id/contribution-graph/*`.
2. Host UIs consume shared contracts and artifacts only.
3. Steward approval envelope remains mandatory for mutating pipeline actions.

## DEC-123-003: SpatialPlane Phase 5 Evidence-First Hardening
**Date**: 2026-02-22  
**Status**: Approved

**Decision**: Complete SpatialPlane Phase 5 as a web-first, evidence-first hardening gate with JSONL experiment persistence and a desktop parity contract/spec, while keeping feature flags default-off.

**Rationale**:
1. Promotion decisions require deterministic replay evidence and measurable fallback/error rates.
2. Web host is available now for controlled operator evaluation; desktop implementation can follow a fixed parity contract.
3. JSONL event persistence enables auditable run summaries and go/no-go computation without manual reconstruction.

**Implications**:
1. Spatial experiment events are persisted via gateway APIs and run summaries are queryable by `run_id`.
2. Event contract is locked for `cortex:a2ui:event` emissions across adapters and widgets.
3. Desktop implementation remains deferred, but parity requirements are frozen in `SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md`.

## DEC-123-004: Formally Deprecate Dioxus as Primary UI Shell
**Date**: 2026-03-01  
**Status**: ✅ Decided

**Decision**: Formally deprecate Dioxus as the primary architectural choice for Cortex execution shells. 

**Rationale**: While Dioxus aligned with the Rust-native "Single Language" purity goals, the operational reality of injecting dynamic visualization engines (D3, xterm.js) and managing the webview serialization bridging generated cascading technical debt. Aligning the primary interactive shell directly with the React ecosystem (`cortex-web`) natively opens access to the broader AI toolchain (AG-UI), enables simpler generative UI workflows, and stabilizes complex graphical interfaces.

**Implications**: The `cortex-desktop` crate should be viewed functionally as a headless Temporal worker, daemon, and local gateway. Visually interactive capabilities are deferred entirely to the `cortex-web` React application. References across the research base referring to Dioxus as the active UI substrate should be considered deprecated.
