# Overlay Surface Observability

## Scope

This note records the advisory overlay and modal contract for Cortex Web. It documents current frontend truth without changing product behavior, runtime authority, overlay layering, focus handling, or global settings IA.

Authority mode: `recommendation_only`.

## Contract Surfaces

- Fixture: `cortex/apps/cortex-web/src/store/overlaySurfaceInventory.fixture.json`
- Type wrapper: `cortex/apps/cortex-web/src/store/overlaySurfaceInventory.ts`
- Snapshot id: `system:ux:overlay-surface-inventory`
- Preview endpoint: `/api/system/ux/overlay-surface-inventory`
- Validator: `scripts/check_cortex_web_overlay_surface_inventory.py`

The fixture classifies modal, sheet, drawer, sidebar, panel, popover, and confirmation overlays. It records owner route, component, surface kind, authority class, open and close mechanisms, focus and escape policy, z-index band, stack compatibility, collisions, state source, persistence, known gaps, and recommended actions.

## Current Truth

- Heap owns detail, chat, comment, steward gate, aggregation detail, saved-view, history, and create overlays.
- Shell owns Space selector, role selector, and workbench naming overlays. Saved-view confirmations remain owner-route scoped until global settings IA exists.
- System provider create, detail, and discovery sheets are operator-only surfaces and need route-level identity and redaction provenance.
- Artifact and workflow inspector panels are runtime-read surfaces in this stage; replay-style affordances do not gain mutation authority.
- Shared destructive confirmation is cross-route and can stack over route-owned overlays.

## Known Gaps

- Chat panel can intercept create controls.
- Overlay lifecycle metadata is thin across modal, drawer, sheet, sidebar, popover, panel, and confirmation surfaces.
- Chat hydration and socket failures can be under-observed.
- Provider/operator sheets need route-level redaction and identity provenance.
- Artifact inspector has a console-only workflow action placeholder that should be implemented or removed later.

## Follow-On Validation Targets

- Add open, close, escape, and focus smoke coverage for Heap detail, chat, comment, steward gate, provider sheets, saved-view modal, and workflow inspector.
- Add explicit provider sheet provenance and redaction metadata before exposing more operator topology state.
- Introduce a shared overlay lifecycle contract before redesigning overlay stacking or focus behavior.
