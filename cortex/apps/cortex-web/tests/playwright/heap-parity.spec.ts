import { expect, test } from "@playwright/test";

const LAYOUT_SPEC_FIXTURE = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      { routeId: "/system", label: "System", icon: "SY", category: "Core", requiredRole: "viewer" },
      { routeId: "/explore", label: "Explore", icon: "EX", category: "Core", requiredRole: "viewer" },
      { routeId: "/playground", label: "Playground", icon: "PG", category: "Core", requiredRole: "viewer" }
    ]
  }
};

const WHOAMI_FIXTURE = {
  schemaVersion: "1.0.0",
  principal: "local-user",
  effectiveRole: "steward",
  identityVerified: true
};

const NAVIGATION_PLAN_FIXTURE = {
  schemaVersion: "1.0.0",
  entries: [
    { capabilityId: "cap.heap", routeId: "/explore", label: "Explore", icon: "database", navSlot: "primary" },
    { capabilityId: "cap.system", routeId: "/system", label: "System", icon: "settings", navSlot: "primary" }
  ]
};

const ACTION_PLAN_FIXTURE = {
  schemaVersion: "1.0.0",
  zones: [
    {
      zone: "heap_page_bar",
      actions: [
        { id: "mock.create", label: "Create Block", icon: "plus", enabled: true, visible: true }
      ]
    },
    {
      zone: "heap_selection_bar",
      actions: [
        { id: "mock.refine", capabilityId: "cap.refine", label: "Refine Selection", icon: "sparkles", kind: "command", action: "refine", enabled: true, visible: true },
        { id: "mock.synth", capabilityId: "cap.synth", label: "Synthesize", icon: "wand2", kind: "command", action: "synthesize", enabled: true, visible: true }
      ]
    }
  ]
};

const HEAP_WORKBENCH_FIXTURE = {
  type: "surface",
  surfaceId: "surface.heap.parity",
  title: "Heap",
  meta: {},
  components: [
    {
      id: "heap_canvas",
      type: "Container",
      props: {
        widgetType: "HeapCanvas"
      },
      children: []
    }
  ]
};

const HEAP_BLOCKS_FIXTURE = {
  schemaVersion: "1.0.0",
  generatedAt: "2026-03-03T00:00:00Z",
  count: 3,
  hasMore: false,
  items: [
    {
      projection: {
        artifactId: "artifact-heap-parity-1",
        workspaceId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        blockId: "01ARZ3NDEKTSV4RRFFQ69G5FAW",
        title: "Heap Parity Card",
        blockType: "note",
        updatedAt: "2026-03-03T00:00:00Z",
        emittedAt: "2026-03-03T00:00:00Z",
        tags: ["architecture"],
        mentionsInline: ["gate_summary_siq_latest"],
        pageLinks: ["chart_summary_metrics"],
        attributes: { priority: "P0" },
      },
      surfaceJson: {
        payload_type: "rich_text",
        text: "Heap parity body\n- [x] Checked item\n- [ ] Unchecked item\n`inline code` testing.",
        behaviors: ["pinned"],
        version: "v1.0",
        phase: "Alpha",
        confidence: 85,
        authority_scope: "Local"
      },
      warnings: []
    },
    {
      projection: {
        artifactId: "gate_summary_siq_latest",
        workspaceId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        blockId: "01ARZ3NDEKTSV4RRFFQ69G5FB0",
        title: "SIQ Gate Summary (latest)",
        blockType: "gate_summary",
        updatedAt: "2026-03-03T00:00:00Z",
        emittedAt: "2026-03-03T00:00:00Z",
        tags: ["siq", "ops"],
        mentionsInline: [],
        pageLinks: [],
        attributes: { integrity_set: "nostra-core" },
      },
      surfaceJson: {
        payload_type: "a2ui",
        a2ui: {
          tree: {
            widget: "SiqScorecard",
            passing: false,
            score: 50,
            violations: [{ node: "SIQ_GATE_FAIL", error: "Dependency closure missing" }]
          }
        }
      },
      warnings: []
    },
    {
      projection: {
        artifactId: "chart_summary_metrics",
        workspaceId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        blockId: "01ARZ3NDEKTSV4RRFFQ69G5FB1",
        title: "Platform Metrics",
        blockType: "telemetry",
        updatedAt: "2026-03-03T00:00:00Z",
        emittedAt: "2026-03-03T00:00:00Z",
        tags: ["metrics", "system"],
        mentionsInline: [],
        pageLinks: [],
        attributes: { author: "system" },
      },
      surfaceJson: {
        payload_type: "a2ui",
        a2ui: {
          tree: {
            widget: "chart",
            type: "chart",
            chart_data: {
              labels: ["Mon", "Tue", "Wed"],
              datasets: [{ label: "Active Nodes", data: [42, 55, 61], color: "#3b82f6" }],
              chart_type: "bar",
              title: "Active Node Trend"
            }
          }
        }
      },
      warnings: []
    },
  ]
};

