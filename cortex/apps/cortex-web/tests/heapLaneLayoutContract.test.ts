import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapLanes,
  resolveHeapLaneCount,
} from "../src/components/heap/heapLaneLayout.ts";
import {
  DEFAULT_EXPLORE_LIST_POLICY,
  DENSITY_EXPLORE_LIST_POLICY,
  LINEAGE_EXPLORE_LIST_POLICY,
  STORY_EXPLORE_LIST_POLICY,
  resolveExploreSurfacePolicy,
} from "../src/components/heap/exploreSurfacePolicy.ts";

type BlockFixture = Parameters<typeof buildHeapLanes>[0][number];

function makeBlock(
  artifactId: string,
  blockType: string,
  title: string,
  text = ""
): BlockFixture {
  return {
    projection: {
      artifactId,
      workspaceId: "space-1",
      title,
      blockType,
      updatedAt: "2026-03-20T00:00:00Z",
      tags: [],
      mentionsInline: [],
      pageLinks: [],
      attributes: {},
    },
    surfaceJson: {
      payload_type: blockType,
      text,
    },
    warnings: [],
  } as BlockFixture;
}

test("resolveHeapLaneCount matches Explore responsive breakpoints", () => {
  assert.equal(resolveHeapLaneCount(320, DEFAULT_EXPLORE_LIST_POLICY), 1);
  assert.equal(resolveHeapLaneCount(767, DEFAULT_EXPLORE_LIST_POLICY), 1);
  assert.equal(resolveHeapLaneCount(768, DEFAULT_EXPLORE_LIST_POLICY), 2);
  assert.equal(resolveHeapLaneCount(1279, DEFAULT_EXPLORE_LIST_POLICY), 2);
  assert.equal(resolveHeapLaneCount(1280, DEFAULT_EXPLORE_LIST_POLICY), 3);
  assert.equal(resolveHeapLaneCount(1599, DEFAULT_EXPLORE_LIST_POLICY), 3);
  assert.equal(resolveHeapLaneCount(1600, DEFAULT_EXPLORE_LIST_POLICY), 4);
});

test("resolveExploreSurfacePolicy exposes the default reusable Explore list policy", () => {
  const policy = resolveExploreSurfacePolicy({
    surfaceId: "explore.list",
  });

  assert.equal(policy.policyId, "explore.list.default.v1");
  assert.equal(policy.projectionMode, "list");
  assert.equal(policy.projectionIntent, "overview");
  assert.equal(policy.laneBreakpoints[0]?.laneCount, 4);
  assert.equal(policy.laneWeightByBlockType.chart, 8);
});

test("resolveExploreSurfacePolicy maps intro-style archetypes to the story policy", () => {
  const policy = resolveExploreSurfacePolicy({
    surfaceId: "explore.list",
    spaceArchetype: "Intro",
  });

  assert.equal(policy.policyId, STORY_EXPLORE_LIST_POLICY.policyId);
  assert.equal(policy.projectionMode, "list");
  assert.equal(policy.projectionIntent, "story");
});

test("resolveExploreSurfacePolicy maps research-style archetypes to the density policy", () => {
  const policy = resolveExploreSurfacePolicy({
    surfaceId: "explore.list",
    spaceArchetype: "research",
  });

  assert.equal(policy.policyId, DENSITY_EXPLORE_LIST_POLICY.policyId);
  assert.equal(policy.projectionMode, "list");
  assert.equal(policy.projectionIntent, "density");
});

test("resolveExploreSurfacePolicy maps governance-style archetypes to the lineage policy", () => {
  const policy = resolveExploreSurfacePolicy({
    surfaceId: "explore.list",
    spaceArchetype: "Governance",
  });

  assert.equal(policy.policyId, LINEAGE_EXPLORE_LIST_POLICY.policyId);
  assert.equal(policy.projectionMode, "list");
  assert.equal(policy.projectionIntent, "lineage");
});

test("buildHeapLanes distributes blocks across stable lanes without duplication", () => {
  const blocks = [
    makeBlock("chart-1", "chart", "Agent Performance Metrics"),
    makeBlock("note-1", "note", "Cortex Architectural Overview", "High level summary"),
    makeBlock("gate-1", "structured_data", "Infrastructure Gate Summary"),
    makeBlock("media-1", "media", "Architecture Diagram Draft"),
    makeBlock("task-1", "task", "Release Preparation Checklist", "Checklist body"),
    makeBlock("widget-1", "widget", "Pending Agent Proposal"),
  ];

  const lanes = buildHeapLanes(blocks, 3, DEFAULT_EXPLORE_LIST_POLICY);
  const assignedIds = lanes.flatMap((lane) =>
    lane.map((block) => block.projection.artifactId)
  );

  assert.equal(lanes.length, 3);
  assert.equal(assignedIds.length, blocks.length);
  assert.deepEqual([...assignedIds].sort(), [
    "chart-1",
    "gate-1",
    "media-1",
    "note-1",
    "task-1",
    "widget-1",
  ]);
  assert.equal(new Set(assignedIds).size, blocks.length);
  assert.ok(lanes.every((lane) => lane.length >= 1));
  const laneLengths = lanes.map((lane) => lane.length);
  assert.ok(Math.max(...laneLengths) - Math.min(...laneLengths) <= 1);
});
