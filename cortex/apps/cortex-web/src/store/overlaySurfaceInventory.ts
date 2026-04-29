import fixture from "./overlaySurfaceInventory.fixture.json";

export type OverlayAuthorityClass =
  | "local_ui_state"
  | "runtime_read"
  | "runtime_write"
  | "steward_gated_write"
  | "operator_only"
  | "destructive_confirmation";

export type OverlaySurfaceKind =
  | "modal"
  | "sheet"
  | "drawer"
  | "sidebar"
  | "panel"
  | "popover"
  | "confirmation";

export type OverlaySurfaceInventoryFixture = {
  schema_version: "CortexWebOverlaySurfaceInventoryV1";
  snapshot_id: "system:ux:overlay-surface-inventory";
  inventory_id: string;
  authority_mode: "recommendation_only";
  overlay_surfaces: Array<{
    surface_id: string;
    owner_route: string;
    component: string;
    surface_kind: OverlaySurfaceKind;
    authority_class: OverlayAuthorityClass;
    open_trigger: string;
    close_mechanisms: string[];
    focus_policy: string;
    escape_policy: string;
    z_index_band: string;
    can_stack_with: string[];
    known_collision: string;
    state_source: string;
    persistence: string;
    known_gap?: {
      severity: "low" | "medium" | "high";
      summary: string;
      recommended_action: string;
    };
    recommended_action: string;
  }>;
  known_gaps: Array<{
    id: string;
    severity: "low" | "medium" | "high";
    summary: string;
    recommended_action: string;
  }>;
};

export const OVERLAY_SURFACE_INVENTORY_FIXTURE =
  fixture as OverlaySurfaceInventoryFixture;

export function buildOverlaySurfaceInventoryResponse(): OverlaySurfaceInventoryFixture {
  return OVERLAY_SURFACE_INVENTORY_FIXTURE;
}
