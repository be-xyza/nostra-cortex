import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";
import {
  getSpaceDisplayName,
  mapSpaceRegistryRecordToSpace,
} from "../src/store/spacesRegistry.ts";

test("getSpaces targets the canonical registry endpoint", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrl = String(input);
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-19T00:00:00Z",
        count: 0,
        items: [],
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getSpaces();
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/spaces"));
});

test("getSpaceReadiness targets the canonical readiness endpoint", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrl = String(input);
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-19T00:00:00Z",
        spaceId: "01KM4C04QY37V9RV9H2HH9J1NM",
        sourceMode: "registered",
        readinessSummary: "pass",
        readiness: {
          registry: "pass",
          navigationPlan: "pass",
          agentRuns: "pass",
          contributionGraphArtifact: "fail",
          contributionGraphRuns: "pass",
          capabilityGraph: "pass",
          summary: "pass",
        },
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getSpaceReadiness("01KM4C04QY37V9RV9H2HH9J1NM");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/spaces/01KM4C04QY37V9RV9H2HH9J1NM/readiness"));
});

test("createSpace uses the canonical create endpoint and steward identity headers", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(
      JSON.stringify({
        space_id: "alpha-space",
        status: "created",
        message: "created",
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    await workbenchApi.createSpace(
      {
        space_id: "alpha-space",
        creation_mode: "blank",
        owner: "systems-steward",
        governance_scope: "private",
      },
      "steward",
      "systems-steward",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/spaces/create"));
  assert.equal(capturedInit?.method, "POST");
  const headers = capturedInit?.headers as Record<string, string>;
  assert.equal(headers["x-cortex-role"], "steward");
  assert.equal(headers["x-cortex-actor"], "systems-steward");
  assert.match(String(capturedInit?.body ?? ""), /alpha-space/);
  assert.match(String(capturedInit?.body ?? ""), /private/);
});

test("space registry mapping produces deterministic live-space labels and actions", () => {
  const record = {
    spaceId: "01KM4C04QY37V9RV9H2HH9J1NM",
    creationMode: "blank",
    referenceUri: null,
    templateId: null,
    capabilityGraphUri: null,
    capabilityGraphVersion: null,
    capabilityGraphHash: null,
    status: "active",
    createdAt: "1742430000",
    owner: "systems-steward",
    members: ["systems-steward", "agent:cortex-worker-01"] as string[],
    archetype: "Research",
    draftId: "draft-space-12",
    draftSourceMode: "template",
    lineageNote: "Started from the research starter.",
    governanceScope: "private",
    visibilityState: "members_only",
    sourceMode: "observed",
    readinessSummary: "in_progress",
    readiness: {
      registry: "in_progress",
      navigationPlan: "pass",
      agentRuns: "pass",
      contributionGraphArtifact: "fail",
      contributionGraphRuns: "pass",
      capabilityGraph: "pass",
      summary: "in_progress",
    },
  } as const;

  assert.equal(getSpaceDisplayName(record), "Research · 01KM4C04");

  const mapped = mapSpaceRegistryRecordToSpace(record);
  assert.equal(mapped.id, "01KM4C04QY37V9RV9H2HH9J1NM");
  assert.equal(mapped.type, "system");
  assert.equal(mapped.sourceMode, "observed");
  assert.equal(mapped.readinessSummary, "in_progress");
  assert.equal(mapped.stats?.memberCount, 2);
  assert.deepEqual(mapped.config?.actions, ["details", "copy_id", "explore"]);
  assert.equal(mapped.metadata?.lineage?.draftId, "draft-space-12");
  assert.equal(mapped.metadata?.lineage?.sourceMode, "template");
  assert.equal(mapped.metadata?.governance?.scope, "private");
  assert.equal(mapped.metadata?.governance?.visibilityState, "members_only");
});
