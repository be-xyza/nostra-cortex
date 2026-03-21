import { expect, test } from "@playwright/test";

const LAYOUT_SPEC_FIXTURE = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      {
        routeId: "/contributions",
        label: "Contributions",
        icon: "CT",
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

const CONTRIBUTIONS_SURFACE_FIXTURE = {
  type: "surface",
  surfaceId: "surface.contributions.cockpit",
  title: "Contributions",
  meta: {},
  components: [
    {
      id: "contributions_root",
      type: "Column",
      props: {},
      children: [
        "contributions_title",
        "contributions_summary",
        "contributions_runs",
      ],
    },
    {
      id: "contributions_title",
      type: "Heading",
      props: { text: "Contributions Cockpit" },
    },
    {
      id: "contributions_summary",
      type: "Text",
      props: { text: "Steward-facing lifecycle operations for governed contribution work." },
    },
    {
      id: "contributions_runs",
      type: "DataTable",
      props: {
        columns: ["Run ID", "Status"],
        rows: [
          {
            _row_id: "run-123",
            _href: "/contributions?node_id=agent_run:run-123",
            "Run ID": "run-123",
            Status: "waiting_approval",
          },
        ],
      },
    },
  ],
};

test.beforeEach(async ({ page }) => {
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
        generated_at: "2026-03-20T00:00:00Z",
        entries: [
          { routeId: "/contributions", rank: 0, navSlot: "primary_focus" },
          { routeId: "/system", rank: 1, navSlot: "primary_focus" },
        ],
      }),
    });
  });

  await page.route("**/api/system/ux/workbench**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(CONTRIBUTIONS_SURFACE_FIXTURE),
    });
  });

  await page.route("**/api/system/agents/runs?space_id=**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([
        {
          runId: "run-123",
          workflowId: "wf-123",
          spaceId: "space-alpha",
          contributionId: "proposal-alpha",
          agentId: "agent:eudaemon-alpha-01",
          status: "waiting_approval",
          startedAt: "2026-03-20T00:00:00Z",
          updatedAt: "2026-03-20T00:01:00Z",
          authorityLevel: "operator",
          requiresReview: true,
        },
      ]),
    });
  });

  await page.route("**/api/system/agents/runs/*/run-123", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        runId: "run-123",
        workflowId: "wf-123",
          spaceId: "meta",
        contributionId: "proposal-alpha",
        status: "waiting_approval",
        startedAt: "2026-03-20T00:00:00Z",
        updatedAt: "2026-03-20T00:01:00Z",
        streamChannel: "/ws/agent-runs/run-123",
        surfaceUpdate: {
          id: "root",
          type: "Column",
          componentProperties: { Column: {} },
          children: {
            explicitList: [
              {
                id: "heading",
                type: "Heading",
                componentProperties: { Heading: { text: "Approval Gate" } },
              },
              {
                id: "approval",
                type: "ApprovalControls",
                componentProperties: {
                  ApprovalControls: {
                    runId: "run-123",
                    spaceId: "meta",
                    decisionRef: "DEC-123",
                  },
                },
              },
            ],
          },
        },
        approvalTimeoutSeconds: 3600,
        events: [],
      }),
    });
  });

  await page.route("**/api/kg/spaces/*/agents/contributions/run-123", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        runId: "run-123",
        workflowId: "wf-123",
        spaceId: "meta",
        contributionId: "proposal-alpha",
        status: "waiting_approval",
        startedAt: "2026-03-20T00:00:00Z",
        updatedAt: "2026-03-20T00:01:00Z",
        streamChannel: "/ws/agent-runs/run-123",
        surfaceUpdate: {
          id: "root",
          type: "Column",
          componentProperties: { Column: {} },
          children: {
            explicitList: [
              {
                id: "heading",
                type: "Heading",
                componentProperties: { Heading: { text: "Approval Gate" } },
              },
              {
                id: "approval",
                type: "ApprovalControls",
                componentProperties: {
                  ApprovalControls: {
                    runId: "run-123",
                    spaceId: "meta",
                    decisionRef: "DEC-123",
                  },
                },
              },
            ],
          },
        },
        approvalTimeoutSeconds: 3600,
        events: [],
      }),
    });
  });

  await page.route("**/api/kg/spaces/*/contribution-graph/runs?limit=10", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([
        {
          runId: "graph-run-1",
          mode: "simulate",
          status: "success",
          startedAt: "2026-03-20T00:00:00Z",
        },
      ]),
    });
  });

  await page.route("**/api/kg/spaces/*/contribution-graph/blast-radius?contributionId=proposal-alpha", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        contributionId: "proposal-alpha",
        dependsOn: ["spec-alpha"],
        dependedBy: ["plan-alpha"],
        invalidates: [],
        invalidatedBy: [],
        supersedes: [],
        supersededBy: [],
        references: ["initiative-132"],
        referencedBy: [],
      }),
    });
  });

  await page.route("**/api/kg/spaces/*/agents/contributions/run-123/approval", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        accepted: true,
        runId: "run-123",
        status: "approval_recorded",
      }),
    });
  });

  await page.route("**/api/kg/spaces/*/contribution-graph/steward-packet/export", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        packetPath: "/tmp/steward-packet.md",
        goal: "stable-cortex-domain",
        fromVersion: "v0.1.0",
        toVersion: "v0.2.0",
      }),
    });
  });
});

