---
id: 029
name: ai-chat-template-integration
title: 'Research: Interactive Agent Chat (A2UI Integration)'
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-03-28'
---

# Research: Interactive Agent Chat (A2UI Integration)

**Date**: 2026-01-17
**Status**: ACTIVE, PARTIALLY IMPLEMENTED
**Context**: Enable AI agents within Cortex chat to utilize A2UI-based interfaces, workflow builder modules, and D3 graph representations directly in the conversation stream.

> [!NOTE]
> **Resolved with [028 A2UI Integration](../028-a2ui-integration-feasibility/RESEARCH.md)**: This initiative now focuses on *implementing* the A2UI protocol within the chat surface, rather than inventing a new embed protocol.

> [!NOTE]
> **Implementation Update (2026-03-28)**: `cortex-web` now ships a production-aligned v1 conversation surface. `/ws/chat` remains a thin transport, the gateway resolves canonical heap context bundles and persisted thread history, turns are persisted as heap-backed conversation blocks, `/conversations` reads server-backed projections, and generation dispatches to the provider-runtime Responses path first with `workflow-engine` retained only as compatibility fallback.

---

## 1. Executive Summary

This research initiative defines the **Chat Surface** implementation of the A2UI (Abstract Agent UI) protocol. By adopting A2UI, Cortex chat becomes a dynamic canvas where agents can stream interactive applications—wizards, forms, dashboards, and graphs—interleaved with natural language.

We validate this approach by resolving capabilities across multiple research tracks:
- **028 A2UI**: Specifies the rendering protocol and JSON schema.
- **029 (This)**: Specifies how Chat *consumes* A2UI to enable interactive modeling.
- **019 Logs**: Provides real-time debugging streams within chat.
- **023 Graphs**: Provides the visual component for structural modeling.

---

## 2. Core Questions

1. **How does Chat host A2UI surfaces?** (Ephemeral vs. Persistent)
2. **How do we handle "Multi-Modal" messages?** (Text + A2UI Payload)
3. **How do generic A2UI components interact with specialized Nostra capabilities (D3)?**
4. **How does chat facilitate "Workflow Module" installation from natural language?**

---

## 3. Resolution & Integration Strategy

### 3.1 Adopting A2UI Protocol (from 028)

Instead of custom markdown markers (e.g., `:::graph{...}:::`), chat messages will carry **A2UI Payloads**.

| Concept | Old Manual Approach | A2UI Approach (New) |
|:---|:---|:---|
| **Protocol** | Custom JSON string | Standard `surfaceUpdate` / `dataModelUpdate` |
| **Renderer** | `ChatEmbedRenderer` | Shared React `A2UIInterpreter` in `cortex-web` |
| **Components** | Hardcoded components | `WidgetRegistry` lookups |
| **State** | Ephemeral/Lost | Managed via `dataModel` |

**Benefit**: Any agent capable of speaking A2UI (e.g., the Workflow Builder agent) works instantly in Chat without modification.

### 3.2 Visualizing "Hidden" Data (from 019 Logs)

Chat is often "black box." By integrating **019 Log Registry**, we can offer a "Show Logs" toggle on any AI action.

*   **UseCase**: "Why did my workflow fail?"
*   **Response**: AI embeds a `LogStream` widget (A2UI component) filtered to the specific `trace_id` of that workflow execution.

### 3.3 The "Installer" Pattern (from 013 Modules)

Chat becomes the primary interface for installing **Workflow Modules**.

*   **User**: "I need a team to build a React app."
*   **AI**: Explores `013` Modules, finds "Software Dev Cycle".
*   **Action**: Renders an **A2UI Wizard** to configure the module (Select repositories, assign users to roles) before "Installing" (Minting agents).

---

## 4. Use Cases (Resolved)

### 4.1 The "Generative Wizard"

**User**: "Start a nonprofit for saving stray cats."

**AI Agent**:
1.  **Thinking**: Analyzes request, selects `org-template-nonprofit`.
2.  **Action**: Generates an A2UI `beginRendering` message.
3.  **Chat UI**: Renders a multi-step Wizard card *inline*.

```json
// A2UI Payload (Simplified)
{
  "surfaceId": "msg-123-wizard",
  "components": [
    { "id": "root", "type": "Card", "child": "col1" },
    { "id": "col1", "type": "Column", "children": ["title", "name_input", "next_btn"] },
    { "id": "title", "type": "Text", "text": "Setup 'Cat Rescue' Non-Profit" },
    { "id": "name_input", "type": "TextField", "label": "Organization Name" }
  ]
}
```

### 4.2 The "Graph Explorer" (D3 Integration)

**User**: "Show me connections between 'Project X' and 'Alice'."

**AI Agent**:
1.  **Tool**: Calls `render_graph(query="...")`.
2.  **Response**: Sends A2UI message with a custom `NostraGraph` widget.
3.  **Chat UI**: The shared React A2UI interpreter encounters `type: "NostraGraph"` and looks up the implementation from the registered catalog (023).

### 4.3 The "Debugger" (Log Integration)

**User**: "The 'Release' task is stuck."

**AI Agent**:
1.  **Tool**: Checks workflow status. Finds "Blocked".
2.  **Response**:
    *   Text: "The release is waiting for approval from @bob."
    *   Embed: A2UI `LogViewer` widget showing the last 5 events for context.
    *   Action: A2UI `Button` labeled "Nudge Bob" (triggers notification).

---

## 5. Technical Architecture

### 5.1 Chat Message Structure

The shipped v1 surface uses a mixed-content envelope at the transport edge and persists each message as canonical heap-backed block parts.

