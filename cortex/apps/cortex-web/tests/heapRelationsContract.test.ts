import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapRelationIndex,
  resolveHeapRelationBlock,
} from "../src/components/heap/heapRelations.ts";

type MockBlock = {
  projection: {
    artifactId: string;
    title: string;
    blockType: string;
    tags?: string[];
    pageLinks?: string[];
    mentionsInline?: string[];
  };
};

const blocks: MockBlock[] = [
  {
    projection: {
      artifactId: "alpha",
      title: "Alpha",
      blockType: "note",
      tags: ["system"],
      pageLinks: ["beta"],
      mentionsInline: ["gamma"],
    },
  },
  {
    projection: {
      artifactId: "beta",
      title: "Beta",
      blockType: "task",
      tags: ["system"],
      pageLinks: [],
      mentionsInline: ["alpha"],
    },
  },
  {
    projection: {
      artifactId: "gamma",
      title: "Gamma",
      blockType: "note",
      tags: ["docs"],
      pageLinks: ["alpha"],
      mentionsInline: [],
    },
  },
];

test("heap relation index derives backlinks, outbound relations, and tag neighbors", () => {
  const relationIndex = buildHeapRelationIndex(blocks[0] as never, blocks as never[]);

  assert.deepEqual(relationIndex.outboundLinks.map((item) => item.id), ["beta"]);
  assert.deepEqual(relationIndex.outboundMentions.map((item) => item.id), ["gamma"]);
  assert.deepEqual(relationIndex.backlinks.map((item) => item.id), ["beta", "gamma"]);
  assert.deepEqual(relationIndex.tagNeighbors.map((item) => item.id), ["beta"]);
});

test("heap relation resolver finds blocks by artifact id", () => {
  assert.equal(resolveHeapRelationBlock("gamma", blocks as never[])?.projection.title, "Gamma");
  assert.equal(resolveHeapRelationBlock("missing", blocks as never[]), null);
});
