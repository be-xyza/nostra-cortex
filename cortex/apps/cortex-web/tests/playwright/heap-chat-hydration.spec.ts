import { expect, test } from "@playwright/test";

const THREAD_ID = "thread-acceptance-thread";
const STREAMED_TEXT = "Runtime response from websocket";
const HYDRATED_TEXT = "Hydrated canonical response";

const LAYOUT_SPEC_FIXTURE = {
  layoutId: "shell_layout_v2",
  navigationGraph: {
    entries: [
      { routeId: "/system", label: "System", icon: "SY", category: "Core", requiredRole: "viewer" },
      { routeId: "/explore", label: "Explore", icon: "EX", category: "Core", requiredRole: "viewer" },
      { routeId: "/conversations", label: "Conversations", icon: "CV", category: "Core", requiredRole: "viewer" },
    ],
  },
};

const WHOAMI_FIXTURE = {
  schemaVersion: "1.0.0",
  principal: "local-user",
  effectiveRole: "steward",
  identityVerified: true,
};

const NAVIGATION_PLAN_FIXTURE = {
  schemaVersion: "1.0.0",
  entries: [
    { capabilityId: "cap.heap", routeId: "/explore", label: "Explore", icon: "database", navSlot: "primary" },
    { capabilityId: "cap.conversations", routeId: "/conversations", label: "Conversations", icon: "messages-square", navSlot: "secondary" },
  ],
};

const ACTION_PLAN_FIXTURE = {
  schemaVersion: "1.0.0",
  zones: [
    {
      zone: "heap_selection_bar",
      actions: [
        { id: "mock.synth", capabilityId: "cap.synth", label: "Synthesize", icon: "wand2", kind: "command", action: "synthesize", enabled: true, visible: true },
      ],
    },
  ],
};

const HEAP_WORKBENCH_FIXTURE = {
  type: "surface",
  surfaceId: "surface.heap.chat",
  title: "Heap",
  meta: {},
  components: [
    {
      id: "heap_canvas",
      type: "Container",
      props: {
        widgetType: "HeapCanvas",
      },
      children: [],
    },
  ],
};

const HEAP_BLOCKS_FIXTURE = {
  schemaVersion: "1.0.0",
  generatedAt: "2026-03-28T00:00:00Z",
  count: 2,
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
        mentionsInline: ["chart_summary_metrics"],
        pageLinks: [],
        attributes: { priority: "P0" },
      },
      surfaceJson: {
        payload_type: "rich_text",
        text: "Heap parity body",
      },
      warnings: [],
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
        tags: ["metrics"],
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
              title: "Active Node Trend",
            },
          },
        },
      },
      warnings: [],
    },
  ],
};

function noDeltaResponse() {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-28T00:00:00Z",
    count: 0,
    hasMore: false,
    changed: [],
    deleted: [],
  };
}

function conversationSummary() {
  return {
    generatedAt: "2026-03-28T12:00:03Z",
    count: 1,
    items: [
      {
        threadId: THREAD_ID,
        title: "Summarize the selected block.",
        anchor: {
          kind: "view",
          label: "Explore",
          href: `/explore?thread=${THREAD_ID}`,
          routeId: "/explore",
          viewId: "aggregate:prompt-like",
        },
        messageCount: 2,
        lastMessagePreview: HYDRATED_TEXT,
        createdAt: "2026-03-28T12:00:00Z",
        updatedAt: "2026-03-28T12:00:03Z",
        recentTurns: [
          {
            role: "user",
            text: "Summarize the selected block.",
            timestamp: "2026-03-28T12:00:00Z",
          },
          {
            role: "agent",
            text: HYDRATED_TEXT,
            timestamp: "2026-03-28T12:00:03Z",
          },
        ],
      },
    ],
  };
}

function emptyConversationDetail() {
  return {
    threadId: THREAD_ID,
    title: "Summarize the selected block.",
    anchor: {
      kind: "view",
      label: "Explore",
      href: `/explore?thread=${THREAD_ID}`,
      routeId: "/explore",
      viewId: "aggregate:prompt-like",
    },
    messageCount: 0,
    lastMessagePreview: "",
    createdAt: "2026-03-28T12:00:00Z",
    updatedAt: "2026-03-28T12:00:00Z",
    recentTurns: [],
    messages: [],
  };
}

