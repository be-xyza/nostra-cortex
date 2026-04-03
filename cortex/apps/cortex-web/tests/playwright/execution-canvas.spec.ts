import { expect, test, Page } from "@playwright/test";

const LAYOUT_SPEC_FIXTURE = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      {
        routeId: "/labs",
        label: "Labs",
        icon: "LB",
        category: "Core",
        requiredRole: "operator",
      },
    ],
  },
};

function labsWorkbenchFixture() {
  return {
    type: "surface",
    surfaceId: "surface.workbench.labs",
    title: "Labs",
    meta: {},
    components: [
      {
        id: "labs_root",
        type: "Column",
        props: {},
        children: ["labs_title", "space_studio", "execution_canvas", "open_execution_canvas"],
      },
      {
        id: "labs_title",
        type: "Heading",
        props: { text: "Labs" },
      },
      {
        id: "space_studio",
        type: "Text",
        props: { text: "Space Studio" },
      },
      {
        id: "execution_canvas",
        type: "Text",
        props: { text: "Execution Canvas" },
      },
      {
        id: "open_execution_canvas",
        type: "Button",
        props: { label: "Open Execution Canvas", href: "/labs/execution-canvas" },
      },
    ],
  };
}

function executionCanvasFixture(workflowBacked: boolean) {
  return {
    type: "surface",
    surfaceId: "surface.execution.canvas",
    title: "Execution Canvas",
    meta: {},
    components: [
      {
        id: "root",
        type: "Column",
        props: {},
        children: ["title", "plane"],
      },
      {
        id: "title",
        type: "Heading",
        props: { text: workflowBacked ? "Execution Canvas Workflow" : "Execution Canvas" },
      },
      {
        id: "plane",
        type: "SpatialPlane",
        props: {
          plane_id: workflowBacked ? "workflow-plane" : "labs-plane",
          surface_class: "execution",
          focus_bounds: { x: 0, y: 0, w: 1200, h: 720 },
          layout_ref: workflowBacked
            ? {
                space_id: "space-1",
                view_spec_id: "viewspec-1",
                workflow_id: "workflow-alpha",
                graph_hash: "graph-alpha",
              }
            : {
                space_id: "space-1",
                view_spec_id: "viewspec-1",
              },
          commands: [
            {
              op: "create_shape",
              shape: {
                id: "lane-1",
                kind: "group",
                x: 40,
                y: 40,
                w: 1100,
                h: 360,
                label: "Execution lane",
                member_ids: ["node-input", "node-tool", "node-output"],
                collapsed: false,
              },
            },
            {
              op: "create_shape",
              shape: {
                id: "node-input",
                kind: "node",
                node_class: "input",
                status: "idle",
                x: 120,
                y: 128,
                text: "Intent",
                ports: [{ id: "out", side: "right", direction: "out", label: "intent" }],
              },
            },
            {
              op: "create_shape",
              shape: {
                id: "node-tool",
                kind: "node",
                node_class: "tool",
                status: "running",
                x: 432,
                y: 128,
                text: "Worker Tool",
                ports: [
                  { id: "in", side: "left", direction: "in", label: "context" },
                  { id: "out", side: "right", direction: "out", label: "result" },
                ],
              },
            },
            {
              op: "create_shape",
              shape: {
                id: "node-output",
                kind: "node",
                node_class: "output",
                status: "blocked",
                x: 800,
                y: 128,
                text: "Projection",
                ports: [{ id: "in", side: "left", direction: "in", label: "surface" }],
              },
            },
            {
              op: "create_shape",
              shape: {
                id: workflowBacked ? "edge:node-tool:out:node-output:in" : "edge-worker-output",
                kind: "edge",
                edge_class: "control",
                x: 660,
                y: 196,
                from_shape_id: "node-tool",
                to_shape_id: "node-output",
                from_port_id: "out",
                to_port_id: "in",
                text: "Render",
              },
            },
            {
              op: "set_selection",
              shape_ids: ["node-tool"],
            },
          ],
        },
      },
    ],
  };
}

