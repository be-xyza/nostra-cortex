import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapViewCounts,
  filterHeapBlocksByReviewLane,
  filterHeapBlocksByView,
  readHeapBlockReviewLane,
  type HeapReviewLane,
} from "../src/components/heap/heapViewModel.ts";

type MockBlock = {
  pinnedAt?: string | null;
  deletedAt?: string;
  projection: {
    artifactId: string;
    blockType: string;
    tags?: string[];
    pageLinks?: string[];
    mentionsInline?: string[];
    status?: string;
    attributes?: Record<string, string>;
  };
  surfaceJson?: {
    behaviors?: string[];
  };
};

const reviewBlocks: MockBlock[] = [
  {
    projection: {
      artifactId: "public-review",
      blockType: "space_promotion_request",
      tags: ["proposal"],
      pageLinks: ["space-alpha"],
      mentionsInline: [],
      attributes: { review_lane: "public_review" },
    },
    surfaceJson: { behaviors: [] },
  },
  {
    projection: {
      artifactId: "private-review",
      blockType: "space_promotion_request",
      tags: ["proposal"],
      pageLinks: ["space-beta"],
      mentionsInline: [],
      attributes: { review_lane: "private_review" },
    },
    surfaceJson: { behaviors: [] },
  },
  {
    projection: {
      artifactId: "kickoff-approval",
      blockType: "agent_solicitation",
      tags: ["initiative"],
      pageLinks: ["space-gamma"],
      mentionsInline: [],
      attributes: { review_lane: "private_review" },
    },
    surfaceJson: { behaviors: ["awaiting_approval"] },
  },
  {
    projection: {
      artifactId: "ordinary-note",
      blockType: "note",
      tags: ["note"],
      pageLinks: ["space-alpha"],
      mentionsInline: [],
      attributes: {},
    },
    surfaceJson: { behaviors: [] },
  },
];

test("Inbox includes space promotion requests even when they already have relations", () => {
  const inboxIds = filterHeapBlocksByView(reviewBlocks as never[], "Inbox").map(
    (block) => block.projection.artifactId,
  );

  assert.deepEqual(inboxIds, ["public-review", "private-review", "kickoff-approval"]);
});

test("review lane helpers expose promotion queue lanes and filter them cleanly", () => {
  const allReviewBlocks = filterHeapBlocksByReviewLane(reviewBlocks as never[], null);
  const publicReviewBlocks = filterHeapBlocksByReviewLane(
    reviewBlocks as never[],
    "public_review" satisfies HeapReviewLane,
  );
  const privateReviewBlocks = filterHeapBlocksByReviewLane(
    reviewBlocks as never[],
    "private_review" satisfies HeapReviewLane,
  );

  assert.equal(readHeapBlockReviewLane(reviewBlocks[0] as never), "public_review");
  assert.equal(readHeapBlockReviewLane(reviewBlocks[1] as never), "private_review");
  assert.equal(readHeapBlockReviewLane(reviewBlocks[2] as never), "private_review");
  assert.equal(readHeapBlockReviewLane(reviewBlocks[3] as never), null);
  assert.deepEqual(
    allReviewBlocks.map((block) => block.projection.artifactId),
    ["public-review", "private-review", "kickoff-approval", "ordinary-note"],
  );
  assert.deepEqual(
    publicReviewBlocks.map((block) => block.projection.artifactId),
    ["public-review"],
  );
  assert.deepEqual(
    privateReviewBlocks.map((block) => block.projection.artifactId),
    ["private-review", "kickoff-approval"],
  );
});

test("view counts treat promotion requests as Inbox work even without mentions or urgency", () => {
  const counts = buildHeapViewCounts(reviewBlocks as never[]);

  assert.equal(counts.Inbox, 3);
  assert.equal(counts.Proposals, 1);
  assert.equal(counts.Explore, 4);
  assert.equal(counts.Drafts, 0);
});
