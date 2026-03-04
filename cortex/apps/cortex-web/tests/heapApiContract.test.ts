import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";

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
  assert.ok(capturedUrl.includes("spaceId=space-1"));
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
  assert.ok(capturedUrl.includes("spaceId=space-1"));
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
      workspace_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
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
        workspace_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
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
