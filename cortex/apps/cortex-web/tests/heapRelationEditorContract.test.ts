import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapRelationUpsertRequest,
  createInitialHeapRelationDraft,
  buildMinimalHeapBlockRequest,
} from "../src/components/heap/heapRelationEditor.ts";

const baseBlock = {
  projection: {
    artifactId: "artifact-alpha",
    workspaceId: "space-alpha",
    blockId: "block-alpha",
    title: "Alpha Block",
    blockType: "note",
    updatedAt: "2026-03-12T10:00:00Z",
    emittedAt: "2026-03-12T09:00:00Z",
    tags: ["tag-one"],
    mentionsInline: ["mention-one"],
    pageLinks: ["link-one"],
    attributes: {
      status: "draft",
    },
  },
  surfaceJson: {
    plain_text: "Alpha body",
  },
} as const;

test("createInitialHeapRelationDraft reflects the current projected relations", () => {
  const draft = createInitialHeapRelationDraft(baseBlock as never);

  assert.deepEqual(draft.tags, ["tag-one"]);
  assert.deepEqual(draft.pageLinks, ["link-one"]);
  assert.deepEqual(draft.mentions, [
    { artifactId: "mention-one", label: "mention-one" },
  ]);
});

test("buildHeapRelationUpsertRequest preserves heap identity and maps relation edits", () => {
  const request = buildHeapRelationUpsertRequest({
    block: baseBlock as never,
    relationDraft: {
      tags: ["tag-two", "tag-one", "tag-two"],
      mentions: [
        { artifactId: "mention-two", label: "Mention Two" },
        { artifactId: "mention-two", label: "Mention Two" },
      ],
      pageLinks: ["link-two", "link-one", "link-two"],
    },
    emittedAt: "2026-03-12T11:00:00Z",
    agentId: "cortex-web.relations",
  });

  assert.equal(request.space_id, "space-alpha");
  assert.equal(request.block.id, "block-alpha");
  assert.equal(request.block.type, "note");
  assert.equal(request.block.title, "Alpha Block");
  assert.deepEqual(request.block.attributes, { status: "draft" });
  assert.equal(request.source.agent_id, "cortex-web.relations");
  assert.equal(request.source.emitted_at, "2026-03-12T11:00:00Z");
  assert.equal(request.content.payload_type, "rich_text");
  assert.equal(request.content.rich_text?.plain_text, "Alpha body");
  assert.deepEqual(request.relations?.tags, [
    { to_block_id: "tag-two" },
    { to_block_id: "tag-one" },
  ]);
  assert.deepEqual(request.relations?.mentions, [
    { to_block_id: "mention-two", label: "Mention Two" },
  ]);
  assert.deepEqual(request.relations?.page_links, [
    { to_block_id: "link-two" },
    { to_block_id: "link-one" },
  ]);
  assert.deepEqual(request.crdt_projection, {
    artifact_id: "artifact-alpha",
  });
});

test("buildHeapRelationUpsertRequest preserves pointer payloads for non-rich-text blocks", () => {
  const request = buildHeapRelationUpsertRequest({
    block: {
      projection: {
        artifactId: "artifact-pointer",
        workspaceId: "space-alpha",
        blockId: "block-pointer",
        title: "Pointer Block",
        blockType: "upload",
        updatedAt: "2026-03-12T10:00:00Z",
        tags: [],
        mentionsInline: [],
        pageLinks: [],
      },
      surfaceJson: {
        pointer: "local://uploads/doc.pdf",
      },
    } as never,
    relationDraft: {
      tags: [],
      mentions: [],
      pageLinks: [],
    },
    emittedAt: "2026-03-12T11:00:00Z",
  });

  assert.equal(request.content.payload_type, "pointer");
  assert.equal(request.content.pointer, "local://uploads/doc.pdf");
  assert.equal(request.content.rich_text, undefined);
});

test("buildMinimalHeapBlockRequest constructs a placeholder block request", () => {
  const request = buildMinimalHeapBlockRequest(
    "new-target-id",
    "test-workspace",
    "2026-03-12T12:00:00Z",
    "test-agent"
  );

  assert.equal(request.schema_version, "1.0.0");
  assert.equal(request.mode, "heap");
  assert.equal(request.space_id, "test-workspace");
  assert.equal(request.source.agent_id, "test-agent");
  assert.equal(request.source.emitted_at, "2026-03-12T12:00:00Z");
  assert.equal(request.block.type, "note");
  assert.equal(request.block.title, "new-target-id");
  assert.equal(request.content.payload_type, "rich_text");
  assert.equal(request.content.rich_text?.plain_text, "Placeholder block for new-target-id");
  assert.deepEqual(request.crdt_projection, { artifact_id: "new-target-id" });
});