```ts
type ChatMessagePart =
  | { type: "text"; text: string }
  | { type: "a2ui"; surfaceId: string; title?: string; tree: Json }
  | { type: "pointer"; href: string; label: string; artifactId?: string; description?: string };

type ChatClientEnvelope = {
  type: "message";
  content: ChatMessagePart[];
  threadId?: string;
  spaceId?: string;
  context?: {
    blockIds?: string[];
    sourceAnchor?: ChatConversationAnchor;
  };
};
```

Persistence is block-native rather than message-native:
- text turns persist as `conversation_message` rich-text blocks
- inline interfaces persist as `conversation_a2ui` blocks
- citations and links persist as `conversation_pointer` blocks
- thread history is rehydrated from a server-backed projection over those persisted blocks

### 5.2 The Renderer Bridge

```
┌──────────────────┐      ┌──────────────────────────┐
│ ChatHistory (UI) │ ───> │ A2UIRenderer (Component) │
└────────┬─────────┘      └────────────┬─────────────┘
         │                             │
   [Msg: "Here is the form"]           │ lookup("NostraGraph")
   [Embed: Surface #123] ──────────────┤
                                       ▼
                              ┌──────────────────┐
                              │ D3Graph (Native) │
                              └──────────────────┘
```

---

## 6. Recommendations

1.  **Use the shared renderer only**: Chat must not ship a "lite" embed path; `cortex-web` uses the shared React A2UI interpreter.
2.  **Preserve runtime-first dispatch**: Provider-runtime Responses is the canonical generator path; canister fallback is compatibility-only.
3.  **Advance structured output**: The next increment should let runtime agents emit first-class A2UI payloads and action events instead of only text plus deterministic context cards.
4.  **Complete attachment grounding**: Re-enable attachment UX only when artifact-backed or byte-backed ingestion is available to the runtime.

---

## 7. Cross-Research Impact Matrix

| Research | Impact on 029 |
|:---|:---|
| **028 A2UI** | **Foundational**. Defines the protocol and renderer we will use. |
| **023 D3 Lab** | **Component Provider**. Provides the D3 logic wrapped in an A2UI widget. |
| **019 Log Registry** | **Component Provider**. Provides the `LogService` and `LogStream` widget. |
| **013 Workflow** | **Content Provider**. Workflows define the *forms* A2UI renders. |
| **005 Design Design** | **Style Guide**. Chat embeds must respect glassmorphism/theming. |

---

## 8. Remaining Gaps

1.  **Model-authored A2UI**: v1 renders inline A2UI, but most structured UI is still server-authored context summary output rather than arbitrary model-generated interface trees.
2.  **Historical action validity**: Old conversation surfaces can be re-rendered, but full action replay and expiration semantics are not yet formalized.
3.  **Attachment bytes**: The surface correctly avoids promising unsupported descriptor-only attachment reasoning, but true file-grounded chat is still pending.

---

## 9. Polymorphic Block Resolution (2026-02-25)

> Alignment with Initiative 124: Universal Polymorphic Block.

### 9.1 ChatMessage Simplification

The `ChatContent` enum from Section 5.1 is superseded. Chat messages are now a stream of **Polymorphic Blocks**:

```rust
// Supersedes Section 5.1 ChatContent / ChatMessage
struct ChatMessage {
    id: String,
    blocks: Vec<PolymorphicBlockRef>,  // Ordered block IDs
    sender: Principal,
    timestamp: Timestamp,
}
```

Each block in the message uses the canonical `EMIT_HEAP_BLOCK` schema:
- **Conversational text** → `payload_type: rich_text`
- **Workflow wizard** → `payload_type: a2ui`
- **Entity citation** → `payload_type: pointer`
- **Log stream embed** → `payload_type: a2ui` (component: `LogViewer`)

### 9.2 Resolved Open Questions

| Question (Section 8) | Resolution |
|:---|:---|
| Save A2UI JSON in history? | **Yes.** Blocks are persisted via the canonical heap CRDT; chat history is a projection over block IDs. |
| State sync for old messages? | **Resolved.** Blocks carry `payload_type` metadata enabling read-only fallback rendering for expired surfaces. |
| Canonical generator path? | **Resolved in v1.** `cortex-eudaemon` dispatches chat to the provider-runtime Responses path first and uses `workflow-engine` only as compatibility fallback. |
| Is `/conversations` authoritative? | **No.** `/conversations` is a transitional projection over canonical server-backed conversation history, not a separate client-owned authority. |

## 10. Implemented V1 Alignment (2026-03-28)

The current `cortex-web` / `cortex-eudaemon` implementation now realizes the production-aligned v1 chat path described by this research:

1. **Canonical transport**: `cortex-web` uses `/ws/chat` as the live chat transport.
2. **Runtime dispatch**: `cortex-eudaemon` resolves canonical heap context bundles and thread history, then dispatches to the provider-runtime Responses path as the primary chat generator. `workflow-engine.process_message` remains compatibility fallback only.
3. **Persistence**: User and agent turns persist as heap-backed conversation blocks:
   - `conversation_message` for rich text
   - `conversation_a2ui` for A2UI surfaces
   - `conversation_pointer` for citations and anchors
4. **Projection**: `/api/cortex/chat/conversations` and `/api/cortex/chat/conversations/:threadId` expose server-backed conversation projections. Browser `localStorage` is cache only, not source of truth.
5. **Rendering**: Chat bubbles render mixed text, A2UI, and pointer content inline through the shared React A2UI interpreter in `cortex-web`.

### Remaining Gaps

1. Runtime responses are still text-first; freeform model-authored A2UI remains a follow-up slice.
2. Descriptor-only binary attachment reasoning is intentionally not surfaced until artifact-backed bytes/context are available.
3. General post-message A2UI action routing remains governed by the shared A2UI action path and is not yet expanded into a broader chat-specific action model.
