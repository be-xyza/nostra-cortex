import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";
import {
  buildContributionFocusGraphModel,
  normalizeContributionSelection,
  pipelineModeRequiresApproval,
} from "../src/components/contributions/contributionsRouteState.ts";

test("normalizeContributionSelection prioritizes explicit run and contribution query state", () => {
  const fromNodeRun = normalizeContributionSelection(
    new URLSearchParams("node_id=agent_run:run-123")
  );
  assert.deepEqual(fromNodeRun, {
    focusKind: "agent_run",
    selectedContributionId: null,
    selectedAgentRunId: "run-123",
    selectedGraphRunId: null,
  });

  const fromNodeContribution = normalizeContributionSelection(
    new URLSearchParams("node_id=contribution:proposal-alpha")
  );
  assert.deepEqual(fromNodeContribution, {
    focusKind: "contribution",
    selectedContributionId: "proposal-alpha",
    selectedAgentRunId: null,
    selectedGraphRunId: null,
  });

  const fromAliases = normalizeContributionSelection(
    new URLSearchParams("run_id=run-456&contribution_id=proposal-beta")
  );
  assert.deepEqual(fromAliases, {
    focusKind: "graph_run",
    selectedContributionId: "proposal-beta",
    selectedAgentRunId: null,
    selectedGraphRunId: "run-456",
  });

  const fromMixedAliases = normalizeContributionSelection(
    new URLSearchParams("node_id=agent_run:run-777&run_id=graph-run-2&contribution_id=proposal-gamma")
  );
  assert.deepEqual(fromMixedAliases, {
    focusKind: "agent_run",
    selectedContributionId: "proposal-gamma",
    selectedAgentRunId: "run-777",
    selectedGraphRunId: "graph-run-2",
  });
});

test("pipelineModeRequiresApproval only gates steward-mutating pipeline modes", () => {
  assert.equal(pipelineModeRequiresApproval("validate"), false);
  assert.equal(pipelineModeRequiresApproval("path"), false);
  assert.equal(pipelineModeRequiresApproval("diff"), false);
  assert.equal(pipelineModeRequiresApproval("doctor"), true);
  assert.equal(pipelineModeRequiresApproval("simulate"), true);
  assert.equal(pipelineModeRequiresApproval("publish"), true);
  assert.equal(pipelineModeRequiresApproval("full"), true);
});

test("buildContributionFocusGraphModel produces a centered, relation-aware focus map", () => {
  const graph = buildContributionFocusGraphModel("proposal-alpha", {
    contributionId: "proposal-alpha",
    dependsOn: ["spec-alpha"],
    dependedBy: ["plan-alpha"],
    invalidates: ["draft-beta"],
    invalidatedBy: [],
    supersedes: [],
    supersededBy: [],
    references: ["initiative-132"],
    referencedBy: ["brief-gamma"],
  });

  assert.deepEqual(graph.nodes.map((node) => node.id), [
    "proposal-alpha",
    "spec-alpha",
    "plan-alpha",
    "draft-beta",
    "initiative-132",
    "brief-gamma",
  ]);
  assert.deepEqual(
    graph.edges.map((edge) => [edge.source, edge.target, edge.relationship]),
    [
      ["proposal-alpha", "spec-alpha", "dependsOn"],
      ["plan-alpha", "proposal-alpha", "dependedBy"],
      ["proposal-alpha", "draft-beta", "invalidates"],
      ["proposal-alpha", "initiative-132", "references"],
      ["brief-gamma", "proposal-alpha", "referencedBy"],
    ]
  );
  assert.deepEqual(
    graph.groups.map((group) => [group.key, group.items.length]),
    [
      ["dependsOn", 1],
      ["dependedBy", 1],
      ["invalidates", 1],
      ["references", 1],
      ["referencedBy", 1],
    ]
  );
});

test("contributions cockpit API methods hit canonical endpoints", async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return new Response(JSON.stringify({ ok: true }), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  }) as typeof fetch;

  try {
    await workbenchApi.getSystemAgentRuns("space-alpha");
    await workbenchApi.getSystemAgentRun("space-alpha", "run-123");
    await workbenchApi.getContributionBlastRadius("proposal-alpha", "space-alpha");
    await workbenchApi.exportStewardPacket(
      {
        goal: "stable-cortex-domain",
        fromVersion: "v0.1.0",
        toVersion: "v0.2.0",
        approval: {
          approvedBy: "steward:operator",
          rationale: "Ready for steward export",
          approvedAt: "2026-03-20T00:00:00Z",
          decisionRef: "DEC-123",
        },
      },
      "steward",
      "steward:operator",
      "space-alpha"
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(
    calls.map((entry) => ({
      url: entry.url,
      method: entry.init?.method ?? "GET",
      actor: entry.init?.headers instanceof Headers
        ? entry.init.headers.get("x-cortex-actor")
        : (entry.init?.headers as Record<string, string> | undefined)?.["x-cortex-actor"],
      role: entry.init?.headers instanceof Headers
        ? entry.init.headers.get("x-cortex-role")
        : (entry.init?.headers as Record<string, string> | undefined)?.["x-cortex-role"],
    })),
    [
      {
        url: "/api/system/agents/runs?space_id=space-alpha",
        method: "GET",
        actor: undefined,
        role: undefined,
      },
      {
        url: "/api/system/agents/runs/space-alpha/run-123",
        method: "GET",
        actor: undefined,
        role: undefined,
      },
      {
        url: "/api/kg/spaces/space-alpha/contribution-graph/blast-radius?contributionId=proposal-alpha",
        method: "GET",
        actor: undefined,
        role: undefined,
      },
      {
        url: "/api/kg/spaces/space-alpha/contribution-graph/steward-packet/export",
        method: "POST",
        actor: "steward:operator",
        role: "steward",
      },
    ]
  );
});
