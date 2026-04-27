import assert from "node:assert/strict";
import test from "node:test";

import { openGatewayApiArtifact, workbenchApi } from "../src/api.ts";

test("workflow api methods hit canonical gateway endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return new Response(
      JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getWorkflowDraftProposalReplay("proposal-1");
    await workbenchApi.getWorkflowDraftProposalDigest("proposal-1");
    await workbenchApi.getWorkflowDefinition("definition-1");
    await workbenchApi.getWorkflowDefinitionProjection("definition-1", "flow_graph_v1");
    await workbenchApi.getWorkflowActiveDefinition("scope-alpha");
    await workbenchApi.getWorkflowInstance("instance-1");
    await workbenchApi.getWorkflowInstanceTrace("instance-1");
    await workbenchApi.getWorkflowInstanceCheckpoints("instance-1");
    await workbenchApi.getWorkflowInstanceOutcome("instance-1");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(
    calls.map((entry) => entry.url),
    [
      "/api/cortex/workflow-drafts/proposals/proposal-1/replay",
      "/api/cortex/workflow-drafts/proposals/proposal-1/digest",
      "/api/cortex/workflow-definitions/definition-1",
      "/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1",
      "/api/cortex/workflow-definitions/active/scope-alpha",
      "/api/cortex/workflow-instances/instance-1",
      "/api/cortex/workflow-instances/instance-1/trace",
      "/api/cortex/workflow-instances/instance-1/checkpoints",
      "/api/cortex/workflow-instances/instance-1/outcome",
    ]
  );
});

test("workflow definition fetch preserves typed definition payload", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () =>
    new Response(
      JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
        definition: {
          schemaVersion: "1.0.0",
          definitionId: "definition-1",
          scope: { spaceId: "space-alpha" },
          intent: "Typed definition payload",
          motifKind: "sequential",
          constraints: [],
          graph: { nodes: [], edges: [] },
          contextContract: { allowedSections: ["inputs"] },
          confidence: { score: 0.9, rationale: "fixture" },
          lineage: { mergeRefs: [] },
          policy: {
            recommendationOnly: false,
            requireReview: true,
            allowShadowExecution: false,
          },
          provenance: {
            createdBy: "tester",
            createdAt: "2026-03-11T00:00:00Z",
            sourceMode: "fixture",
          },
          digest: "sha256:typed-definition",
        },
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    )) as typeof fetch;

  try {
    const response = await workbenchApi.getWorkflowDefinition("definition-1");
    assert.equal(response.definition.motifKind, "sequential");
    assert.equal(response.definition.digest, "sha256:typed-definition");
    assert.equal(response.definition.scope.spaceId, "space-alpha");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("openGatewayApiArtifact resolves inline workflow artifacts through typed fetches", async () => {
  const originalFetch = globalThis.fetch;
  const calls: string[] = [];
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    calls.push(String(input));
    return new Response(
      JSON.stringify({
        schema_version: "1.0.0",
        generated_at: "2026-03-11T00:00:00Z",
        projection_kind: "flow_graph_v1",
        projection: { nodes: [] },
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const artifact = await openGatewayApiArtifact(
      "/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1",
      "inline"
    );
    assert.equal((artifact as { projection_kind?: string }).projection_kind, "flow_graph_v1");
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(calls, [
    "/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1",
  ]);
});