function hydratedConversationDetail() {
  return {
    threadId: THREAD_ID,
    title: "Summarize the selected block.",
    anchor: {
      kind: "view",
      label: "Explore",
      href: `/explore?thread=${THREAD_ID}`,
      routeId: "/explore",
      viewId: "aggregate:prompt-like",
    },
    messageCount: 2,
    lastMessagePreview: HYDRATED_TEXT,
    createdAt: "2026-03-28T12:00:00Z",
    updatedAt: "2026-03-28T12:00:03Z",
    recentTurns: [
      {
        role: "user",
        text: "Summarize the selected block.",
        timestamp: "2026-03-28T12:00:00Z",
      },
      {
        role: "agent",
        text: HYDRATED_TEXT,
        timestamp: "2026-03-28T12:00:03Z",
      },
    ],
    messages: [
      {
        id: "user-msg-1",
        role: "user",
        text: "Summarize the selected block.",
        timestamp: "2026-03-28T12:00:00Z",
        artifactIds: [],
        content: [{ type: "text", text: "Summarize the selected block." }],
      },
      {
        id: "agent-msg-1",
        role: "agent",
        text: HYDRATED_TEXT,
        timestamp: "2026-03-28T12:00:03Z",
        artifactIds: ["artifact-heap-parity-1"],
        content: [
          { type: "text", text: HYDRATED_TEXT },
          {
            type: "a2ui",
            surfaceId: `chat_context_summary:${THREAD_ID}`,
            title: "Resolved context bundle",
            tree: {
              type: "Container",
              children: {
                explicitList: [
                  {
                    id: "context-heading",
                    componentProperties: {
                      Heading: { text: "Resolved context bundle" },
                    },
                  },
                ],
              },
            },
          },
          {
            type: "pointer",
            href: "/explore?artifact_id=artifact-heap-parity-1",
            label: "Heap Parity Card",
            description: "note · updated 2026-03-03T00:00:00Z",
          },
        ],
        agent: {
          id: "provider",
          label: "Cortex Runtime",
          route: "provider-runtime.responses",
          mode: "runtime",
        },
      },
    ],
  };
}

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    console.log(`[BROWSER ${msg.type()}] ${msg.text()}`);
  });
  page.on("pageerror", (error) => {
    console.log(`[PAGEERROR] ${error.message}`);
  });
  page.on("requestfailed", (request) => {
    console.log(`[REQUEST FAILED] ${request.method()} ${request.url()}`);
  });

  await page.addInitScript((streamedText) => {
    // @ts-ignore
    window.navigator.serviceWorker.register = () => new Promise(() => {});
    // @ts-ignore
    window.navigator.serviceWorker.getRegistrations = () => Promise.resolve([]);
    indexedDB.deleteDatabase("cortex-event-store");

    if (window.crypto && typeof window.crypto.randomUUID === "function") {
      Object.defineProperty(window.crypto, "randomUUID", {
        configurable: true,
        value: () => "acceptance-thread",
      });
    }

    const chatState = { sent: [], urls: [], lastEnvelope: null };
    // @ts-ignore
    window.__chatSocketState = chatState;

    class MockWebSocket {
      constructor(url) {
        this.url = url;
        this.readyState = 1;
        chatState.urls.push(url);
        setTimeout(() => {
          if (this.onopen) this.onopen({ type: "open" });
        }, 0);
      }

      send(data) {
        chatState.sent.push(data);
        chatState.lastEnvelope = JSON.parse(data);
        const textPart = Array.isArray(chatState.lastEnvelope.content)
          ? chatState.lastEnvelope.content.find((part) => part && part.type === "text")
          : null;
        const prompt = textPart && typeof textPart.text === "string" ? textPart.text : "";
        const threadId = chatState.lastEnvelope.threadId || "thread-acceptance-thread";
        const agent = {
          id: "provider",
          label: "Cortex Runtime",
          route: "provider-runtime.responses",
          mode: "runtime",
        };
        const frames = [
          { type: "processing" },
          {
            type: "streaming",
            id: "agent-msg-1",
            delta: streamedText,
            timestamp: "2026-03-28T12:00:01Z",
            agent,
          },
          {
            type: "message",
            id: "agent-msg-1",
            text: `${streamedText}${prompt ? ` for ${prompt}` : ""}`,
            timestamp: "2026-03-28T12:00:02Z",
            agent,
            content: [
              { type: "text", text: `${streamedText}${prompt ? ` for ${prompt}` : ""}` },
              {
                type: "a2ui",
                surfaceId: `chat_context_summary:${threadId}`,
                title: "Resolved context bundle",
                tree: {
                  type: "Container",
                  children: {
                    explicitList: [
                      {
                        id: "context-heading",
                        componentProperties: {
                          Heading: { text: "Resolved context bundle" },
                        },
                      },
                    ],
                  },
                },
              },
              {
                type: "pointer",
                href: "/explore?artifact_id=artifact-heap-parity-1",
                label: "Heap Parity Card",
                description: "note · updated 2026-03-03T00:00:00Z",
              },
            ],
          },
        ];

        frames.forEach((frame, index) => {
          setTimeout(() => {
            if (this.onmessage) {
              this.onmessage({ data: JSON.stringify(frame) });
            }
          }, index * 15);
        });
      }

      close() {
        this.readyState = 3;
        if (this.onclose) this.onclose({ type: "close" });
      }
    }

    Object.defineProperty(MockWebSocket, "OPEN", { value: 1 });
    Object.defineProperty(MockWebSocket, "CLOSED", { value: 3 });
    // @ts-ignore
    window.WebSocket = MockWebSocket;
  }, STREAMED_TEXT);
});

