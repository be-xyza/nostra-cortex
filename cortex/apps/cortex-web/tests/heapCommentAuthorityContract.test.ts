import assert from "node:assert/strict";
import test from "node:test";

import {
  buildLocalHeapComment,
  HEAP_COMMENT_AUTHORITY_CONTRACT,
  isDurableHeapComment,
} from "../src/components/heap/heapCommentAuthority.ts";

test("heap comments are explicitly local annotations, not durable evidence", () => {
  assert.deepEqual(HEAP_COMMENT_AUTHORITY_CONTRACT, {
    persistence: "local_ui_state",
    durableEvidence: false,
    governedHeapRecord: false,
    exportableAsEvidence: false,
    recommendedPersistenceTarget: "undecided",
  });
});

test("buildLocalHeapComment stamps local-only authority metadata", () => {
  const comment = buildLocalHeapComment({
    id: "comment-1",
    author: "operator.alex",
    text: "  Keep this as a local note.  ",
    createdAt: "2026-04-29T00:00:00Z",
  });

  assert.equal(comment.text, "Keep this as a local note.");
  assert.equal(comment.authority?.persistence, "local_ui_state");
  assert.equal(isDurableHeapComment(comment), false);
});
