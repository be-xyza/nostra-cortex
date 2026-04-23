---
id: 029
name: ai-chat-template-integration
title: 'Requirements: Interactive Agent Chat (A2UI Integration)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Interactive Agent Chat (A2UI Integration)

**Status**: PROPOSED
**Version**: 2.0 (Resolved)
**Context**: Technical specifications for the Chat Surface implementation of the A2UI Protocol.

---

## Functional Requirements

### FR-01: A2UI Payload Support

The Chat System MUST support the ingestion and storage of A2UI JSONL payloads.

*   **Input**: Agents (Backend/Worker) emit `SurfaceUpdate` messages.
*   **Storage**: `ChatMessage` struct must accommodate `Variant::A2UISurface(PayloadID)`.
*   **Transport**: WebSocket/Query response must include full JSONL payload.

### FR-02: Inline Rendering (The "Surface Host")

The Chat UI MUST render A2UI surfaces inline with text.

*   **Constraint**: Embeds must be contained within the message bubble width (max-width constraints).
*   **Isolation**: CSS/Styling of the embed must not bleed into the global chat layout, but *should* inherit the global theme (005 Design).
*   **A2UIRenderer**: Must use the shared renderer from `028`. Custom "lite" renderers are forbidden.

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

*   **Local**: Simple state changes (expanding a section) handled by Dioxus client.
*   **Remote**: `UserAction` events must be dispatched back to the Agent/Backend via `client_to_server` protocol (028).

---

## Non-Functional Requirements

### NFR-01: Style Consistency (005 Design)

*   **Glassmorphism**: Embed backgrounds should use `bg-card/50 backdrop-blur-sm`.
*   **Typography**: Inherit `Inter` font stack.
*   **Colors**: Use semantic `primary`, `secondary`, `destructive` tokens.

### NFR-02: Performance

*   **Lazy Loading**: Components (especially D3) must not initialize until scrolled into view.
*   **State Retention**: Navigating away from Chat tab and back should preserve form state (via A2UI `dataModel`).

---

## API Contracts

### Message Content Variant

```rust
pub enum MessageContent {
   Text(String),
   A2UI {
       surface_id: String,
       initial_payload: String, // JSON
   }
}
```

### Agent Tool Output

When an agent wants to show a UI, it returns a tool result containing the A2UI payload:

```json
{
  "tool": "render_interface",
  "result": {
    "type": "a2ui_payload",
    "surfaceId": "unique-id",
    "payload": { ... }
  }
}
```
