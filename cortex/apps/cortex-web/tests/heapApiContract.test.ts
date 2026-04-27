import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";
import { SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID } from "../src/store/spaceDesignProfilePreviewContract.ts";

test("getHeapBlocks forwards desktop parity filters and cursor", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-02-24T00:00:00Z",
        count: 0,
        hasMore: false,
        items: []
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getHeapBlocks({
      spaceId: "space-1",
      tag: "tag-1",
      mention: "mention-1",
      pageLink: "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      attribute: "priority:p0",
      blockType: "widget",
      hasFiles: true,
      fromTs: "2026-02-01T00:00:00Z",
      changedSince: "2026-02-01T00:00:00Z",
      toTs: "2026-02-02T00:00:00Z",
      includeDeleted: true,
      limit: 25,
      cursor: "cursor-1"
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.includes("/api/cortex/studio/heap/blocks?"));
  assert.ok(capturedUrl.includes("space_id=space-1"));
  assert.ok(capturedUrl.includes("tag=tag-1"));
  assert.ok(capturedUrl.includes("mention=mention-1"));
  assert.ok(capturedUrl.includes("pageLink=01ARZ3NDEKTSV4RRFFQ69G5FAZ"));
  assert.ok(capturedUrl.includes("attribute=priority%3Ap0"));
  assert.ok(capturedUrl.includes("blockType=widget"));
  assert.ok(capturedUrl.includes("hasFiles=true"));
  assert.ok(capturedUrl.includes("fromTs=2026-02-01T00%3A00%3A00Z"));
  assert.ok(capturedUrl.includes("changedSince=2026-02-01T00%3A00%3A00Z"));
  assert.ok(capturedUrl.includes("toTs=2026-02-02T00%3A00%3A00Z"));
  assert.ok(capturedUrl.includes("includeDeleted=true"));
  assert.ok(capturedUrl.includes("limit=25"));
  assert.ok(capturedUrl.includes("cursor=cursor-1"));
  assert.equal(capturedInit?.method, undefined);
});

