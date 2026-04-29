# Cortex Web Shell Surface Observability

## Scope

This note defines the first observability slice for Cortex Web frontend hardening:

- shell navigation entries
- route inventory
- route/surface classification
- settings information architecture gaps
- visible navigation pointing to placeholders, redirects, or failing routes
- first-paint console signals

This slice does not validate Heap block capabilities, block action menus, upload/extraction flows, steward-gate mutations, relation editing, or chat-to-block behavior. Those belong in a follow-on Heap / Block Capability Observability fixture after the shell map is stable.

## Current Finding Summary

The active local shell at `http://127.0.0.1:4173/` has a real route/surface mismatch: some routes are functional, some routes are native React host surfaces rather than governed A2UI surfaces, and some visible navigation points to placeholders.

High-priority gaps:

- `Global Discovery` is visible in shell navigation but `/discovery` renders an under-construction placeholder.
- `/labs/execution-canvas` is an execution surface route but returns a surface-fetch HTTP 500 in local preview.
- `/spaces` has contradictory registry/readiness signals and emits 404s in local preview.
- There is no first-class `/settings` surface despite existing user, layout, registry, theme, role, Space, runtime, and design-governance preferences.

Medium-priority gaps:

- `/contributions` is a functional steward cockpit, but it is a native Cortex Web host surface, not a WorkbenchSurfaceView/A2UI surface. That can be valid, but it needs explicit route metadata and tests.
- `/workflows` renders a cockpit but mostly empty/degraded local state.
- `/artifacts` renders a governed A2UI surface with a native inspector, but interaction and error states remain thin.
- Shell overlays can intercept each other: the Space selector can block role-selector clicks, and the chat panel can block the create button.

## Contract Fixture

The seed fixture is:

- `cortex/apps/cortex-web/src/store/shellSurfaceInventory.fixture.json`

The local validator is:

- `python3 scripts/check_cortex_web_shell_surface_inventory.py`

The validator keeps this stage recommendation-only. It does not fail merely because a route has a known gap; it fails when the known gap is not explicitly recorded with severity, summary, and recommended action.

## Settings Surface Requirements

A future Cortex Web settings surface should separate authority levels:

- Personal preferences: theme/contrast, reduced motion, nav mode, saved filters, custom views, local preview versus live registry mode.
- Space settings: Space identity/readiness, navigation plan, capability overlay, active Space design profile preview, member/agent participation policy, Space-specific layout preferences.
- Workbench settings: shell layout source, route/surface inventory, A2UI fixture mode, visible/hidden nav entries, accessibility diagnostics.
- Operator settings: providers, gateway target, runtime health, logs/telemetry stream, execution surface visibility, auth/session diagnostics.
- Design/theme governance: NDL/A2UI token source, Cortex Web preview fixture, Space design profiles, unresolved design lint findings, route classification compliance.

## Follow-On Scope

Heap / Block Capability Observability should be a separate fixture and validation pass covering:

- block type rendering
- block action menu availability
- create block flow
- upload/extraction flow
- comments/discussions
- steward gates
- relation editor
- block detail modal tabs
- chat-to-block interactions
- aggregation and derived views
- action authority by role and Space
