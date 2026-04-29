# Graph Surface Observability

## Scope

This note records the advisory graph surface contract for Cortex Web. It treats current frontend behavior as locked truth and adds observability around graph-like surfaces without changing product behavior, runtime authority, or graph data correctness.

Authority mode: `recommendation_only`.

## Contract Surfaces

- Fixture: `cortex/apps/cortex-web/src/store/graphSurfaceInventory.fixture.json`
- Type wrapper: `cortex/apps/cortex-web/src/store/graphSurfaceInventory.ts`
- Snapshot id: `system:ux:graph-surface-inventory`
- Preview endpoint: `/api/system/ux/graph-surface-inventory`
- Validator: `scripts/check_cortex_web_graph_surface_inventory.py`

The fixture classifies contribution, capability, workflow, spatial-plane, and Heap relation graph surfaces. It records route ownership, graph authority, source endpoint, projection schema, graph hash exposure, layout engine, renderer status, fetch status, visible counts, fallback reasons, known gaps, and recommended actions.

## Current Truth

- Contribution cockpit graph surfaces include runs, full graph, edge quality, weak-edge review, and blast-radius/focus-map projections.
- Heap includes the ambient background graph and relation graph surfaces in the detail modal.
- Capability graph surfaces include `/system` fallback graph data, A2UI `CapabilityMap`, `CapabilityMapV2`, and the Space capability graph editor.
- Workflow projections include `flow_graph_v1`, `normalized_graph_v1`, and `execution_topology_v1`.
- Execution canvas and spatial plane surfaces reference graph data through layout refs and rendered node/edge state.

## Known Gaps

- There is no shared graph health contract across graph renderers today.
- `flow_graph_v1` and `normalized_graph_v1` render as JSON, not graph views.
- A2UI `ContributionGraph` can fall back to mock data.
- Execution canvas does not yet expose topology drift between `layout_ref.graph_hash` and the rendered graph.
- Capability graph rendering lacks explicit hidden, locked, and broken-route observability.

## Follow-On Validation Targets

- Graph smoke coverage should assert non-empty graph rendering or an explicit fallback reason.
- Execution canvas should surface drift between topology refs and rendered spatial nodes/edges.
- A shared graph health contract should normalize fetch, render, fallback, and count metadata across React, A2UI, and spatial-plane graph surfaces.