test("heap chat sends canonical context and hydrates after resume", async ({ page }) => {
  let canonicalProjectionReady = false;

  await page.route(/\/api\/cortex\/layout\/spec$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(LAYOUT_SPEC_FIXTURE),
    });
  });
  await page.route(/\/api\/system\/whoami$/, async (route) => {
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
  await page.route(/\/api\/spaces\/.*\/navigation-plan$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(NAVIGATION_PLAN_FIXTURE),
    });
  });
  await page.route(/\/api\/spaces\/.*\/action-plan$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(ACTION_PLAN_FIXTURE),
    });
  });
  await page.route(/\/api\/cortex\/studio\/heap\/blocks/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(HEAP_BLOCKS_FIXTURE),
    });
  });
  await page.route(/\/api\/cortex\/studio\/heap\/changed_blocks/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(noDeltaResponse()),
    });
  });
  await page.route(/\/api\/cortex\/chat\/conversations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(
        canonicalProjectionReady
          ? conversationSummary()
          : { generatedAt: "2026-03-28T12:00:00Z", count: 0, items: [] },
      ),
    });
  });
  await page.route(/\/api\/cortex\/chat\/conversations\/[^/]+$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(
        canonicalProjectionReady ? hydratedConversationDetail() : emptyConversationDetail(),
      ),
    });
  });

  await page.goto("/explore");

  const firstCard = page
    .locator(".heap-block-card")
    .filter({ hasText: "Heap Parity Card" })
    .first();
  await expect(firstCard).toBeVisible({ timeout: 120_000 });
  await firstCard.click();
  const actionBar = page.locator(".heap-action-bar");
  await expect(actionBar).toHaveAttribute("data-selection-count", "1");

  await actionBar.locator("button[title='Chat about selection']").click();
  const chatPanel = page.locator(".chat-panel");
  await expect(chatPanel).toBeVisible();
  await expect(chatPanel).toContainText("1 context");

  const input = chatPanel.locator("textarea");
  await input.fill("Summarize the selected block.");
  await input.press("Enter");

  await expect(chatPanel).toContainText(STREAMED_TEXT, { timeout: 10000 });
  await expect(chatPanel).toContainText("Cortex Runtime");

  const chatState = await page.evaluate(() => {
    // @ts-ignore
    return window.__chatSocketState;
  });
  const outgoing = JSON.parse(chatState.sent[0] as string) as {
    threadId?: string;
    context?: { blockIds?: string[] };
  };
  expect(outgoing.threadId).toBe(THREAD_ID);
  expect(outgoing.context?.blockIds).toEqual(["artifact-heap-parity-1"]);

  const storageAfterSend = await page.evaluate(() => ({
    records: window.localStorage.getItem("cortex.conversations.registry.v1"),
    activeThreadId: window.localStorage.getItem("cortex.conversations.activeThreadId"),
  }));
  expect(storageAfterSend.records).toContain(THREAD_ID);
  expect(storageAfterSend.activeThreadId).toContain(THREAD_ID);

  canonicalProjectionReady = true;

  await page.goto(`/conversations?thread=${THREAD_ID}`);
  await expect(page.locator("h2", { hasText: "Summarize the selected block." })).toBeVisible();
  await expect(page.getByText("Hydrated canonical response").last()).toBeVisible();

  await page.getByRole("button", { name: "Resume in source" }).click();
  await expect(page).toHaveURL(new RegExp(`/explore\\?thread=${THREAD_ID}`));
  await expect(chatPanel).toBeVisible();
  await expect(chatPanel).toContainText(HYDRATED_TEXT);
  await expect(chatPanel).toContainText("Resolved context bundle");
  await expect(chatPanel).toContainText("note · updated 2026-03-03T00:00:00Z");
});