test("contributions cockpit surfaces live run detail and steward tools", async ({ page }) => {
  await page.goto("/contributions?node_id=agent_run:run-123");
  const livePane = page.getByRole("region", { name: "Contributions Live Execution" });
  const summaryPane = page.getByRole("region", { name: "Contributions Summary and History" });

  await expect(
    page.getByRole("heading", { name: "Contributions Cockpit", level: 1 })
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Contributions Summary and History" })
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Contributions Steward Tools" })
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Contributions Live Execution" })
  ).toBeVisible();
  await expect(summaryPane.getByText("Contribution Graph Runs")).toBeVisible();
  await expect(summaryPane.getByText("graph-run-1")).toBeVisible();
  await expect(summaryPane.getByText("Agent Run History")).toBeVisible();
  await expect(
    summaryPane.getByRole("button", { name: /run-123 waiting_approval/i })
  ).toBeVisible();
  await expect(
    summaryPane.getByText("under construction", { exact: false })
  ).toHaveCount(0);
  await expect(page.getByText("run-123").first()).toBeVisible();
  await expect(page.getByRole("heading", { name: "Approval Gate" }).first()).toBeVisible();
  await expect(livePane.getByTestId("approval-controls").first()).toBeVisible();
  await expect(
    livePane.getByRole("button", { name: "Approve contribution changes" }).first()
  ).toBeVisible();
  await expect(page.getByText("proposal-alpha").first()).toBeVisible();
});

test("contributions cockpit keeps contribution focus passive until the operator launches live execution", async ({ page }) => {
  let launchCount = 0;

  await page.route("**/api/kg/spaces/*/agents/contributions", async (route) => {
    launchCount += 1;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        runId: "run-456",
        startedAt: "2026-03-20T00:02:00Z",
        status: "waiting_approval",
        streamChannel: "/ws/agent-runs/run-456",
        runtimeMode: "governed",
      }),
    });
  });

  await page.route("**/api/kg/spaces/*/agents/contributions/run-456", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        runId: "run-456",
        workflowId: "wf-456",
        spaceId: "meta",
        contributionId: "proposal-alpha",
        status: "waiting_approval",
        startedAt: "2026-03-20T00:02:00Z",
        updatedAt: "2026-03-20T00:03:00Z",
        streamChannel: "/ws/agent-runs/run-456",
        surfaceUpdate: {
          id: "root",
          type: "Column",
          componentProperties: { Column: {} },
          children: {
            explicitList: [
              {
                id: "heading",
                type: "Heading",
                componentProperties: { Heading: { text: "Approval Gate" } },
              },
            ],
          },
        },
        approvalTimeoutSeconds: 3600,
        events: [],
      }),
    });
  });

  await page.goto("/contributions?node_id=contribution:proposal-alpha");
  const stewardPane = page.getByRole("region", { name: "Contributions Steward Tools" });
  await expect(stewardPane.getByText("Start from contribution focus")).toBeVisible();
  await expect(page.getByText("proposal-alpha").first()).toBeVisible();
  const focusMap = page.getByRole("region", { name: "Contribution Focus Map" });
  await expect(page.getByText("Depends On").first()).toBeVisible();
  await expect(focusMap).toBeVisible();
  await expect(focusMap.getByRole("button", { name: "Focus relation spec-alpha" })).toBeVisible();

  await page.goto("/contributions?contribution_id=proposal-alpha");
  await expect(stewardPane.getByText("No graph run selected yet")).toBeVisible();
  await expect(stewardPane.getByText("No live agent run selected yet")).toBeVisible();
  await expect(page.getByText("A2UI Substrate Idle")).toBeVisible();
  await expect(page.getByRole("button", { name: "Launch Live Run" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Approval Gate" })).toHaveCount(0);
  await expect.poll(() => launchCount).toBe(0);

  await page.getByRole("button", { name: "Launch Live Run" }).click();

  await expect.poll(() => launchCount).toBe(1);
  await expect(page.getByRole("heading", { name: "Approval Gate" }).first()).toBeVisible();
});
