import fixture from "./graphSurfaceInventory.fixture.json";

export type GraphSurfaceKind =
  | "contribution"
  | "capability"
  | "workflow"
  | "spatial_plane"
  | "relation";

export type GraphSourceAuthority =
  | "nostra"
  | "cortex"
  | "a2ui"
  | "fixture"
  | "local_cache";

export type GraphLayoutEngine =
  | "react_force_graph"
  | "react_flow"
  | "tldraw"
  | "d3"
  | "list"
  | "json"
  | "svg_fallback";

export type GraphSurfaceCount = number | "runtime_dependent" | "not_exposed";

export type GraphSurfaceInventoryFixture = {
  schema_version: "CortexWebGraphSurfaceInventoryV1";
  snapshot_id: "system:ux:graph-surface-inventory";
  inventory_id: string;
  authority_mode: "recommendation_only";
  graph_surfaces: Array<{
    surface_id: string;
    route_id: string;
    graph_kind: GraphSurfaceKind;
    source_authority: GraphSourceAuthority;
    source_endpoint: string;
    schema_version: string;
    projection_kind: string;
    graph_hash?: string;
    graph_root_hash?: string;
    layout_engine: GraphLayoutEngine;
    renderer_mode: string;
    render_status: string;
    fetch_status: string;
    node_count: GraphSurfaceCount;
    edge_count: GraphSurfaceCount;
    visible_node_count: GraphSurfaceCount;
    visible_edge_count: GraphSurfaceCount;
    fallback_reason?: string;
    known_gap?: {
      severity: "low" | "medium" | "high";
      summary: string;
      recommended_action: string;
    };
  }>;
  known_gaps: Array<{
    id: string;
    severity: "low" | "medium" | "high";
    summary: string;
    recommended_action: string;
  }>;
};

export const GRAPH_SURFACE_INVENTORY_FIXTURE =
  fixture as GraphSurfaceInventoryFixture;

export function buildGraphSurfaceInventoryResponse(): GraphSurfaceInventoryFixture {
  return GRAPH_SURFACE_INVENTORY_FIXTURE;
}