function noDeltaResponse() {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-03T00:00:00Z",
    count: 0,
    hasMore: false,
    changed: [],
    deleted: []
  };
}

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => console.log(`BROWSER LOG [${msg.type()}]: ${msg.text()}`));
  page.on("request", (req) => console.log(`[REQ] ${req.method()} ${req.url()}`));
  page.on("response", (res) => console.log(`[RES] ${res.status()} ${res.url()}`));

  // Disable Service Worker and clear IndexedDB to allow Playwright interceptions
  await page.addInitScript(() => {
    // @ts-ignore
    window.navigator.serviceWorker.register = () => new Promise(() => { });
    // @ts-ignore
    window.navigator.serviceWorker.getRegistrations = () => Promise.resolve([]);

    // Clear IndexedDB for a clean state
    const DB_NAME = 'cortex-event-store';
    indexedDB.deleteDatabase(DB_NAME);
  });

  await page.route(/\/api\/cortex\/layout\/spec/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(LAYOUT_SPEC_FIXTURE),
    });
  });

  await page.route(/\/api\/system\/whoami/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(WHOAMI_FIXTURE),
    });
  });

  await page.route(/\/api\/system\/ux\/workbench/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(HEAP_WORKBENCH_FIXTURE),
    });
  });

  await page.route(/\/api\/spaces\/.*\/navigation-plan/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(NAVIGATION_PLAN_FIXTURE),
    });
  });

  await page.route(/\/api\/spaces\/.*\/action-plan/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(ACTION_PLAN_FIXTURE),
    });
  });

  await page.route(/\/api\/cortex\/studio\/heap\/blocks/, async (route) => {
    console.log(`[ROUTE] Intercepting fetch for blocks: ${route.request().url()}`);
    try {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(HEAP_BLOCKS_FIXTURE),
      });
      console.log(`[ROUTE] Fulfilled blocks request with ${HEAP_BLOCKS_FIXTURE.items.length} items`);
    } catch (err) {
      console.error(`[ROUTE] Failed to fulfill blocks request: ${err}`);
      await route.abort();
    }
  });
});

test("heap parity renders structural controls and interactions", async ({ page }) => {
  await page.route(/\/api\/cortex\/studio\/heap\/changed_blocks/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(noDeltaResponse()),
    });
  });
  await page.goto("/explore");

  await expect(page.locator(".shell-layout")).toBeVisible();
  await expect(page.locator(".heap-surface")).toBeVisible();
  await expect(page.locator("button[title='Create New Block']").first()).toBeVisible();
  await expect(page.locator("text=Agent Emit")).toHaveCount(0);

  const firstCard = page.locator(".heap-block-card").filter({ hasText: "Heap Parity Card" }).first();
  await expect(firstCard).toBeVisible();
  await expect(firstCard.locator(".heap-block-card__header")).toBeVisible();
  await expect(firstCard.locator(".heap-payload-label")).toBeVisible();
  await expect(firstCard.locator(".heap-tag-chip").first()).toBeVisible();
  await expect(firstCard.locator(".heap-mention-chip")).toBeVisible();
  await expect(firstCard.locator(".heap-page-link-chip")).toBeVisible();

  const gateSummaryCard = page.locator(".heap-block-card").filter({ hasText: "SIQ Gate Summary" }).first();
  await expect(gateSummaryCard).toContainText("Failing");
  await expect(gateSummaryCard).toContainText("50/100");
  await expect(gateSummaryCard).toContainText("SIQ_GATE_FAIL");

  const chartCard = page.locator(".heap-block-card").filter({ hasText: "Platform Metrics" }).first();
  await expect(chartCard).toBeVisible();
  await expect(chartCard.locator("text=d3 workbench engine")).toBeVisible();
  await expect(chartCard.locator("text=Active Node Trend")).toBeVisible();

  const richTextCard = page.locator(".heap-block-card").filter({ hasText: "Heap Parity Card" }).first();
  await expect(richTextCard).toBeVisible();
  await expect(richTextCard.locator('input[type="checkbox"]')).toHaveCount(2);
  await expect(richTextCard.locator('input[type="checkbox"]').first()).toBeChecked();

  await firstCard.click();
  // Single click now selects the card (adds a ring/border) and shows action bar
  await expect(page.locator(".heap-action-bar")).toBeVisible();
  await expect(page.locator(".heap-action-bar__actions")).toContainText("Refine Selection");
  // Check for selection ring or toast
  await expect(page.locator("text=Block Selected")).toBeVisible();

  // Double click opens the modal
  await firstCard.dblclick();
  await expect(page.locator(".heap-modal-content")).toBeVisible();

  // Verify tabs instead of section labels
  await expect(page.locator("button", { hasText: "preview" })).toBeVisible();
  await expect(page.locator("button", { hasText: "attributes" })).toBeVisible();
  await expect(page.locator("button", { hasText: "relations" })).toBeVisible();
  await expect(page.locator("button", { hasText: "code" })).toBeVisible();
});

