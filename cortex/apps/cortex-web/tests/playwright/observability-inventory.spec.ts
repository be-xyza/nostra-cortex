import { expect, test, type Page, type Route } from "@playwright/test";
import { readFileSync } from "node:fs";

function readFixture<T>(relativePath: string): T {
  return JSON.parse(readFileSync(new URL(relativePath, import.meta.url), "utf8")) as T;
}

const graphInventory = readFixture<any>("../../src/store/graphSurfaceInventory.fixture.json");
const overlayInventory = readFixture<any>("../../src/store/overlaySurfaceInventory.fixture.json");
const routeIaInventory = readFixture<any>("../../src/store/routeIaInventory.fixture.json");

const EMPTY_SURFACE = {
  type: "surface",
  surfaceId: "surface.observability.smoke",
  title: "Observability Smoke",
  meta: {},
  components: [
    {
      id: "root",
      type: "Column",
      props: {},
      children: ["title"],
    },
    {
      id: "title",
      type: "Heading",
      props: { text: "Observability Smoke" },
    },
  ],
};

const COMMON_HEADERS = { "Content-Type": "application/json" };

async function fulfillJson(route: Route, body: unknown) {
  await route.fulfill({
    status: 200,
    headers: COMMON_HEADERS,
    body: JSON.stringify(body),
  });
}

async function installObservabilityMocks(page: Page) {
  await page.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    const path = url.pathname;

    if (!path.startsWith("/api/")) {
      return route.fulfill({
        status: 200,
        contentType: "text/html",
        body: "<!doctype html><html><body><main>Observability Smoke</main></body></html>",
      });
    }

    if (path === "/api/system/ux/graph-surface-inventory") {
      return fulfillJson(route, graphInventory);
    }
    if (path === "/api/system/ux/overlay-surface-inventory") {
      return fulfillJson(route, overlayInventory);
    }
    if (path === "/api/system/ux/route-ia-inventory") {
      return fulfillJson(route, routeIaInventory);
    }
    if (path === "/api/system/session") {
      return fulfillJson(route, {
        authenticated: true,
        principal: "2vxsx-fae",
        activeRole: "operator",
        authMode: "preview",
      });
    }
    if (path === "/api/system/whoami") {
      return fulfillJson(route, {
        principal: "2vxsx-fae",
        roles: ["operator"],
        source: "observability-smoke",
      });
    }
    if (path === "/api/cortex/layout/spec") {
      return fulfillJson(route, {
        layoutId: "shell_layout_v2",
        navigationGraph: {
          entries: [
            { routeId: "/spaces", label: "Spaces", icon: "grid", category: "Core", requiredRole: "operator" },
            { routeId: "/contributions", label: "Contributions", icon: "git-merge", category: "Bridge", requiredRole: "operator" },
            { routeId: "/artifacts", label: "Artifacts", icon: "file-code", category: "Bridge", requiredRole: "operator" },
            { routeId: "/workflows", label: "Workflows", icon: "git-branch", category: "Bridge", requiredRole: "operator" },
            { routeId: "/system/providers", label: "Providers", icon: "brain", category: "Admin", requiredRole: "operator" },
          ],
        },
      });
    }
    if (path.endsWith("/navigation-plan")) {
      return fulfillJson(route, {
        schema_version: "1.0.0",
        generated_at: "2026-04-29T00:00:00Z",
        entries: [],
      });
    }
    if (path === "/api/system/ux/workbench") {
      return fulfillJson(route, EMPTY_SURFACE);
    }
    if (path.includes("/heap/blocks")) {
      return fulfillJson(route, {
        schemaVersion: "1.0.0",
        generatedAt: "2026-04-29T00:00:00Z",
        count: 0,
        hasMore: false,
        items: [],
      });
    }
    if (path.includes("/providers") || path.includes("/provider-")) {
      return fulfillJson(route, {
        schema_version: "1.0.0",
        generated_at: "2026-04-29T00:00:00Z",
        providers: [],
        authBindings: [],
        runtimeHosts: [],
        discoveryRecords: [],
      });
    }

    return fulfillJson(route, {
      schema_version: "1.0.0",
      generated_at: "2026-04-29T00:00:00Z",
      items: [],
    });
  });
}

async function fetchInventory(page: Page, path: string) {
  return page.evaluate(async (urlPath) => {
    const response = await fetch(urlPath);
    if (!response.ok) {
      throw new Error(`${urlPath} returned ${response.status}`);
    }
    return response.json();
  }, path);
}

test.beforeEach(async ({ page }) => {
  await installObservabilityMocks(page);
});