async function installBaseMocks(page: Page) {
  await page.addInitScript(() => {
    window.localStorage.setItem("cortex.feature.VITE_A2UI_SPATIAL_PLANE", "1");
    window.localStorage.setItem("cortex.feature.VITE_A2UI_TLDRAW_EXPERIMENT", "0");
  });

  await page.route("**/api/cortex/layout/spec", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(LAYOUT_SPEC_FIXTURE),
    });
  });

  await page.route("**/api/spaces/*/navigation-plan**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-04-02T00:00:00Z",
        entries: [
          { routeId: "/labs", rank: 0, navSlot: "labs" },
          { routeId: "/labs/execution-canvas", rank: 1, navSlot: "labs" },
        ],
      }),
    });
  });
}

async function dragBy(page: Page, selector: string, dx: number, dy: number) {
  const target = page.locator(selector);
  const box = await target.boundingBox();
  if (!box) {
    throw new Error(`Missing bounding box for ${selector}`);
  }
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 8 });
  await page.mouse.up();
}

test("labs index shows the execution canvas entry", async ({ page }) => {
  await installBaseMocks(page);
  await page.route("**/api/system/ux/workbench**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(labsWorkbenchFixture()),
    });
  });

  await page.goto("/labs");

  await expect(page.getByText("Execution Canvas", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Open Execution Canvas" }).click();
  await expect(page).toHaveURL(/\/labs\/execution-canvas$/);
});

test("execution canvas saves layout and restores it after reload with svg fallback authoring", async ({ page }) => {
  await installBaseMocks(page);
  let savedLayout: any = null;

  await page.route("**/api/system/ux/workbench**", async (route) => {
    const url = new URL(route.request().url());
    const targetRoute = url.searchParams.get("route");
    const body = targetRoute === "/labs"
      ? labsWorkbenchFixture()
      : executionCanvasFixture(false);
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });

  await page.route("**/api/cortex/viewspecs/spatial/layouts/space-1/viewspec-1", async (route) => {
    if (route.request().method() === "GET") {
      if (!savedLayout) {
        await route.fulfill({ status: 404, contentType: "application/json", body: JSON.stringify({ errorCode: "SPATIAL_LAYOUT_NOT_FOUND" }) });
        return;
      }
      await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ accepted: true, layout: savedLayout }) });
      return;
    }

    savedLayout = JSON.parse(route.request().postData() ?? "{}");
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ accepted: true, layout: savedLayout }) });
  });

  await page.goto("/labs/execution-canvas");

  await expect(page.getByText(/renderer=svg-fallback/i)).toBeVisible();
  await dragBy(page, ".a2ui-spatial-plane__node--tool", 96, 42);
  await page.getByRole("button", { name: "Save Layout" }).click();
  await expect(page.getByText(/layout=saved/i)).toBeVisible();
  expect(savedLayout?.layout?.shape_positions?.["node-tool"]?.x).toBeGreaterThan(432);

  await page.reload();
  await expect(page.getByText(/layout=loaded|layout=saved/i)).toBeVisible();
  const toolNode = page.locator(".a2ui-spatial-plane__node--tool");
  await expect(toolNode).toBeVisible();
});

test("workflow-backed execution canvas blocks topology edits but still allows layout saves", async ({ page }) => {
  await installBaseMocks(page);
  let savedLayout: any = null;

  await page.route("**/api/system/ux/workbench**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(executionCanvasFixture(true)),
    });
  });

  await page.route("**/api/cortex/viewspecs/spatial/layouts/space-1/viewspec-1", async (route) => {
    if (route.request().method() === "GET") {
      if (!savedLayout) {
        await route.fulfill({ status: 404, contentType: "application/json", body: JSON.stringify({ errorCode: "SPATIAL_LAYOUT_NOT_FOUND" }) });
        return;
      }
      await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ accepted: true, layout: savedLayout }) });
      return;
    }

    savedLayout = JSON.parse(route.request().postData() ?? "{}");
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ accepted: true, layout: savedLayout }) });
  });

  await page.goto("/labs/execution-canvas");

  await dragBy(page, ".a2ui-spatial-plane__node--tool", 80, 24);
  await page.locator(".a2ui-spatial-plane__port").first().click();
  await expect(page.getByText(/workflow-backed mode keeps topology read-only/i)).toBeVisible();
  await expect(page.getByRole("button", { name: "Delete Selection" })).toBeDisabled();

  await page.getByRole("button", { name: "Save Layout" }).click();
  await expect(page.getByText(/layout=saved/i)).toBeVisible();
  expect(savedLayout?.lineage?.workflow_id).toBe("workflow-alpha");
  expect(savedLayout?.layout?.shape_positions?.["node-tool"]?.x).toBeGreaterThan(432);
});