test("heap parity delta polling reconciles changed blocks when local flag is enabled", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("cortex.heap.deltaPolling", "1");
    window.localStorage.setItem("cortex.heap.deltaPollingIntervalMs", "500");
  });

  let changedBlocksCalls = 0;
  await page.route(/\/api\/cortex\/studio\/heap\/changed_blocks/, async (route) => {
    changedBlocksCalls += 1;
    if (changedBlocksCalls === 1) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schemaVersion: "1.0.0",
          generatedAt: "2026-03-03T00:00:01Z",
          count: 1,
          hasMore: false,
          changed: [
            {
              projection: {
                artifactId: "artifact-heap-parity-1",
                workspaceId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                blockId: "01ARZ3NDEKTSV4RRFFQ69G5FAW",
                title: "Heap Parity Card (Delta)",
                blockType: "note",
                updatedAt: "2026-03-03T00:00:01Z",
                emittedAt: "2026-03-03T00:00:00Z",
                tags: ["architecture"],
                mentionsInline: ["01ARZ3NDEKTSV4RRFFQ69G5FAX"],
                pageLinks: ["01ARZ3NDEKTSV4RRFFQ69G5FAZ"],
                attributes: { priority: "P0" }
              },
              surfaceJson: {
                payload_type: "rich_text",
                text: "Heap parity body (delta)",
                behaviors: ["pinned"],
                version: "v1.1",
                phase: "Alpha",
                confidence: 90,
                authority_scope: "Local"
              },
              warnings: []
            }
          ],
          deleted: []
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(noDeltaResponse()),
    });
  });

  await page.goto("/explore");
  await expect(page.locator(".heap-block-card").filter({ hasText: "Heap Parity Card" }).first()).toBeVisible();
  await expect(page.locator(".heap-block-card").filter({ hasText: "Heap Parity Card (Delta)" }).first()).toBeVisible({ timeout: 10000 });
  await expect.poll(() => changedBlocksCalls, { timeout: 10000 }).toBeGreaterThan(0);
});

test("heap workbench enrichment features function correctly", async ({ page }) => {
  await page.route(/\/api\/cortex\/studio\/heap\/changed_blocks/, async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(noDeltaResponse()) });
  });
  await page.goto("/explore");

  // 1. Discussions / Comments Verification
  const firstCard = page.locator(".heap-block-card").filter({ hasText: "Heap Parity Card" }).first();
  await firstCard.waitFor({ state: "visible" });
  await page.keyboard.press("Escape"); // Clear any default selection

  const commentBtn = firstCard.locator("button.heap-card-comment-btn");
  await commentBtn.click();
  await page.waitForSelector(".heap-comment-sidebar", { state: "visible", timeout: 10000 });
  // Small wait for sidebar animation and layout adjustment
  await page.waitForTimeout(500);
  const sidebar = page.locator(".heap-comment-sidebar").first();
  await expect(sidebar).toBeVisible();
  await expect(sidebar).toContainText("Discussions");

  const commentInput = sidebar.locator('textarea[placeholder*="Write a comment"]');
  await commentInput.fill("Test comment from Playwright");
  await commentInput.press("Enter");
  await expect(sidebar).toContainText("Test comment from Playwright");
  await expect(sidebar).toContainText("System Intelligence"); // Default author
  
  // Close the sidebar to prevent it from intercepting clicks on the grid
  await sidebar.locator("button").first().click();
  await expect(sidebar).not.toBeVisible();
  await page.waitForTimeout(500);

  // 3. Synthesis Verification
  // Need to select 3 blocks
  const cards = page.locator(".heap-block-card");
  // Click first block normally (no force: true so React event has correct modifiers)
  await cards.nth(0).scrollIntoViewIfNeeded();
  await cards.nth(0).click();
  await page.waitForTimeout(500);

  // Multi-select with Meta key (modifier clicks must NOT use force:true
  // because forced clicks bypass event system and metaKey/ctrlKey are always false)
  await cards.nth(1).scrollIntoViewIfNeeded();
  await cards.nth(1).click({ modifiers: ["Meta"] });
  await page.waitForTimeout(500);

  await cards.nth(2).scrollIntoViewIfNeeded();
  await cards.nth(2).click({ modifiers: ["Meta"] });
  await page.waitForTimeout(500);

  const actionBar = page.locator(".heap-action-bar");
  // Wait longer for action bar to reflect selection
  // Wait for the action bar to reflect the 3rd selection state precisely
  await expect(actionBar).toHaveAttribute("data-selection-count", "3", { timeout: 15000 });

  const synthBtn = actionBar.locator("button", { hasText: "Synthesize" });
  await expect(synthBtn).toBeVisible();

  // Mock the emission call
  await page.route(/\/api\/cortex\/studio\/heap\/emit/, async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ success: true, artifactId: "new-synth-block" }) });
  });

  await synthBtn.click();
  await expect(page.locator("#heap-grid-header")).toContainText(/steward\.synth.*processing/i, { timeout: 10000 });
  await expect(page.locator("#heap-grid-header")).toContainText(/emitted to space/i, { timeout: 15000 });
});
