import assert from "node:assert/strict";
import test from "node:test";

import {
  applyChatServerEnvelope,
  buildChatClientMessageEnvelope,
  buildUserChatPanelMessage,
  normalizeChatErrorMessage,
  type ChatPanelMessage,
} from "../src/components/heap/chatSocketProtocol.ts";
import { resolveChatHints } from "../src/components/heap/chatHintRegistry.ts";

test("buildChatClientMessageEnvelope preserves canonical context input", () => {
  assert.deepEqual(
    buildChatClientMessageEnvelope({
      text: "Summarize this",
      contextBlockIds: ["artifact-1"],
      threadId: "thread-1",
      spaceId: "nostra-space",
    }),
    {
      type: "message",
      content: [{ type: "text", text: "Summarize this" }],
      context: { blockIds: ["artifact-1"] },
      threadId: "thread-1",
      spaceId: "nostra-space",
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
    agent: {
      id: "provider",
      label: "Cortex Runtime",
      route: "provider-runtime.responses",
      mode: "runtime",
    },
  });
  assert.equal(streaming.chatState, "streaming");
  assert.equal(streaming.messages.length, 1);
  assert.equal(streaming.messages[0]?.text, "Hello ");
  assert.equal(streaming.messages[0]?.content[0]?.type, "text");

  const finalized = applyChatServerEnvelope(streaming, {
    type: "message",
    id: "chat-1",
    text: "Hello world",
    timestamp: "2026-03-22T12:00:01Z",
    content: [
      { type: "text", text: "Hello world" },
      {
        type: "pointer",
        href: "/explore?artifact_id=artifact-1",
        label: "artifact-1",
      },
    ],
    agent: {
      id: "provider",
      label: "Cortex Runtime",
      route: "provider-runtime.responses",
      mode: "runtime",
    },
  });
  assert.equal(finalized.chatState, "idle");
  assert.equal(finalized.error, null);
  assert.equal(finalized.messages.length, 1);
  assert.equal(finalized.messages[0]?.id, "chat-1");
  assert.equal(finalized.messages[0]?.text, "Hello world");
  assert.equal(finalized.messages[0]?.content.length, 2);
});

test("buildChatClientMessageEnvelope includes source anchors when present", () => {
  assert.deepEqual(
    buildChatClientMessageEnvelope({
      text: "Inspect this view",
      threadId: "thread-42",
      contextBlockIds: ["artifact-1"],
      sourceAnchor: {
        kind: "view",
        label: "Explore",
        href: "/explore",
        routeId: "/explore",
        viewId: "aggregate:prompts",
      },
    }),
    {
      type: "message",
      content: [{ type: "text", text: "Inspect this view" }],
      threadId: "thread-42",
      context: {
        blockIds: ["artifact-1"],
        sourceAnchor: {
          kind: "view",
          label: "Explore",
          href: "/explore",
          routeId: "/explore",
          viewId: "aggregate:prompts",
        },
      },
    },
  );
});

test("applyChatServerEnvelope preserves structured-only assistant replies", () => {
  const state = applyChatServerEnvelope(
    { messages: [], chatState: "processing", error: null },
    {
      type: "message",
      id: "chat-structured",
      text: "Structured response",
      timestamp: "2026-03-28T12:00:01Z",
      content: [
        {
          type: "a2ui",
          surfaceId: "chat_context_summary:thread-42",
          title: "Resolved context bundle",
          tree: { type: "Container", children: { explicitList: [] } },
        },
        {
          type: "pointer",
          href: "/explore?artifact_id=artifact-1",
          label: "Heap Parity Card",
          description: "note · updated 2026-03-28T00:00:00Z",
        },
      ],
      agent: {
        id: "provider",
        label: "Cortex Runtime",
        route: "provider-runtime.responses",
        mode: "runtime",
      },
    },
  );

  assert.equal(state.chatState, "idle");
  assert.equal(state.messages.length, 1);
  assert.equal(state.messages[0]?.text, "Structured response");
  assert.equal(state.messages[0]?.content[0]?.type, "a2ui");
  assert.equal(state.messages[0]?.content[1]?.type, "pointer");
  assert.equal(state.messages[0]?.agent?.route, "provider-runtime.responses");
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

test("normalizeChatErrorMessage softens known IC tooling color panics", () => {
  const message = normalizeChatErrorMessage(
    "gateway_error",
    "CommandFailed: thread 'main' panicked at src/icp/src/main.rs:108:14: Failed to set stderr output color.: ColorOutOfRange",
  );

  assert.match(message, /temporarily unavailable/i);
  assert.match(message, /icp-cli/i);
});

test("normalizeChatErrorMessage softens the terminal color tooling failure message", () => {
  const message = normalizeChatErrorMessage(
    "gateway_error",
    "Local IC tooling failed while formatting terminal colors. This is a known IC tooling issue in this environment, not a heap/chat content failure.",
  );

  assert.match(message, /temporarily unavailable/i);
  assert.match(message, /icp-cli/i);
});

test("resolveChatHints includes context-aware derived view prompts", () => {
  const hints = resolveChatHints("Explore", 0, {
    viewId: "aggregate:prompt-like",
    viewLabel: "Prompts",
    description: "Recent prompts, requests, and solicitations grouped into a compact list.",
    itemCount: 2,
    recentTitles: ["Prompt A", "Prompt B"],
    signals: [
      {
        label: "Newest request",
        summary: "Prompt A",
        prompt: "Review the newest prompt block 'Prompt A' and summarize its request shape, including role, authority, and budget when present.",
      },
    ],
  });

  assert.equal(hints[0]?.label, "Prompt A");
  assert.match(hints[0]?.prompt ?? "", /request shape/i);
});

test("buildUserChatPanelMessage creates a canonical text part", () => {
  const message = buildUserChatPanelMessage({
    id: "user-1",
    text: "Inspect this block",
    timestamp: "2026-03-22T12:00:00Z",
    contextRefs: ["artifact-1"],
  });

  assert.equal(message.text, "Inspect this block");
  assert.deepEqual(message.content, [{ type: "text", text: "Inspect this block" }]);
  assert.deepEqual(message.contextRefs, ["artifact-1"]);
});