test("getSpaceDesignProfilePreview targets metadata-only preview endpoint", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(
      JSON.stringify({
        schema_version: "CortexWebSpaceDesignProfilePreviewFixtureV1",
        snapshot_id: SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID,
        source_mode: "fixture",
        generated_from_profile_ref: "research/120-nostra-design-language/prototypes/space-design/SPACE_DESIGN.space-profile.v1.json",
        runtime_binding: "none",
        runtime_policy: {
          recommendation_only: true,
          applies_tokens_to_cortex_web: false,
          runtime_theme_selection: false,
          requires_verified_projection_for_governance: true,
        },
        prohibited_runtime_claims: ["approved"],
        profiles: [
          {
            profile_id: "space-design-profile:space-research-observatory",
            profile_version: "v0.1.0",
            space_id: "space:research-observatory",
            authority_mode: "recommendation_only",
            review_status: "draft",
            approved_by_count: 0,
            lineage_ref: "research/120-nostra-design-language/prototypes/space-design/SPACE_DESIGN.md",
            surface_scope: ["workbench"],
            a2ui_theme_policy: {
              token_version: "ndl-token-v1",
              safe_mode: true,
              theme_allowlist_id: "ndl-space-profile-draft",
              motion_policy: "system",
              contrast_preference: "system",
            },
            preview_status: "metadata_only",
            preview_note: "Metadata only.",
          },
        ],
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    const response = await workbenchApi.getSpaceDesignProfilePreview();
    assert.equal(response.snapshot_id, SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID);
    assert.equal(response.runtime_binding, "none");
    assert.equal(response.runtime_policy.applies_tokens_to_cortex_web, false);
    assert.equal(response.runtime_policy.runtime_theme_selection, false);
    assert.equal(response.profiles[0]?.preview_status, "metadata_only");
    assert.equal("design_tokens" in response.profiles[0]!, false);
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/system/ux/space-design-profiles"));
  assert.equal(capturedInit?.method, undefined);
});

test("getHeapChangedBlocks uses canonical delta endpoint and forwards query filters", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrl = String(input);
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-03T00:00:00Z",
        count: 0,
        hasMore: false,
        changed: [],
        deleted: []
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getHeapChangedBlocks({
      spaceId: "space-1",
      pageLink: "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      changedSince: "2026-02-01T00:00:00Z",
      includeDeleted: true,
      limit: 15
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.includes("/api/cortex/studio/heap/changed_blocks?"));
  assert.ok(capturedUrl.includes("space_id=space-1"));
  assert.ok(capturedUrl.includes("pageLink=01ARZ3NDEKTSV4RRFFQ69G5FAZ"));
  assert.ok(capturedUrl.includes("changedSince=2026-02-01T00%3A00%3A00Z"));
  assert.ok(capturedUrl.includes("includeDeleted=true"));
  assert.ok(capturedUrl.includes("limit=15"));
});

test("getHeapBlocks preserves projection pageLinks for heap relation rendering", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => {
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-03T00:00:00Z",
        count: 1,
        hasMore: false,
        items: [
          {
            projection: {
              artifactId: "artifact-page-link-1",
              workspaceId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
              title: "Block With Page Link",
              blockType: "note",
              updatedAt: "2026-03-03T00:00:00Z",
              tags: [],
              mentionsInline: [],
              pageLinks: ["01ARZ3NDEKTSV4RRFFQ69G5FAZ"]
            },
            surfaceJson: {
              payload_type: "rich_text",
              text: "hello"
            }
          }
        ]
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const response = await workbenchApi.getHeapBlocks();
    assert.deepEqual(response.items[0]?.projection.pageLinks, ["01ARZ3NDEKTSV4RRFFQ69G5FAZ"]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("heap actions use canonical endpoint paths and operator headers", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-123",
        action: "ok",
        updatedAt: "2026-02-24T00:00:00Z"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.pinHeapBlock("artifact-123");
    await workbenchApi.deleteHeapBlock("artifact-123");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/pin"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/delete"));
  assert.equal(calls[0]?.init?.method, "POST");
  assert.equal(calls[1]?.init?.method, "POST");
  const firstHeaders = calls[0]?.init?.headers as Record<string, string>;
  assert.equal(firstHeaders["x-cortex-role"], "operator");
  assert.equal(firstHeaders["x-cortex-actor"], "cortex-web");
});

test("heap context bundle and history endpoints match canonical desktop paths", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    const url = String(input);
    if (url.includes("/history")) {
      return new Response(
        JSON.stringify({
          artifact_id: "artifact-123",
          versions: []
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    return new Response(
      JSON.stringify({
        context_bundle: {
          blocks: [],
          block_count: 0,
          prepared_at: "2026-02-24T00:00:00Z"
        }
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.createHeapContextBundle(["artifact-123", "artifact-456"]);
    await workbenchApi.getHeapBlockHistory("artifact-123");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/studio/heap/blocks/context"));
  assert.equal(calls[0]?.init?.method, "POST");
  assert.ok(String(calls[0]?.init?.body).includes("artifact-123"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/history"));
});

test("chat conversation api wrappers use canonical conversation projection endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    const url = String(input);
    calls.push({ url, init });
    if (url.endsWith("/api/cortex/chat/conversations")) {
      return new Response(
        JSON.stringify({
          generatedAt: "2026-03-28T00:00:00Z",
          count: 1,
          items: [
            {
              threadId: "thread-1",
              title: "Summarize block set",
              anchor: {
                kind: "view",
                label: "Explore",
                href: "/explore?thread=thread-1",
              },
              messageCount: 2,
              lastMessagePreview: "Hello world",
              createdAt: "2026-03-28T00:00:00Z",
              updatedAt: "2026-03-28T00:00:01Z",
              recentTurns: [
                {
                  role: "user",
                  text: "Summarize this",
                  timestamp: "2026-03-28T00:00:00Z",
                },
              ],
            },
          ],
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      );
    }
    return new Response(
      JSON.stringify({
        threadId: "thread-1",
        title: "Summarize block set",
        anchor: {
          kind: "view",
          label: "Explore",
          href: "/explore?thread=thread-1",
        },
        messageCount: 2,
        lastMessagePreview: "Hello world",
        createdAt: "2026-03-28T00:00:00Z",
        updatedAt: "2026-03-28T00:00:01Z",
        recentTurns: [],
        messages: [
          {
            id: "msg-1",
            role: "agent",
            text: "Hello world",
            timestamp: "2026-03-28T00:00:01Z",
            artifactIds: ["artifact-1"],
            content: [
              { type: "text", text: "Hello world" },
              {
                type: "pointer",
                href: "/explore?artifact_id=artifact-1",
                label: "artifact-1",
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
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    const summary = await workbenchApi.listChatConversations();
    const detail = await workbenchApi.getChatConversation("thread-1");

    assert.equal(summary.count, 1);
    assert.equal(summary.items[0]?.threadId, "thread-1");
    assert.equal(detail.threadId, "thread-1");
    assert.equal(detail.messages[0]?.content[1]?.type, "pointer");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/chat/conversations"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/chat/conversations/thread-1"));
  assert.equal(calls[0]?.init?.method, undefined);
  assert.equal(calls[1]?.init?.method, undefined);
});

test("spatial layout api wrappers use canonical layout persistence endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return new Response(
      JSON.stringify({
        accepted: true,
        layout: {
          schema_version: "1.0.0",
          plane_id: "plane-1",
          view_spec_id: "viewspec-1",
          space_id: "space-1",
          revision: 1,
          layout: {
            shape_positions: {
              "node-1": { x: 120, y: 80 }
            },
            collapsed_groups: {},
            view_state: {
              zoom: 1.1,
              pan_x: 24,
              pan_y: -16
            },
            selected_shape_ids: ["node-1"]
          },
          lineage: {
            updated_by: "operator",
            updated_at: "2026-04-01T00:00:00Z"
          }
        }
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getSpatialPlaneLayout("space-1", "viewspec-1");
    await workbenchApi.saveSpatialPlaneLayout("space-1", "viewspec-1", {
      schema_version: "1.0.0",
      plane_id: "plane-1",
      view_spec_id: "viewspec-1",
      space_id: "space-1",
      revision: 1,
      layout: {
        shape_positions: {
          "node-1": { x: 120, y: 80 }
        },
        collapsed_groups: {},
        view_state: {
          zoom: 1.1,
          pan_x: 24,
          pan_y: -16
        },
        selected_shape_ids: ["node-1"]
      },
      lineage: {
        updated_by: "operator",
        updated_at: "2026-04-01T00:00:00Z"
      }
    } as any);
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/viewspecs/spatial/layouts/space-1/viewspec-1"));
  assert.equal(calls[0]?.init?.method, undefined);
  assert.ok(calls[1]?.url.endsWith("/api/cortex/viewspecs/spatial/layouts/space-1/viewspec-1"));
  assert.equal(calls[1]?.init?.method, "POST");
  assert.ok(String(calls[1]?.init?.body).includes("\"shape_positions\""));
  assert.ok(String(calls[1]?.init?.body).includes("\"view_state\""));
  assert.ok(String(calls[1]?.init?.body).includes("\"selected_shape_ids\""));
});

test("submitA2UIFeedback uses canonical heap feedback endpoint and explicit caller headers", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(JSON.stringify({ accepted: true }), {
      status: 200,
      headers: { "Content-Type": "application/json" }
    });
  }) as typeof fetch;

  try {
    await workbenchApi.submitA2UIFeedback("artifact-123", {
      decision: "approved",
      feedback: "Proceed with bounded live run."
    }, "steward", "systems-steward");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/cortex/studio/heap/blocks/artifact-123/a2ui/feedback"));
  assert.equal(capturedInit?.method, "POST");
  const headers = capturedInit?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "steward");
  assert.equal(headers["x-cortex-actor"], "systems-steward");
  assert.match(String(capturedInit?.body), /bounded live run/i);
});

test("heap export endpoint supports json and markdown payloads", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    const url = String(input);
    if (url.includes("format=json")) {
      return new Response(
        JSON.stringify({ artifact_id: "artifact-123", payload: "ok" }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    return new Response("# Exported Block", {
      status: 200,
      headers: { "Content-Type": "text/markdown; charset=utf-8" }
    });
  }) as typeof fetch;

  try {
    const asJson = await workbenchApi.getHeapBlockExport("artifact-123", "json");
    const asMarkdown = await workbenchApi.getHeapBlockExport("artifact-123", "markdown");
    assert.equal(typeof asJson, "object");
    assert.equal(typeof asMarkdown, "string");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/export?format=json"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/export?format=markdown"));
});

test("space-scoped graph helpers respect the provided active space", async () => {
  const originalFetch = globalThis.fetch;
  const calls: string[] = [];
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    calls.push(String(input));
    return new Response(JSON.stringify({ ok: true }), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  }) as typeof fetch;

  try {
    await workbenchApi.getOverview("space-ops");
    await workbenchApi.getPath("space-ops");
    await workbenchApi.getRuns("space-ops");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(calls[0]?.includes("/api/kg/spaces/space-ops/contribution-graph/overview"));
  assert.ok(calls[1]?.includes("/api/kg/spaces/space-ops/contribution-graph/path-assessment"));
  assert.ok(calls[2]?.includes("/api/kg/spaces/space-ops/contribution-graph/runs?limit=10"));
});

test("heap emit endpoint uses canonical path, operator headers, and schema payload", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-emit-1"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.emitHeapBlock({
      schema_version: "1.0.0",
      mode: "heap",
      space_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      source: {
        agent_id: "cortex-web",
        emitted_at: "2026-03-02T00:00:00Z"
      },
      block: {
        type: "note",
        title: "Emit Contract Test"
      },
      content: {
        payload_type: "rich_text",
        rich_text: {
          plain_text: "hello"
        }
      }
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 1);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/studio/heap/emit"));
  assert.equal(calls[0]?.init?.method, "POST");
  const headers = calls[0]?.init?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "operator");
  assert.equal(headers["x-cortex-actor"], "cortex-web");
  const body = JSON.parse(String(calls[0]?.init?.body)) as Record<string, unknown>;
  assert.equal(body["schema_version"], "1.0.0");
  assert.equal(body["mode"], "heap");
});

test("heap emit accepts task payloads for kickoff blocks", async () => {
  const originalFetch = globalThis.fetch;
  let capturedBody: string | null = null;
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) => {
    capturedBody = String(init?.body ?? "");
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-task-emit-1"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.emitHeapBlock({
      schema_version: "1.0.0",
      mode: "heap",
      space_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      source: {
        agent_id: "cortex-web",
        emitted_at: "2026-03-23T00:00:00Z"
      },
      block: {
        type: "task",
        title: "Initiative 078 Kickoff",
        attributes: {
          initiative_id: "initiative-078-kickoff",
          agent_role: "research-architect"
        }
      },
      content: {
        payload_type: "task",
        task: "# Initiative 078 Kickoff\n- [ ] Read the plan"
      }
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedBody?.includes("\"payload_type\":\"task\""));
  assert.ok(capturedBody?.includes("\"task\":\"# Initiative 078 Kickoff\\n- [ ] Read the plan\""));
});

test("heap emit allows explicit steward identity headers when required by the caller", async () => {
  const originalFetch = globalThis.fetch;
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) => {
    capturedInit = init;
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-emit-2"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.emitHeapBlock(
      {
        schema_version: "1.0.0",
        mode: "heap",
        space_id: "01LIVE123",
        source: {
          agent_id: "systems-steward",
          emitted_at: "2026-03-20T12:00:00Z"
        },
        block: {
          type: "space_promotion_receipt",
          title: "Space created from draft"
        },
        content: {
          payload_type: "rich_text",
          rich_text: {
            plain_text: "receipt"
          }
        }
      },
      "steward",
      "systems-steward",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  const headers = capturedInit?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "steward");
  assert.equal(headers["x-cortex-actor"], "systems-steward");
});

test("heap emit validation rejects malformed payload before network call", async () => {
  const originalFetch = globalThis.fetch;
  let fetchCalled = false;
  globalThis.fetch = (async () => {
    fetchCalled = true;
    return new Response("{}", { status: 200, headers: { "Content-Type": "application/json" } });
  }) as typeof fetch;

  try {
    await assert.rejects(async () => {
      await workbenchApi.emitHeapBlock({
        schema_version: "1.0.0",
        mode: "heap",
        space_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        source: {
          agent_id: "cortex-web",
          emitted_at: "2026-03-02T00:00:00Z"
        },
        block: {
          type: "generated",
          title: "Invalid Generated Payload"
        },
        content: {
          payload_type: "structured_data"
        }
      });
    }, /content\.structured_data/);
    assert.equal(fetchCalled, false);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("heap emit canonicalizes legacy workspace_id payloads to space_id before network call", async () => {
  const originalFetch = globalThis.fetch;
  let capturedBody: Record<string, unknown> | null = null;
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) => {
    capturedBody = JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>;
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-emit-legacy-space"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.emitHeapBlock({
      schema_version: "1.0.0",
      mode: "heap",
      workspace_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      source: {
        agent_id: "cortex-web",
        emitted_at: "2026-03-31T00:00:00Z"
      },
      block: {
        type: "note",
        title: "Legacy scope alias"
      },
      content: {
        payload_type: "rich_text",
        rich_text: {
          plain_text: "compatibility"
        }
      }
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(capturedBody?.["space_id"], "01ARZ3NDEKTSV4RRFFQ69G5FAV");
  assert.equal("workspace_id" in (capturedBody ?? {}), false);
});


test("heap upload helpers use multipart upload and extraction routes", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    const url = String(input);
    if (url.endsWith("/api/cortex/studio/uploads")) {
      return new Response(
        JSON.stringify({
          upload_id: "upload-123",
          resource_ref: "resource://uploads/upload-123",
          hash: "sha256:abc123",
          name: "paper.pdf",
          mime_type: "application/pdf",
          file_size: 42,
          is_uploaded: true,
          thumbnails: [{ type: "preview", size: "small", path: "thumb://1" }],
          extraction_supported: true
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    if (url.endsWith("/extract")) {
      return new Response(
        JSON.stringify({
          job_id: "job-123",
          status: "submitted",
          upload_id: "upload-123",
          requested_parser_profile: "docling"
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    if (url.endsWith("/extractions")) {
      return new Response(
        JSON.stringify({
          generated_at: "2026-03-29T00:00:00Z",
          upload_id: "upload-123",
          items: [
            {
              job_id: "job-123",
              upload_id: "upload-123",
              status: "completed",
              created_at: "2026-03-29T00:00:00Z",
              requested_parser_profile: "docling",
              parser_backend: "docling",
              confidence: 0.91,
              flags: ["ocr"],
              result_ref: "artifact-456",
              summary: "Primary parse",
              page_count: 9,
              block_count: 228,
              last_updated_at: "2026-03-29T00:01:00Z"
            }
          ]
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    if (url.endsWith("/extractions/job-123")) {
      return new Response(
        JSON.stringify({
          job_id: "job-123",
          upload_id: "upload-123",
          status: "completed",
          created_at: "2026-03-29T00:00:00Z",
          requested_parser_profile: "docling",
          parser_backend: "docling",
          confidence: 0.91,
          flags: ["ocr"],
          result_ref: "artifact-456",
          summary: "Primary parse",
          page_count: 9,
          block_count: 228,
          last_updated_at: "2026-03-29T00:01:00Z",
          attempted_backends: ["docling"],
          model_id: "docling:python-api:2.82.0",
          first_page_preview: ["HyEvo", "Self-Evolving", "Hybrid", "Agentic", "Workflows"],
          first_page_block_count: 31
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    return new Response(
      JSON.stringify({
        job_id: "job-123",
        upload_id: "upload-123",
        status: "completed",
        created_at: "2026-03-29T00:00:00Z",
        requested_parser_profile: "docling",
        parser_backend: "docling",
        confidence: 0.91,
        flags: ["ocr"],
        result_ref: "artifact-456",
        page_count: 9,
        block_count: 228
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const uploaded = await workbenchApi.uploadHeapFile({
      file: new File(["hello"], "paper.pdf", { type: "application/pdf" }),
      spaceId: "space-1",
      title: "Paper Upload",
      sourceAgentId: "cortex-web"
    });
    const queued = await workbenchApi.triggerHeapUploadExtraction(uploaded.upload_id);
    const status = await workbenchApi.getHeapUploadExtractionStatus(uploaded.upload_id);
    const runs = await workbenchApi.getHeapUploadExtractionRuns(uploaded.upload_id);
    const detail = await workbenchApi.getHeapUploadExtractionRun(uploaded.upload_id, "job-123");

    assert.equal(uploaded.resource_ref, "resource://uploads/upload-123");
    assert.equal(queued.status, "submitted");
    assert.equal(status.status, "completed");
    assert.equal(status.parser_backend, "docling");
    assert.equal(runs.items.length, 1);
    assert.equal(runs.items[0]?.page_count, 9);
    assert.equal(detail.model_id, "docling:python-api:2.82.0");
    assert.equal(detail.first_page_block_count, 31);
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 5);
  const upload = calls[0];
  assert.ok(upload?.url.endsWith("/api/cortex/studio/uploads"));
  assert.equal(upload?.init?.method, "POST");
  const headers = upload?.init?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "operator");
  assert.equal(headers["x-cortex-actor"], "cortex-web");
  assert.ok(upload?.init?.body instanceof FormData);
  const form = upload?.init?.body as FormData;
  assert.equal(form.get("space_id"), "space-1");
  assert.equal(form.get("title"), "Paper Upload");
  assert.equal(form.get("source_agent_id"), "cortex-web");
  const fileField = form.get("file") as File | FormDataEntryValue | null;
  assert.equal(typeof fileField === "object" && fileField !== null, true);

  const trigger = calls[1];
  assert.ok(trigger?.url.endsWith("/api/cortex/studio/uploads/upload-123/extract"));
  assert.equal(trigger?.init?.method, "POST");
  const statusCall = calls[2];
  assert.ok(statusCall?.url.endsWith("/api/cortex/studio/uploads/upload-123/extraction"));
  const runsCall = calls[3];
  assert.ok(runsCall?.url.endsWith("/api/cortex/studio/uploads/upload-123/extractions"));
  const detailCall = calls[4];
  assert.ok(detailCall?.url.endsWith("/api/cortex/studio/uploads/upload-123/extractions/job-123"));
});

test("heap emit accepts upload-backed pointer payloads", async () => {
  const originalFetch = globalThis.fetch;
  let capturedBody: string | null = null;
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) => {
    capturedBody = String(init?.body ?? "");
    return new Response(JSON.stringify({ accepted: true, artifactId: "artifact-upload-1" }), {
      status: 200,
      headers: { "Content-Type": "application/json" }
    });
  }) as typeof fetch;

  try {
    await workbenchApi.emitHeapBlock({
      schema_version: "1.0.0",
      mode: "heap",
      space_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      source: {
        agent_id: "cortex-web",
        emitted_at: "2026-03-27T00:00:00Z"
      },
      block: {
        type: "upload",
        title: "Paper Upload",
        attributes: {
          file_name: "paper.pdf",
          mime_type: "application/pdf",
          file_size: "42"
        }
      },
      content: {
        payload_type: "pointer",
        pointer: "resource://uploads/upload-123"
      },
      files: [
        {
          hash: "sha256:abc123",
          file_size: 42,
          name: "paper.pdf",
          mime_type: "application/pdf",
          path: "resource://uploads/upload-123",
          is_uploaded: true,
          thumbnails: [{ type: "preview", size: "small", path: "thumb://1" }]
        }
      ]
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedBody?.includes('"payload_type":"pointer"'));
  assert.ok(capturedBody?.includes('"pointer":"resource://uploads/upload-123"'));
  assert.ok(capturedBody?.includes('"type":"upload"'));
  assert.ok(capturedBody?.includes('"files"'));
});

test("steward gate validate and apply use canonical heap endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    const url = String(input);
    if (url.endsWith("/steward-gate/validate")) {
      return new Response(
        JSON.stringify({
          schemaVersion: "1.0.0",
          artifactId: "artifact-123",
          status: "action_required",
          outcome: {
            mode: "warn_or_block",
            shouldBlock: false,
            shouldWarn: true,
            violations: [],
            suggestedEnrichments: [
              {
                enrichmentId: "enrichment_1",
                kind: "pull_request",
                displayLabel: "Convert PR-102",
                matchedText: "PR-102",
                start: 10,
                end: 16,
                metadata: {}
              }
            ]
          },
          stewardGateToken: "sgt1.payload.sig"
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        accepted: true,
        artifactId: "artifact-123",
        enrichmentId: "enrichment_1",
        childArtifactId: "child-1",
        childBlockId: "01ARZ3NDEKTSV4RRFFQ69G5FAY",
        validation: {
          schemaVersion: "1.0.0",
          artifactId: "artifact-123",
          status: "pass",
          outcome: {
            mode: "warn_or_block",
            shouldBlock: false,
            shouldWarn: false,
            violations: [],
            suggestedEnrichments: []
          }
        }
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.validateHeapStewardGate("artifact-123");
    await workbenchApi.applyHeapStewardEnrichment("artifact-123", "enrichment_1");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 2);
  assert.ok(calls[0]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/steward-gate/validate"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/studio/heap/blocks/artifact-123/steward-gate/apply"));
  assert.equal(calls[0]?.init?.method, "POST");
  assert.equal(calls[1]?.init?.method, "POST");
  const headers = calls[0]?.init?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "operator");
  assert.equal(headers["x-cortex-actor"], "cortex-web");
  const body = JSON.parse(String(calls[1]?.init?.body)) as Record<string, unknown>;
  assert.equal(body["enrichmentId"], "enrichment_1");
});

test("publishArtifact forwards steward gate token and governance envelope", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(
      JSON.stringify({
        artifactId: "artifact-123",
        status: "published",
        headRevisionId: "rev-2"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.publishArtifact("artifact-123", {
      notes: "publish",
      stewardGateToken: "sgt1.payload.sig",
      governance: {
        approvedBy: "Systems Steward",
        rationale: "Contract coverage",
        approvedAt: "2026-03-04T00:00:00Z",
        actorId: "cortex-web",
        decisionProof: {
          decisionId: "decision-1",
          signature: "sig",
          signer: "cortex-web"
        }
      }
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/cortex/studio/artifacts/artifact-123/publish"));
  assert.equal(capturedInit?.method, "POST");
  const headers = capturedInit?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "steward");
  assert.equal(headers["x-cortex-actor"], "cortex-web");
  const body = JSON.parse(String(capturedInit?.body)) as Record<string, unknown>;
  assert.equal(body["stewardGateToken"], "sgt1.payload.sig");
});

test("workflow draft api wrappers use canonical workflow endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    const url = String(input);
    calls.push({ url, init });

    if (url.endsWith("/api/cortex/workflow-intents")) {
      return new Response(
        JSON.stringify({
          accepted: true,
          workflowIntent: {
            schemaVersion: "1.0.0",
            workflowIntentId: "workflow_intent_1",
            scope: { spaceId: "space-1" },
            intent: "Draft a workflow proposal for review.",
            motifKind: "parallel_compare",
            constraints: [],
            authorityCeiling: "l2",
            provenance: {
              createdBy: "cortex-web",
              createdAt: "2026-03-23T00:00:00Z",
              sourceMode: "hybrid"
            }
          }
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }

    if (url.endsWith("/api/cortex/workflow-drafts/candidates")) {
      return new Response(
        JSON.stringify({
          schemaVersion: "1.0.0",
          generatedAt: "2026-03-23T00:00:00Z",
          candidateSetId: "workflow_set_1",
          blockedCount: 0,
          candidates: [
            {
              candidateId: "workflow_draft_1",
              workflowDraft: {
                workflowDraftId: "workflow_draft_1",
                scope: { spaceId: "space-1" }
              },
              validation: { valid: true, errors: [], warnings: [] },
              compileResult: { valid: true, warnings: [], digest: "digest-1" },
              generationTrace: {
                strategy: "deterministic_scaffold",
                seedRefs: [],
                policyFlags: {}
              },
              inputHash: "hash-1"
            }
          ]
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }

    if (url.endsWith("/api/cortex/workflow-drafts/candidates/workflow_set_1/stage")) {
      return new Response(
        JSON.stringify({
          accepted: true,
          workflowDraftId: "workflow_draft_1",
          scopeKey: "space:space-1",
          storedAt: "2026-03-23T00:00:00Z"
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }

    if (url.endsWith("/api/cortex/workflow-drafts/workflow_draft_1/propose")) {
      return new Response(
        JSON.stringify({
          accepted: true,
          proposal: {
            proposalId: "workflow_proposal_1",
            workflowDraftId: "workflow_draft_1",
            definitionId: "workflow_def_1",
            scopeKey: "space:space-1",
            proposedBy: "cortex-web",
            rationale: "Route the task into a governed workflow draft.",
            createdAt: "2026-03-23T00:00:00Z",
            status: "staged"
          }
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }

    return new Response(JSON.stringify({ accepted: true }), {
      status: 200,
      headers: { "Content-Type": "application/json" }
    });
  }) as typeof fetch;

  try {
    const intent = await workbenchApi.postWorkflowIntent({
      intent: "Draft a workflow proposal for review.",
      motifKind: "parallel_compare",
      scope: { spaceId: "space-1" },
      authorityCeiling: "l2",
      createdBy: "cortex-web",
      sourceMode: "hybrid"
    });
    const candidateSet = await workbenchApi.postWorkflowCandidates({
      intent: "Draft a workflow proposal for review.",
      motifKind: "parallel_compare",
      scope: intent.workflowIntent.scope,
      generationMode: "motif_hybrid",
      createdBy: "cortex-web",
      sourceMode: "hybrid",
      count: 2
    });
    const staged = await workbenchApi.stageWorkflowCandidate(candidateSet.candidateSetId, {
      candidateId: candidateSet.candidates[0]!.candidateId,
      stagedBy: "cortex-web",
      rationale: "Use the best-scaffolded workflow draft.",
      expectedInputHash: candidateSet.candidates[0]!.inputHash
    });
    const proposal = await workbenchApi.proposeWorkflowDraft(staged.workflowDraftId, {
      proposedBy: "cortex-web",
      rationale: "Route the task into a governed workflow draft."
    });

    assert.equal(intent.accepted, true);
    assert.equal(candidateSet.candidateSetId, "workflow_set_1");
    assert.equal(staged.workflowDraftId, "workflow_draft_1");
    assert.equal(proposal.proposal.proposalId, "workflow_proposal_1");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(calls[0]?.url.endsWith("/api/cortex/workflow-intents"));
  assert.ok(calls[1]?.url.endsWith("/api/cortex/workflow-drafts/candidates"));
  assert.ok(calls[2]?.url.endsWith("/api/cortex/workflow-drafts/candidates/workflow_set_1/stage"));
  assert.ok(calls[3]?.url.endsWith("/api/cortex/workflow-drafts/workflow_draft_1/propose"));
  assert.equal(calls[0]?.init?.method, "POST");
  assert.equal(calls[1]?.init?.method, "POST");
  assert.equal(calls[2]?.init?.method, "POST");
  assert.equal(calls[3]?.init?.method, "POST");
});
