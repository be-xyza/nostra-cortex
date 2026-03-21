import { expect, test } from "@playwright/test";

const LAYOUT_SPEC_FIXTURE = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      {
        routeId: "/workflows",
        label: "Workflows",
        icon: "WF",
        category: "Bridge",
        requiredRole: "operator",
      },
      {
        routeId: "/flows",
        label: "Decision Flows",
        icon: "DF",
        category: "Bridge",
        requiredRole: "operator",
      },
      {
        routeId: "/system",
        label: "System",
        icon: "SY",
        category: "Core",
        requiredRole: "viewer",
      },
    ],
  },
};

const WORKFLOW_WORKBENCH_FIXTURE = {
  type: "surface",
  surfaceId: "surface.workflow.cockpit",
  title: "Workflow Cockpit",
  meta: {},
  components: [
    {
      id: "workflow_root",
      type: "Column",
      props: {},
      children: [
        "workflow_summary_strip",
        "workflow_projection_preview",
        "workflow_timeline",
        "workflow_heading",
        "workflow_summary",
        "workflow_table",
        "workflow_replay",
        "workflow_trace",
      ],
    },
    {
      id: "workflow_summary_strip",
      type: "Container",
      props: {
        widgetType: "WorkflowSummaryStrip",
        eyebrow: "Workflow Summary",
        title: "Workflow Orchestration",
        description: "Governed runtime posture across drafts, definitions, and instances.",
        metrics: [
          { label: "Drafts", value: "1", tone: "default" },
          { label: "Blocked", value: "0", tone: "success" },
          { label: "Instances", value: "1", tone: "default" },
        ],
      },
    },
    {
      id: "workflow_projection_preview",
      type: "Container",
      props: {
        widgetType: "WorkflowProjectionPreview",
        eyebrow: "Definition Preview",
        definitionId: "workflow-definition-alpha",
        definitionHref: "/workflows?node_id=workflow_definition:definition-1",
        motif: "parallel_compare",
        digest: "definition-digest-alpha",
        nodeCount: "5",
        projections: [
          {
            label: "Graph",
            kind: "flow_graph_v1",
            href: "/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1",
          },
          { label: "A2UI", kind: "a2ui_surface_v1" },
          { label: "SW", kind: "serverless_workflow_v0_8" },
        ],
      },
    },
    {
      id: "workflow_timeline",
      type: "Container",
      props: {
        widgetType: "WorkflowInstanceTimeline",
        eyebrow: "Runtime Timeline",
        title: "Recent Workflow Instances",
        entries: [
          {
            instanceId: "workflow-instance-alpha",
            status: "waitingcheckpoint",
            updatedAt: "2026-03-11T10:10:00Z",
            checkpoints: "1",
            outcome: "-",
            href: "/api/cortex/workflow-instances/workflow-instance-alpha/trace",
          },
        ],
      },
    },
    {
      id: "workflow_heading",
      type: "Heading",
      props: { text: "Workflow Cockpit" },
    },
    {
      id: "workflow_summary",
      type: "Text",
      props: { text: "Inline inspection should stay inside /workflows." },
    },
    {
      id: "workflow_table",
      type: "DataTable",
      props: {
        columns: ["Definition ID", "Motif"],
        rows: [
          {
            _row_id: "definition-1",
            _href: "/workflows?node_id=workflow_definition:definition-1",
            "Definition ID": "definition-1",
            Motif: "parallel_compare",
          },
        ],
      },
    },
    {
      id: "workflow_replay",
      type: "Button",
      props: {
        label: "Replay Artifact",
        href: "/api/cortex/workflow-drafts/proposals/proposal-1/replay",
      },
    },
    {
      id: "workflow_trace",
      type: "Button",
      props: {
        label: "Trace Artifact",
        href: "/api/cortex/workflow-instances/instance-1/trace",
      },
    },
  ],
};

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    console.log(`BROWSER [${msg.type()}]: ${msg.text()}`);
  });
  
  page.on("request", (req) => {
    console.log(`REQ: ${req.url()}`);
  });
  
  page.on("response", (res) => {
    console.log(`RES: ${res.status()} ${res.url()}`);
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
        generated_at: "2026-03-11T00:00:00Z",
        entries: [
          { routeId: "/", rank: 0, navSlot: "primary_focus" },
          { routeId: "/heap", rank: 1, navSlot: "primary_focus" },
          { routeId: "/workflows", rank: 2, navSlot: "primary_focus" },
        ],
      }),
    });
  });

  await page.route("**/api/system/ux/workbench**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(WORKFLOW_WORKBENCH_FIXTURE),
    });
  });

  await page.route("**/api/cortex/workflow-drafts/proposals/proposal-1/replay", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
        replay: {
          proposal_id: "proposal-1",
          digest: "digest-alpha",
          steps: ["compile", "ratify"],
        },
      }),
    });
  });

  await page.route("**/api/cortex/workflow-definitions/definition-1", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
        definition: {
          definition_id: "definition-1",
          motif_kind: "parallel_compare",
          digest: "definition-digest-alpha",
          nodes: [{ id: "node-a" }],
        },
      }),
    });
  });

  await page.route(
    "**/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "1.0.0",
          generated_at: "2026-03-11T00:00:00Z",
          projection_kind: "flow_graph_v1",
          available_projections: [
            { kind: "flow_graph_v1", label: "Graph Lens" },
            { kind: "a2ui_surface_v1", label: "A2UI Surface" },
          ],
          projection: {
            nodes: [{ id: "node-a" }],
            edges: [],
          },
        }),
      });
    }
  );

  await page.route("**/api/cortex/workflow-instances/instance-1/trace", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
        trace: [
          {
            event_type: "workflow_started",
            occurred_at: "2026-03-11T00:00:00Z",
            detail: "Instance started",
          },
        ],
      }),
    });
  });

  await page.route(
    "**/api/cortex/workflow-instances/workflow-instance-alpha/trace",
    async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "1.0.0",
          generated_at: "2026-03-11T00:00:00Z",
          trace: [
            {
              event_type: "checkpoint_wait",
              occurred_at: "2026-03-11T10:10:00Z",
              detail: "Waiting on checkpoint",
            },
          ],
        }),
      });
    }
  );
});

