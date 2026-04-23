---
id: 029
name: ai-chat-template-integration
title: 'Requirements: Interactive Agent Chat (A2UI Integration)'
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-03-28'
---

# Requirements: Interactive Agent Chat (A2UI Integration)

**Status**: ACTIVE, PARTIALLY IMPLEMENTED
**Version**: 3.0 (Chat v1 Alignment)
**Context**: Technical specifications for the Chat Surface implementation of the A2UI Protocol.

---

## Functional Requirements

### FR-01: Mixed Content Envelope Support

The Chat System MUST support mixed text and structured content in a canonical chat envelope.

*   **Client Request**: `/ws/chat` requests carry `content[]`, `threadId`, optional `spaceId`, and optional canonical chat context.
*   **Input**: Agents/runtime services emit mixed-content chat responses over `/ws/chat`.
*   **Storage**: Conversation turns persist as canonical heap-backed blocks (`conversation_message`, `conversation_a2ui`, `conversation_pointer`).
*   **Transport**: WebSocket responses include mixed content parts (`text`, `a2ui`, `pointer`) and agent route metadata.

### FR-02: Inline Rendering (The "Surface Host")

The Chat UI MUST render A2UI surfaces inline with text.

*   **Constraint**: Embeds must be contained within the message bubble width (max-width constraints).
*   **Isolation**: CSS/Styling of the embed must not bleed into the global chat layout, but *should* inherit the global theme (005 Design).
*   **A2UIRenderer**: Must use the shared React A2UI interpreter from the canonical `cortex-web` substrate. Custom "lite" renderers are forbidden.

### FR-03: Custom Component Support

The renderer MUST support these specific Nostra components (via A2UI `Custom` type):

| Component | Source | Func |
|:---|:---|:---|
| `NostraGraph` | 023 D3 Lab | Interactive D3 Force visualization |
| `WorkflowTrace` | 013 Engine | Step-by-step progress view |
| `LogStream` | 019 Registry | Real-time log/event list |
| `TypeEditor` | 026 Schema | KIP Type form |

### FR-04: Interaction Handling

User interactions within the chat embed (Button clicks, Form submits) MUST be handled correctly.

*   **Local**: Simple state changes (expanding a section) are handled by the shared web host runtime.
*   **Remote**: `UserAction` events must be dispatched back to the Agent/Backend via the canonical chat/runtime transport.

### FR-05: Canonical Runtime Dispatch

The canonical chat generation path MUST dispatch through the Cortex runtime provider layer before any fallback path.

*   **Primary Route**: `cortex-eudaemon` dispatches to the provider-runtime Responses client.
*   **Fallback**: `workflow-engine.process_message` may remain as compatibility fallback only.
*   **Identity**: Final responses include explicit agent identity metadata (`id`, `label`, `route`, `mode`).

### FR-06: Canonical Conversation Projection

The system MUST expose server-backed conversation projections for hydration and browsing.

*   **List Route**: `GET /api/cortex/chat/conversations`
*   **Detail Route**: `GET /api/cortex/chat/conversations/:thread_id`
*   **Frontend Rule**: `/conversations` is a projection UI over canonical server state; browser `localStorage` is cache only.

### FR-07: Attachment Honesty

The frontend MUST not promise unsupported attachment reasoning.

*   **Allowed**: Artifact-backed pointers and persisted upload resources.
*   **Forbidden**: Descriptor-only attachment UI that implies the runtime can reason over bytes it never receives.

---

## Non-Functional Requirements

### NFR-01: Style Consistency (005 Design)

*   **Chat Bubbles**: Mixed content must remain visually bounded by message bubble sizing rules.
*   **Renderer Consistency**: Shared A2UI surfaces must inherit active host theming and policy metadata.
*   **No Renderer Forks**: Chat rendering cannot diverge from the shared host interpreter contract.

### NFR-02: Performance

*   **Lazy Loading**: Components (especially D3) must not initialize until scrolled into view.
*   **State Retention**: Reopening a thread should hydrate canonical persisted history.
*   **Projection Stability**: Rehydrated mixed-content messages must preserve persisted part order.

---

## API Contracts

### Message Content Variant

```rust
pub enum ChatMessagePart {
   Text { text: String },
   A2ui { surface_id: String, title: Option<String>, tree: Value },
   Pointer { href: String, label: String, artifact_id: Option<String>, description: Option<String> },
}
```

### Request Envelope

```json
{
  "type": "message",
  "content": [
    { "type": "text", "text": "Here is the structured result." },
    { "type": "a2ui", "surfaceId": "unique-id", "tree": { ... } }
  ],
  "threadId": "thread-123",
  "spaceId": "meta",
  "context": {
    "blockIds": ["artifact-1"],
    "sourceAnchor": { "kind": "view", "label": "Explore", "href": "/explore?thread=thread-123" }
  }
}
```

## Implementation Status (2026-03-28)

| Requirement | Status | Notes |
|:---|:---|:---|
| FR-01 A2UI Payload Support | **Implemented v1** | Mixed-content `/ws/chat` envelopes and heap-backed conversation persistence are live. |
| FR-02 Inline Rendering | **Implemented v1** | `cortex-web` renders text, A2UI, and pointer content inline with the shared React interpreter. |
| FR-03 Custom Component Support | **Partially implemented** | Shared interpreter path is live; broader widget catalog expansion remains incremental. |
| FR-04 Interaction Handling | **Partial** | Read/render path is live; broader chat-specific action handling remains follow-up work. |
| NFR-02 State Retention | **Implemented v1** | Thread history hydrates from server-backed conversation projection; local storage is cache only. |
