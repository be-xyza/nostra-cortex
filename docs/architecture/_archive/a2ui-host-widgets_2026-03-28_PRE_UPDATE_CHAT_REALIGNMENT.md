# A2UI Host Widget Allowlist

## Purpose
Define the small set of frontend host widgets that remain acceptable in `cortex-web` while route-level Workbench UX stays server-driven and primitives-first.

## Allowed Host Widgets
- `HeapCanvas`
  - Hosts the Heap workspace shell and block inventory canvas.
- `HeapBoard`
  - Hosts focused Heap board composition when the server selects a board-style surface.
- `HeapBlockCard`
  - Renders canonical Heap block cards inside generic collections.
- `ContributionGraph`
  - Hosts the graph visualization runtime for contribution metadata inspection.
- `CapabilityMap`
  - Hosts the capability graph visualization for route/capability inspection.
- `TldrawCanvas`
  - Allowed only where an interactive drawing runtime is explicitly required by an existing route contract.

## Forbidden Class
- Monolithic route-level React macro-widgets that replace Workbench projection with client-owned screen logic.
- Examples of forbidden patterns:
  - route-specific synthesis surfaces
  - route-specific playground shells
  - bespoke SIQ/testing dashboards embedded as frontend-only widgets

## Contract Rule
- Workbench route structure, navigation placement, and operator actions must be projected from server-driven A2UI contracts.
- Host widgets are runtime adapters for bounded visualization or canvas primitives, not alternate screen ownership.
- Inline chat bubbles may render shared A2UI surfaces through the canonical React A2UI interpreter, but conversation history and `/conversations` must remain server-backed projections over canonical conversation state.
- Client-side caches such as `localStorage` may optimize chat UX, but they must not become the authority for conversation persistence or route ownership.
