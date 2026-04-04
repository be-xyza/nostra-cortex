import assert from "node:assert/strict";
import test from "node:test";

import {
  buildAgentExecutionRecentWorkItem,
  buildProposalReviewRecentWorkItem,
  buildPromotionReceiptRecentWorkItem,
  buildSpaceDetailModel,
} from "../src/components/spaces/spaceDetailModel.ts";
import type { HeapBlockListItem } from "../src/contracts.ts";
import type { Space } from "../src/store/spacesRegistry.ts";

test("buildSpaceDetailModel favors plain language and real member data", () => {
  const model = buildSpaceDetailModel({
    id: "01KM4C04QY37V9RV9H2HH9J1NM",
    name: "Research · 01KM4C04",
    type: "system",
    description: "Research Space",
    owner: "systems-steward",
    status: "active",
    createdAt: "2026-03-20T00:58:40.728964+00:00",
    members: ["systems-steward", "agent:eudaemon-alpha-01"],
    archetype: "Research",
  } satisfies Space);

  assert.equal(model.statusLabel, "Active");
  assert.match(model.statusMessage, /ready to open/i);
  assert.equal(model.aboutRows[0]?.label, "Purpose");
  assert.equal(model.aboutRows[1]?.label, "Owner");
  assert.equal(model.aboutRows[2]?.label, "Created");
  assert.equal(model.aboutRows[3]?.label, "Access");
  assert.equal(model.people[0]?.name, "Systems Steward");
  assert.equal(model.people[0]?.roleLabel, "Owner");
  assert.equal(model.people[1]?.name, "Eudaemon Alpha 01");
  assert.equal(model.people[1]?.roleLabel, "Agent");
  assert.equal(model.recentWork[0]?.label, "Connected agent");
});

test("buildSpaceDetailModel keeps the owner visible even when registry members only list agents", () => {
  const model = buildSpaceDetailModel({
    id: "01KM4C04QY37V9RV9H2HH9J1NM",
    name: "Research · 01KM4C04",
    type: "system",
    description: "Research Space",
    owner: "systems-steward",
    status: "active",
    createdAt: "2026-03-20T00:58:40.728964+00:00",
    members: ["agent:cortex-worker-01"],
    archetype: "Research",
  } satisfies Space);

  assert.equal(model.people[0]?.name, "Systems Steward");
  assert.equal(model.people[0]?.roleLabel, "Owner");
  assert.equal(model.people[1]?.name, "Cortex Worker 01");
  assert.equal(model.aboutRows[3]?.value, "2 people or agents can work here");
});

test("buildPromotionReceiptRecentWorkItem turns a promotion receipt into plain recent work text", () => {
  const item = buildPromotionReceiptRecentWorkItem({
    projection: {
      artifactId: "artifact-123",
      title: "Space created from draft",
      blockType: "space_promotion_receipt",
      updatedAt: "2026-03-20T12:00:00Z",
      emittedAt: "2026-03-20T12:00:00Z",
      tags: [],
      mentionsInline: [],
      attributes: {
        source_mode: "template",
        created_space_id: "01KM4C04QY37V9RV9H2HH9J1NM",
      },
    },
    surfaceJson: {
      payload_type: "rich_text",
      rich_text: {
        plain_text: "Space created from draft",
      },
    },
  } satisfies HeapBlockListItem);

  assert.deepEqual(item, {
    label: "Created from draft",
    value: "This space was created from a saved draft on March 20, 2026.",
  });
});

test("buildAgentExecutionRecentWorkItem turns the latest agent execution into a plain language update", () => {
  const item = buildAgentExecutionRecentWorkItem({
    projection: {
      artifactId: "artifact-exec-1",
      title: "agent_execution_record block",
      blockType: "agent_execution_record",
      updatedAt: "2026-03-20T13:00:00Z",
      emittedAt: "2026-03-20T13:00:00Z",
      tags: [],
      mentionsInline: [],
    },
    surfaceJson: {
      agent_id: "agent:cortex-worker-01",
      benchmark: {
        pass_rate: 0.12,
        latency_ms: 4725,
        total_tokens: 2140,
        assertions_passed: 3,
        assertions_total: 18,
      },
    },
  } satisfies HeapBlockListItem);

  assert.deepEqual(item, {
    label: "Latest Eudaemon update",
    value:
      "Eudaemon last reviewed this space on March 20, 2026 and flagged that it needs attention. 12% pass rate • 3 of 18 assertions passed • 4725ms latency • 2140 tokens.",
  });
});

test("buildProposalReviewRecentWorkItem only shows a simple review cue for real proposals", () => {
  const item = buildProposalReviewRecentWorkItem({
    projection: {
      artifactId: "artifact-proposal-1",
      title: "proposal block",
      blockType: "proposal",
      updatedAt: "2026-03-20T14:00:00Z",
      emittedAt: "2026-03-20T14:00:00Z",
      tags: [],
      mentionsInline: [],
    },
    surfaceJson: {
      rationale: "Recommend adding a governed source manifest.",
    },
  } satisfies HeapBlockListItem);

  assert.deepEqual(item, {
    label: "Needs review",
    value: "A new recommendation is ready for review in this space.",
    href: "/explore?artifact_id=artifact-proposal-1",
    actionLabel: "Open",
  });
});

test("buildSpaceDetailModel includes persisted draft lineage in plain language", () => {
  const model = buildSpaceDetailModel({
    id: "01KM4C04QY37V9RV9H2HH9J1NM",
    name: "Research · 01KM4C04",
    type: "system",
    description: "Research Space",
    owner: "systems-steward",
    status: "active",
    createdAt: "2026-03-20T00:58:40.728964+00:00",
    members: ["systems-steward", "agent:eudaemon-alpha-01"],
    archetype: "Research",
    metadata: {
      lineage: {
        draftId: "draft-space-12",
        sourceMode: "template",
        note: "Started from the research starter.",
      },
    },
  } satisfies Space);

  assert.equal(model.aboutRows[4]?.label, "Started from");
  assert.equal(model.aboutRows[4]?.value, "Started from the research starter.");
});

test("buildSpaceDetailModel shows plain-language access for governed public spaces", () => {
  const model = buildSpaceDetailModel({
    id: "space-public",
    name: "Research · SPACEPUB",
    type: "user",
    description: "A space for shared research.",
    owner: "systems-steward",
    status: "active",
    createdAt: "2026-03-20T00:58:40.728964+00:00",
    members: ["systems-steward", "agent:eudaemon-alpha-01"],
    archetype: "Research",
    metadata: {
      governance: {
        scope: "public",
        visibilityState: "discoverable",
      },
    },
  } satisfies Space);

  assert.equal(model.aboutRows[3]?.label, "Access");
  assert.equal(model.aboutRows[3]?.value, "Public space with broader shared access");
});
