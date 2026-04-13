import assert from "node:assert/strict";
import test from "node:test";

import {
  PREVIEW_ARTIFACT_IDS,
  filterPreviewDeletedBlocks,
  filterPreviewHeapBlocks,
  isPreviewArtifactId,
} from "../src/store/previewFixtureCatalog.ts";

test("preview fixture catalog recognizes seeded preview artifact ids", () => {
  assert.ok(PREVIEW_ARTIFACT_IDS.has("mock-solicitation-1"));
  assert.equal(isPreviewArtifactId("mock-solicitation-1"), true);
  assert.equal(isPreviewArtifactId("01KM4CDYTP8RD94Z52HJQQNTTD"), false);
});

test("preview fixture filters remove seeded mock blocks from heap responses", () => {
  const blocks = [
    {
      projection: {
        artifactId: "mock-solicitation-1",
        title: "Preview block",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:05:00Z",
        tags: [],
        mentionsInline: [],
      },
    },
    {
      projection: {
        artifactId: "01KM4CDYTP8RD94Z52HJQQNTTD",
        title: "Live block",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:06:00Z",
        tags: [],
        mentionsInline: [],
      },
    },
  ] as any;

  const deleted = [
    { artifactId: "mock-solicitation-1", deletedAt: "2026-03-20T01:07:00Z" },
    { artifactId: "01KM4CDYTP8RD94Z52HJQQNTTD", deletedAt: "2026-03-20T01:08:00Z" },
  ];

  assert.deepEqual(
    filterPreviewHeapBlocks(blocks).map((block) => block.projection.artifactId),
    ["01KM4CDYTP8RD94Z52HJQQNTTD"],
  );
  assert.deepEqual(
    filterPreviewDeletedBlocks(deleted).map((block) => block.artifactId),
    ["01KM4CDYTP8RD94Z52HJQQNTTD"],
  );
});