test("graph inventory smoke exposes non-empty graph surfaces or explicit fallbacks", async ({ page }) => {
  await page.goto("/");
  const inventory = await fetchInventory(page, "/api/system/ux/graph-surface-inventory");

  expect(inventory.authority_mode).toBe("recommendation_only");
  expect(inventory.graph_surfaces.length).toBeGreaterThanOrEqual(13);

  const surfaces = new Map(inventory.graph_surfaces.map((surface: any) => [surface.surface_id, surface]));
  for (const surfaceId of [
    "graph.contributions.full",
    "graph.capability.system",
    "graph.workflow.flow_graph",
    "graph.execution_canvas.spatial_plane",
    "graph.heap.detail_relations",
  ]) {
    const surface = surfaces.get(surfaceId) as any;
    expect(surface, `${surfaceId} must exist`).toBeTruthy();
    expect(surface.render_status).toBeTruthy();
    expect(surface.fetch_status).toBeTruthy();
    const visibleCountsExposed =
      typeof surface.visible_node_count === "number" || typeof surface.visible_edge_count === "number";
    const hasFallbackOrGap = Boolean(surface.fallback_reason || surface.known_gap);
    expect(visibleCountsExposed || hasFallbackOrGap, `${surfaceId} must expose counts or fallback reason`).toBeTruthy();
  }

  const gapIds = new Set(inventory.known_gaps.map((gap: any) => gap.id));
  expect(gapIds.has("graph.surface.shared_health_contract")).toBeTruthy();
  expect(gapIds.has("graph.execution_canvas.topology_drift")).toBeTruthy();
});

test("overlay inventory smoke exposes lifecycle and authority metadata", async ({ page }) => {
  await page.goto("/");
  const inventory = await fetchInventory(page, "/api/system/ux/overlay-surface-inventory");

  expect(inventory.authority_mode).toBe("recommendation_only");
  expect(inventory.overlay_surfaces.length).toBeGreaterThanOrEqual(20);

  const surfaces = new Map(inventory.overlay_surfaces.map((surface: any) => [surface.surface_id, surface]));
  for (const surfaceId of [
    "overlay.heap.detail_modal",
    "overlay.heap.chat_panel",
    "overlay.heap.comment_sidebar",
    "overlay.heap.steward_gate",
    "overlay.system.provider_detail_sheet",
    "overlay.shared.confirmation",
    "overlay.artifacts.workflow_inspector",
  ]) {
    const surface = surfaces.get(surfaceId) as any;
    expect(surface, `${surfaceId} must exist`).toBeTruthy();
    expect(surface.close_mechanisms.length).toBeGreaterThan(0);
    expect(surface.focus_policy).toBeTruthy();
    expect(surface.escape_policy).toBeTruthy();
    expect(surface.authority_class).toBeTruthy();
  }

  expect((surfaces.get("overlay.heap.chat_panel") as any).known_collision).toMatch(/create controls/);
  expect((surfaces.get("overlay.system.provider_detail_sheet") as any).authority_class).toBe("operator_only");
});

test("route IA smoke keeps settings absent and A2UI candidates explicit", async ({ page }) => {
  await page.goto("/");
  const inventory = await fetchInventory(page, "/api/system/ux/route-ia-inventory");

  expect(inventory.authority_mode).toBe("recommendation_only");
  expect(inventory.settings_absence_contract.route_id).toBe("/settings");
  expect(inventory.settings_absence_contract.global_settings_page_allowed_this_stage).toBe(false);

  const routes = new Map(inventory.routes.map((route: any) => [route.route_id, route]));
  expect((routes.get("/settings") as any).readiness_status).toBe("missing");
  expect((routes.get("/system/providers") as any).operator_boundary).toBe("operator_only");
  expect((routes.get("/discovery") as any).visible_in_nav).toBe(true);
  expect((routes.get("/discovery") as any).readiness_status).toBe("under_construction");
  expect((routes.get("/spaces/:id?tab=overview") as any).detail_tabs).toEqual(
    expect.arrayContaining(["overview", "routing", "agents", "history"]),
  );

  const hiddenA2uiCandidates = inventory.routes.filter(
    (route: any) => route.route_class === "a2ui_candidate" && route.visible_in_nav === false,
  );
  expect(hiddenA2uiCandidates.length).toBeGreaterThanOrEqual(9);
  for (const route of hiddenA2uiCandidates) {
    expect(route.a2ui_fallback_allowed, `${route.route_id} must declare A2UI fallback allowance`).toBe(true);
  }
});
