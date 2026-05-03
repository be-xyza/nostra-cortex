import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapAggregationGroups,
  buildHeapDerivedViews,
  collectHeapAggregationArtifactIds,
} from "../src/components/heap/heapAggregation.ts";
import { buildHeapViewContext } from "../src/components/heap/heapViewRegistry.ts";

test("buildHeapAggregationGroups batches prompt-like blocks into a readable summary", () => {
  const groups = buildHeapAggregationGroups([
    {
      projection: {
        artifactId: "prompt-1",
        title: "Prompt A",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:05:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "agent_solicitation",
          role: "research-architect",
          authority_scope: "L1",
          required_capabilities: ["graph-validation"],
          budget: { currency: "USD", max: 20 },
          message: "Review the lineage before approving.",
        },
      },
    },
    {
      projection: {
        artifactId: "feedback-1",
        title: "Feedback A",
        blockType: "steward_feedback",
        updatedAt: "2026-03-20T01:06:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "steward_feedback",
          artifact_id: "feedback-1",
          parent_artifact_id: "prompt-1",
          decision: "approved",
          feedback: "Proceed.",
          submitted_by: "operator",
          submitted_at: "2026-03-20T01:06:00Z",
        },
      },
    },
  ]);

  assert.deepEqual(groups.map((group) => group.groupId), [
    "prompt-like",
    "steward-feedback",
  ]);
  assert.equal(groups[0]?.count, 1);
  assert.equal(groups[0]?.items[0]?.source.projection.artifactId, "prompt-1");
  assert.equal(groups[0]?.label, "Prompts");
  assert.equal(groups[0]?.columns[0]?.label, "Role");
  assert.equal(groups[0]?.items[0]?.fields.role, "research-architect");
  assert.equal(groups[1]?.count, 1);
  assert.equal(groups[1]?.items[0]?.source.projection.artifactId, "feedback-1");
  assert.equal(groups[1]?.items[0]?.fields.decision, "Approved");
});

test("group summaries strip markdown code blocks and fall back to readable text", () => {
  const groups = buildHeapAggregationGroups([
    {
      projection: {
        artifactId: "prompt-2",
        title: "Prompt B",
        blockType: "prompt",
        updatedAt: "2026-03-20T01:08:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "prompt",
          plain_text: "Review this patch.\n```ts\nconst broken = true;\n```",
          role: "operator",
        },
      },
    },
  ]);

  assert.equal(groups[0]?.items[0]?.summary, "Review this patch.");
});

test("prompt-like blocks without a solicitation payload do not claim a requested role", () => {
  const groups = buildHeapAggregationGroups([
    {
      projection: {
        artifactId: "prompt-3",
        title: "Prompt C",
        blockType: "prompt",
        updatedAt: "2026-03-20T01:09:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "prompt",
          summary: "Draft a concise response.",
        },
      },
    },
  ]);

  assert.equal(groups[0]?.items[0]?.fields.role, "unspecified");
  assert.equal(groups[0]?.items[0]?.badge, "prompt");
});

test("derived heap views include board and grouped projections while grouped ids remain suppressible", () => {
  const blocks = [
    {
      projection: {
        artifactId: "prompt-1",
        title: "Prompt A",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:05:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "agent_solicitation",
          role: "research-architect",
          authority_scope: "L1",
          required_capabilities: ["graph-validation"],
          budget: { currency: "USD", max: 20 },
          message: "Review the lineage before approving.",
        },
      },
    },
    {
      projection: {
        artifactId: "note-1",
        title: "Plain Note",
        blockType: "note",
        updatedAt: "2026-03-20T01:07:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "rich_text",
        text: "Ungrouped note",
      },
    },
  ];

  const groups = buildHeapAggregationGroups(blocks);
  const derivedViews = buildHeapDerivedViews(blocks, groups);
  const groupedIds = collectHeapAggregationArtifactIds(groups);

  assert.deepEqual(derivedViews.map((view) => view.id), ["board", "all-blocks", "aggregate:prompt-like"]);
  assert.equal(derivedViews[0]?.label, "Relevant updates");
  assert.equal(derivedViews[1]?.label, "All Blocks");
  assert.equal(groupedIds.has("prompt-1"), true);
  assert.equal(groupedIds.has("note-1"), false);
});

test("system telemetry records are grouped into contributor-friendly digest views", () => {
  const blocks = [
    {
      projection: {
        artifactId: "usage-1",
        title: "usage_report block",
        blockType: "usage_report",
        updatedAt: "2026-03-22T07:14:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: { payload_type: "structured_data", text: "usage report block" },
    },
    {
      projection: {
        artifactId: "agent-1",
        title: "agent_execution_record block",
        blockType: "agent_execution_record",
        updatedAt: "2026-03-22T07:13:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: { payload_type: "structured_data", text: "agent execution record block" },
    },
    {
      projection: {
        artifactId: "proposal-1",
        title: "self_optimization_proposal block",
        blockType: "self_optimization_proposal",
        updatedAt: "2026-03-22T07:12:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: { payload_type: "structured_data", text: "self optimization proposal block" },
    },
  ];

  const groups = buildHeapAggregationGroups(blocks);
  const groupedIds = collectHeapAggregationArtifactIds(groups);

  assert.deepEqual(groups.map((group) => group.groupId), [
    "usage-report",
    "agent-work",
    "suggested-improvements",
  ]);
  assert.equal(groups[0]?.label, "Recent activity summaries");
  assert.equal(groups[0]?.items[0]?.title, "Activity summary");
  assert.equal(groups[1]?.items[0]?.title, "Agent work update");
  assert.equal(groups[2]?.items[0]?.title, "Suggested improvement");
  assert.equal(groupedIds.has("usage-1"), true);
  assert.equal(groupedIds.has("agent-1"), true);
  assert.equal(groupedIds.has("proposal-1"), true);
});

test("heap view context carries context-aware signals for derived views", () => {
  const blocks = [
    {
      projection: {
        artifactId: "prompt-1",
        title: "Prompt A",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:05:00Z",
        tags: [],
        mentionsInline: [],
      },
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          type: "agent_solicitation",
          role: "research-architect",
          authority_scope: "L1",
          required_capabilities: ["graph-validation"],
          budget: { currency: "USD", max: 20 },
          message: "Review the lineage before approving.",
        },
      },
    },
  ];

  const groups = buildHeapAggregationGroups(blocks);
  const derivedViews = buildHeapDerivedViews(blocks, groups);
  const activeView = derivedViews.find((view) => view.id === "aggregate:prompt-like") ?? null;
  const context = buildHeapViewContext(activeView, groups);

  assert.equal(context?.viewLabel, "Prompts");
  assert.equal(context?.signals[0]?.label, "Newest request");
  assert.match(context?.signals[0]?.prompt ?? "", /request shape/i);
});
