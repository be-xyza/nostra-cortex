import fixture from "./routeIaInventory.fixture.json";

export type RouteIaClass =
  | "native_cortex_host_surface"
  | "governed_a2ui_surface"
  | "operator_only_admin"
  | "labs_experiment"
  | "a2ui_candidate"
  | "settings_absence_contract"
  | "redirect_or_alias";

export type RouteReadinessStatus =
  | "functional"
  | "functional_thin"
  | "degraded_empty"
  | "inconsistent"
  | "under_construction"
  | "missing"
  | "failing_500";

export type RouteIaInventoryFixture = {
  schema_version: "CortexWebRouteIaInventoryV1";
  snapshot_id: "system:ux:route-ia-inventory";
  inventory_id: string;
  authority_mode: "recommendation_only";
  settings_absence_contract: {
    route_id: "/settings";
    readiness_status: "missing";
    global_settings_page_allowed_this_stage: false;
    current_owner_rule: string;
  };
  routes: Array<{
    route_id: string;
    declared_route_owner: string;
    nav_source: string;
    route_class: RouteIaClass;
    typed_route: boolean;
    a2ui_fallback_allowed: boolean;
    visible_in_nav: boolean;
    detail_tabs: string[];
    settings_affordances: string[];
    operator_boundary: string;
    readiness_status: RouteReadinessStatus;
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

export const ROUTE_IA_INVENTORY_FIXTURE = fixture as RouteIaInventoryFixture;

export function buildRouteIaInventoryResponse(): RouteIaInventoryFixture {
  return ROUTE_IA_INVENTORY_FIXTURE;
}
