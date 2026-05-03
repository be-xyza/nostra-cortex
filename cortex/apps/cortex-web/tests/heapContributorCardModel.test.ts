import assert from "node:assert/strict";
import test from "node:test";

import { buildHeapContributorCardModel, buildHeapContributorDetailModel } from "../src/components/heap/heapContributorCardModel.ts";
import type { HeapBlockListItem } from "../src/contracts.ts";

function block(blockType: string, title: string, text: string, tags: string[] = []): HeapBlockListItem {
  return {
    projection: {
      artifactId: `${blockType}-1`,
      title,
      blockType,
      updatedAt: "2026-03-22T07:14:00Z",
      emittedAt: "2026-03-22T07:14:00Z",
      tags,
      mentionsInline: [],
      pageLinks: [],
    },
    surfaceJson: {
      payload_type: "structured_data",
      text,
    },
  };
}

test("contributor card model replaces raw placeholder usage report copy", () => {
  const model = buildHeapContributorCardModel(block("usage_report", "usage_report block", "usage report block"));

  assert.equal(model.displayTitle, "Activity summary");
  assert.equal(model.plainSummary, "System activity was recorded for this Space. Open details for the technical record.");
  assert.equal(model.friendlyTypeLabel, "Activity summary");
  assert.equal(model.statusLabel, "System update");
  assert.equal(model.sourceLabel, "Runtime monitor");
  assert.equal(model.relevanceLabel, "Background");
  assert.equal(model.digestable, true);
});

test("contributor card model maps agent and proposal records into contributor language", () => {
  const agent = buildHeapContributorCardModel(block("agent_execution_record", "agent_execution_record block", "agent execution record block"));
  const proposal = buildHeapContributorCardModel(block("self_optimization_proposal", "self_optimization_proposal block", "self optimization proposal block"));

  assert.equal(agent.displayTitle, "Agent work update");
  assert.equal(agent.sourceLabel, "Eudaemon");
  assert.equal(agent.relevanceLabel, "Informational");
  assert.equal(proposal.displayTitle, "Suggested improvement");
  assert.equal(proposal.statusLabel, "Suggestion");
  assert.equal(proposal.relevanceLabel, "Review recommended");
});

test("contributor card model preserves meaningful evidence titles while softening jargon", () => {
  const model = buildHeapContributorCardModel(block(
    "note",
    "Eudaemon Alpha authorized publication proof",
    "Operator-approved evidence note resolved a verified operator principal and emitted exactly one bounded rich-text.",
    ["evidence"],
  ));

  assert.equal(model.displayTitle, "Eudaemon Alpha authorized publication proof");
  assert.equal(model.friendlyTypeLabel, "Evidence note");
  assert.equal(model.statusLabel, "Evidence");
  assert.equal(model.sourceLabel, "Operator review");
  assert.match(model.plainSummary, /verified operator/);
  assert.doesNotMatch(model.plainSummary, /principal|bounded rich-text/);
});

test("contributor detail model answers the contributor review questions", () => {
  const usage = buildHeapContributorDetailModel(block("usage_report", "usage_report block", "usage report block"));
  const proposal = buildHeapContributorDetailModel(block("self_optimization_proposal", "self_optimization_proposal block", "self optimization proposal block"));

  assert.match(usage.whatHappened, /System activity was recorded/);
  assert.match(usage.whyItMatters, /quick pulse/i);
  assert.match(usage.nextStep, /background context/i);
  assert.match(proposal.whyItMatters, /human judgment/i);
  assert.match(proposal.nextStep, /Review the proposal/i);
});
