import fixture from "./shellSurfaceInventory.fixture.json";

export type ShellSurfaceRouteClass =
  | "governed_a2ui_surface"
  | "native_cortex_host_surface"
  | "execution_surface"
  | "labs_experiment"
  | "placeholder"
  | "redirect_alias"
  | "operator_only_admin"
  | "settings_gap";

export type ShellSurfaceInventoryFixture = {
  schema_version: "CortexWebShellSurfaceInventoryV1";
  snapshot_id: "system:ux:shell-surface-inventory";
  inventory_id: string;
  authority_mode: "recommendation_only";
  routes: Array<{
    route: string;
    class: ShellSurfaceRouteClass;
    status: string;
    visible_in_nav: boolean;
    target?: string;
    host_component?: string;
    contract_status?: string;
    known_gap?: {
      severity: "low" | "medium" | "high";
      summary: string;
      recommended_action: string;
    };
  }>;
  visible_navigation: Array<{
    label: string;
    route: string;
    slot: string;
    source: string;
    expected_route_status: string;
  }>;
  settings_requirements: Array<{
    category: string;
    items: string[];
  }>;
};

export const SHELL_SURFACE_INVENTORY_FIXTURE = fixture as ShellSurfaceInventoryFixture;

export function buildShellSurfaceInventoryResponse(): ShellSurfaceInventoryFixture {
  return SHELL_SURFACE_INVENTORY_FIXTURE;
}
