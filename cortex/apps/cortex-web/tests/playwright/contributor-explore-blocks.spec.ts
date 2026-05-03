import { expect, test, type Page, type Route } from "@playwright/test";

const COMMON_HEADERS = { "Content-Type": "application/json" };

const SESSION = {
  schemaVersion: "1.0.0",
  generatedAt: "2026-05-02T00:00:00.000Z",
  principal: "viewer-principal",
  sessionId: "viewer-session",
  identityVerified: false,
  identitySource: "read_fallback_viewer",
  authMode: "read_fallback",
  grantedRoles: ["viewer"],
  activeRole: "viewer",
  globalClaims: [],
  spaceGrants: [],
  allowRoleSwitch: false,
  allowUnverifiedRoleHeader: false,
};

const LAYOUT_SPEC = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      { routeId: "/explore", label: "Explore", icon: "compass", category: "Platform", requiredRole: "viewer" },
      { routeId: "/artifacts", label: "Artifacts", icon: "file-code", category: "Ops", requiredRole: "viewer" },
    ],
  },
};

const HEAP_BLOCKS = {
  schemaVersion: "1.0.0",
  generatedAt: "2026-05-02T00:00:00.000Z",
  count: 4,
  hasMore: false,
  items: [
    block("usage-1", "usage_report", "usage_report block", "usage report block", "2026-03-22T07:14:00Z"),
    block("agent-1", "agent_execution_record", "agent_execution_record block", "agent execution record block", "2026-03-22T07:13:00Z"),
    block("proposal-1", "self_optimization_proposal", "self_optimization_proposal block", "self optimization proposal block", "2026-03-22T07:12:00Z"),
    block("evidence-1", "note", "Eudaemon Alpha authorized publication proof", "Operator-approved evidence note resolved a verified operator principal and emitted exactly one bounded rich-text.", "2026-04-29T00:15:00Z", ["evidence"]),
  ],
};

function block(
  artifactId: string,
  blockType: string,
  title: string,
  text: string,
  updatedAt: string,
  tags: string[] = [],
) {
  return {
    projection: {
      artifactId,
      title,
      blockType,
      updatedAt,
      emittedAt: updatedAt,
      tags,
      mentionsInline: [],
      pageLinks: [],
    },
    surfaceJson: {
      payload_type: "structured_data",
      text,
    },
    warnings: [],
  };
}

async function fulfillJson(route: Route, body: unknown) {
  await route.fulfill({
    status: 200,
    headers: COMMON_HEADERS,
    body: JSON.stringify(body),
  });
}

async function installMocks(page: Page) {
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;

    if (path === "/api/system/session") return fulfillJson(route, SESSION);
    if (path === "/api/cortex/layout/spec") return fulfillJson(route, LAYOUT_SPEC);
    if (path.endsWith("/navigation-plan")) {
      return fulfillJson(route, {
        schema_version: "1.0.0",
        generated_at: "2026-05-02T00:00:00.000Z",
        entries: [],
      });
    }
    if (path.includes("/heap/blocks")) return fulfillJson(route, HEAP_BLOCKS);
    if (path.includes("/action-plan")) {
      const actions = [
        { id: "regenerate", capabilityId: "cap.refresh", zone: "heap_selection_bar", label: "Regen", icon: "refresh-cw", kind: "command", action: "regenerate", priority: 1, group: "secondary", visible: true, enabled: true },
        { id: "refine_selection", capabilityId: "cap.refine", zone: "heap_selection_bar", label: "Refine Selection", icon: "sparkles", kind: "command", action: "refine", priority: 2, group: "secondary", visible: true, enabled: true },
        { id: "synthesize", capabilityId: "cap.synth", zone: "heap_selection_bar", label: "Synthesize", icon: "wand2", kind: "command", action: "synthesize", priority: 4, group: "secondary", visible: true, enabled: false, disabledReason: "Requires at least 3 blocks selected." },
      ];
      return fulfillJson(route, {
        schemaVersion: "1.0.0",
        zones: [
          {
            zone: "heap_selection_bar",
            actions,
          },
        ],
      });
    }

    return fulfillJson(route, { schema_version: "1.0.0", generated_at: "2026-05-02T00:00:00.000Z", items: [] });
  });
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("cortex.shell.actor.id", "viewer-session");
    window.localStorage.setItem("cortex.shell.actor.role", "viewer");
  });
  await installMocks(page);
});

test("Explore defaults to contributor-friendly updates instead of raw heap record titles", async ({ page }) => {
  await page.goto("/explore");

  await expect(page.getByText("Relevant updates")).toBeVisible();
  await expect(page.getByText("Showing recent updates, proposals, evidence, and agent activity for this Space.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Recent activity summaries" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Recent agent work" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Suggested improvements" })).toBeVisible();
  await expect(page.getByText("Eudaemon Alpha authorized publication proof")).toBeVisible();

  await expect(page.getByText("usage_report block")).toHaveCount(0);
  await expect(page.getByText("self_optimization_proposal block")).toHaveCount(0);
  await expect(page.getByText("agent_execution_record block")).toHaveCount(0);
  await expect(page.getByText("All Blocks")).toBeVisible();
});

test("selection bar uses contributor language and hides publish for viewer sessions", async ({ page }) => {
  await page.goto("/explore");
  await page.getByText("Eudaemon Alpha authorized publication proof").click();

  await expect(page.locator(".heap-action-bar")).toBeVisible();
  await expect(page.locator(".heap-action-bar")).toContainText("Refresh");
  await expect(page.locator(".heap-action-bar")).toContainText("Improve summary");
  await expect(page.locator(".heap-action-bar")).toContainText("Summarize selected");
  await expect(page.locator(".heap-action-bar")).not.toContainText("Publish");
});