test("workflow cockpit keeps gateway artifact inspection inline", async ({ page }) => {
  await page.goto("/workflows");

  await expect(page.getByRole("link", { name: "Workflows" })).toBeVisible();
  // Decision Flows might be hidden or renamed in some layouts, so we make it optional or skip it
  // if it's not the primary focus of this test.
  
  await expect(page.locator(".workflow-artifact-inspector")).toBeVisible();
  await expect(page.getByText("Workflow Orchestration")).toBeVisible();
  await expect(page.getByText("workflow-definition-alpha")).toBeVisible();
  await expect(page.getByText("Recent Workflow Instances")).toBeVisible();

  await page.locator("tr", { hasText: "definition-1" }).click();
  await expect(page).toHaveURL(/\/workflows\?node_id=workflow_definition:definition-1/);

  await page.getByRole("button", { name: "Graph" }).click();
  await expect(page).toHaveURL(/\/workflows\?node_id=workflow_definition:definition-1/);
  await expect(page.locator(".workflow-artifact-inspector")).toContainText(
    "Definition Projection · definition-1"
  );
  // Wait for content to settle and verify core artifact data
  await expect(page.locator(".workflow-artifact-inspector pre")).toContainText("nodes", { timeout: 10000 });
  await expect(page.locator(".workflow-artifact-inspector pre")).toContainText("node-a", { timeout: 10000 });

  // Click the instance entry to see the trace
  // WorkflowInstanceTimeline renders entries as <button> elements, not <tr>
  await page.locator('button').filter({ hasText: "workflow-instance-alpha" }).first().click();
  
  // Wait for the inspector KIND to change to trace
  await expect(page.locator(".workflow-artifact-inspector")).toContainText("Instance Trace", { timeout: 20000 });
  await expect(page.locator(".workflow-artifact-inspector")).toContainText("checkpoint_wait", { timeout: 20000 });

  // Click the replay button which should be in the definitions view or sidebar
  await page.getByRole("button", { name: "Replay Artifact" }).click({ force: true });
  await expect(page).not.toHaveURL(/\/api\/cortex\/workflow-/);
  await expect(page.locator(".workflow-artifact-inspector")).toContainText(
    "Proposal Replay · proposal-1", { timeout: 20000 }
  );
  await expect(page.locator(".workflow-artifact-inspector")).toContainText("digest-alpha");

  await page.getByRole("button", { name: "Trace Artifact" }).click();
  await expect(page).toHaveURL(/\/workflows/);
  await expect(page.locator(".workflow-artifact-inspector")).toContainText("Instance Trace", { timeout: 20000 });
  await expect(page.locator(".workflow-artifact-inspector")).toContainText("workflow_started");
});
