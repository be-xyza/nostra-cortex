import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapViewCounts,
  filterHeapBlocksByView,
  normalizeHeapPrimaryViewMode,
  type HeapPrimaryViewMode,
} from "../src/components/heap/heapViewModel.ts";

type MockBlock = {
  pinnedAt?: string | null;
  projection: {
    artifactId: string;
    tags?: string[];
    pageLinks?: string[];
    mentionsInline?: string[];
  };
  surfaceJson?: {
    behaviors?: string[];
  };
};

const blocks: MockBlock[] = [
  {
    projection: {
      artifactId: "block-unlinked",
      tags: [],
      pageLinks: [],
      mentionsInline: [],
    },
    surfaceJson: { behaviors: [] },
  },
  {
    projection: {
      artifactId: "block-sorted",
      tags: ["project"],
      pageLinks: ["space-alpha"],
      mentionsInline: [],
    },
    surfaceJson: { behaviors: [] },
  },
  {
    pinnedAt: "2026-03-12T00:00:00Z",
    projection: {
      artifactId: "block-pinned",
      tags: [],
      pageLinks: [],
      mentionsInline: ["block-sorted"],
    },
    surfaceJson: { behaviors: ["pinned", "urgent"] },
  },
];

test("heap view counts align to all, unlinked, sorted, and pinned semantics", () => {
  const counts = buildHeapViewCounts(blocks as never[]);

  assert.deepEqual(counts, {
    All: 3,
    Unlinked: 1,
    Sorted: 2,
    Pinned: 1,
    Urgent: 1,
  });
});

test("heap view filtering treats sorted as linked content and unlinked as inbox content", () => {
  const views: HeapPrimaryViewMode[] = ["All", "Unlinked", "Sorted", "Pinned"];

  const idsByView = Object.fromEntries(
    views.map((view) => [
      view,
      filterHeapBlocksByView(blocks as never[], view).map((block) => block.projection.artifactId),
    ]),
  );

  assert.deepEqual(idsByView.All, ["block-unlinked", "block-sorted", "block-pinned"]);
  assert.deepEqual(idsByView.Unlinked, ["block-unlinked"]);
  assert.deepEqual(idsByView.Sorted, ["block-sorted", "block-pinned"]);
  assert.deepEqual(idsByView.Pinned, ["block-pinned"]);
});

test("heap view normalization accepts canonical and URL-friendly aliases", () => {
  assert.equal(normalizeHeapPrimaryViewMode("sorted"), "Sorted");
  assert.equal(normalizeHeapPrimaryViewMode("pins"), "Pinned");
  assert.equal(normalizeHeapPrimaryViewMode("UNLINKED"), "Unlinked");
  assert.equal(normalizeHeapPrimaryViewMode("unknown"), "All");
  assert.equal(normalizeHeapPrimaryViewMode(null), "All");
});
