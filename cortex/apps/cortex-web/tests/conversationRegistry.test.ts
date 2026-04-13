import assert from "node:assert/strict";
import test from "node:test";

import {
  appendConversationTurn,
  buildConversationSourceHref,
  createConversationRecord,
  summarizeConversationText,
  type ConversationAnchor,
} from "../src/components/conversations/conversationRegistry.ts";

test("summarizeConversationText strips markdown and code noise", () => {
  const summary = summarizeConversationText(
    "Here is the plan:\n\n```ts\nconst raw = true;\n```\n\n- keep it simple\n- avoid raw code",
  );

  assert.equal(summary, "Here is the plan: keep it simple avoid raw code");
});

test("createConversationRecord preserves anchors and appends readable turns", () => {
  const anchor: ConversationAnchor = {
    kind: "view",
    label: "Prompts",
    href: "/explore?heap_view=Prompts",
  };

  const record = appendConversationTurn(
    createConversationRecord({
      threadId: "thread-123",
      title: "Prompt review",
      anchor,
      createdAt: "2026-03-22T18:00:00Z",
    }),
    {
      role: "user",
      text: "Review the newest prompt and summarize the requested authority.",
      timestamp: "2026-03-22T18:01:00Z",
    },
  );

  assert.equal(record.threadId, "thread-123");
  assert.equal(record.title, "Prompt review");
  assert.deepEqual(record.anchor, anchor);
  assert.equal(record.messageCount, 1);
  assert.equal(record.lastMessagePreview, "Review the newest prompt and summarize the requested authority.");
});

test("buildConversationSourceHref falls back to the anchor route", () => {
  const href = buildConversationSourceHref({
    kind: "block",
    label: "Pending Agent Proposal",
    href: "/explore?artifact_id=mock-solicitation-1",
    artifactId: "mock-solicitation-1",
  });

  assert.equal(href, "/explore?artifact_id=mock-solicitation-1");
});
