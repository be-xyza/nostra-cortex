import assert from "node:assert/strict";
import test from "node:test";

import {
  applyChatServerEnvelope,
  buildChatClientMessageEnvelope,
  type ChatClientAttachmentDescriptor,
  type ChatPanelMessage,
} from "../src/components/heap/chatSocketProtocol.ts";

test("buildChatClientMessageEnvelope preserves optional context and attachments", () => {
  const attachments: ChatClientAttachmentDescriptor[] = [
    { name: "notes.md", type: "text/markdown", size: 128 },
  ];

  assert.deepEqual(
    buildChatClientMessageEnvelope({
      text: "Summarize this",
      contextRefs: ["artifact-1"],
      attachments,
      threadId: "thread-1",
    }),
    {
      type: "message",
      text: "Summarize this",
      contextRefs: ["artifact-1"],
      attachments,
      threadId: "thread-1",
    },
  );
});

test("applyChatServerEnvelope appends streaming deltas and finalizes the assistant reply", () => {
  const initial: ChatPanelMessage[] = [];

  const processing = applyChatServerEnvelope(
    { messages: initial, chatState: "idle", error: null },
    { type: "processing" },
  );
  assert.equal(processing.chatState, "processing");
  assert.equal(processing.messages.length, 0);

  const streaming = applyChatServerEnvelope(processing, {
    type: "streaming",
    id: "chat-1",
    delta: "Hello ",
    timestamp: "2026-03-22T12:00:00Z",
  });
  assert.equal(streaming.chatState, "streaming");
  assert.equal(streaming.messages.length, 1);
  assert.equal(streaming.messages[0]?.text, "Hello ");

  const finalized = applyChatServerEnvelope(streaming, {
    type: "message",
    id: "chat-1",
    text: "Hello world",
    timestamp: "2026-03-22T12:00:01Z",
  });
  assert.equal(finalized.chatState, "idle");
  assert.equal(finalized.error, null);
  assert.equal(finalized.messages.length, 1);
  assert.equal(finalized.messages[0]?.id, "chat-1");
  assert.equal(finalized.messages[0]?.text, "Hello world");
});

test("applyChatServerEnvelope clears processing state and records errors", () => {
  const state = applyChatServerEnvelope(
    { messages: [], chatState: "processing", error: null },
    { type: "error", code: "gateway_error", message: "Gateway unavailable" },
  );

  assert.equal(state.chatState, "idle");
  assert.equal(state.error, "Gateway unavailable");
  assert.equal(state.messages.length, 0);
});
