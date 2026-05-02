import { expect, test, type Page, type Route } from "@playwright/test";

const COMMON_HEADERS = { "Content-Type": "application/json" };

const READ_ONLY_SESSION = {
  schemaVersion: "1.0.0",
  generatedAt: "2026-05-02T00:00:00.000Z",
  principal: "viewer-principal",
  sessionId: "read-only-session",
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
      { routeId: "/workflows", label: "Flows", icon: "git-branch", category: "Execute", requiredRole: "operator" },
      { routeId: "/contributions", label: "Contributions", icon: "git-merge", category: "Ops", requiredRole: "operator" },
    ],
  },
};

async function fulfillJson(route: Route, body: unknown) {
  await route.fulfill({
    status: 200,
    headers: COMMON_HEADERS,
    body: JSON.stringify(body),
  });
}

async function installOperatorLoginMocks(page: Page) {
  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());
    const path = url.pathname;

    if (path === "/api/system/session") {
      return fulfillJson(route, READ_ONLY_SESSION);
    }
    if (path === "/api/system/whoami") {
      return fulfillJson(route, {
        schemaVersion: "1.0.0",
        generatedAt: "2026-05-02T00:00:00.000Z",
        principal: "viewer-principal",
        requestedRole: "viewer",
        effectiveRole: "viewer",
        claims: [],
        identityVerified: false,
        identitySource: "read_fallback_viewer",
      });
    }
    if (path === "/api/cortex/layout/spec") {
      return fulfillJson(route, LAYOUT_SPEC);
    }
    if (path.endsWith("/navigation-plan")) {
      return fulfillJson(route, {
        schema_version: "1.0.0",
        generated_at: "2026-05-02T00:00:00.000Z",
        entries: [],
      });
    }
    if (path.includes("/heap/blocks")) {
      return fulfillJson(route, {
        schemaVersion: "1.0.0",
        generatedAt: "2026-05-02T00:00:00.000Z",
        count: 0,
        hasMore: false,
        items: [],
      });
    }

    return fulfillJson(route, {
      schema_version: "1.0.0",
      generated_at: "2026-05-02T00:00:00.000Z",
      items: [],
    });
  });
}

test.beforeEach(async ({ page }) => {
  await installOperatorLoginMocks(page);
});

test("trusted read-only sessions place operator sign-in in the expanded authority lane", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.addInitScript(() => {
    window.localStorage.setItem("cortex.shell.nav.mode", "expanded");
  });

  await page.goto("/explore");

  const sidebar = page.getByRole("navigation", { name: "Global navigation" });
  await expect(sidebar.getByText("Operator access")).toBeVisible();
  await expect(sidebar.getByRole("button", { name: /sign in/i })).toBeVisible();
  await expect(page.getByText("Read-only observer mode")).toBeVisible();

  const observerDetailsSignIn = page
    .locator("main details")
    .getByRole("button", { name: /verify operator/i });
  await expect(observerDetailsSignIn).toHaveCount(0);
});

test("trusted read-only sessions keep sign-in in observer details when the rail is collapsed", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.addInitScript(() => {
    window.localStorage.setItem("cortex.shell.nav.mode", "rail");
  });

  await page.goto("/explore");

  await expect(page.getByRole("navigation", { name: "Global navigation" }).getByText("Operator access")).toHaveCount(0);
  const observer = page.locator("main details");
  await observer.locator("summary").click();
  await expect(observer.getByRole("button", { name: /verify operator/i })).toBeVisible();
});
