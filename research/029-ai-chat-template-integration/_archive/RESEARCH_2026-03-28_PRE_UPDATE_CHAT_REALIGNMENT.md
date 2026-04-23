---
id: 029
name: ai-chat-template-integration
title: 'Research: Interactive Agent Chat (A2UI Integration)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-25'
---

# Research: Interactive Agent Chat (A2UI Integration)

**Date**: 2026-01-17
**Status**: PROPOSED
**Context**: Enable AI agents within Cortex chat to utilize A2UI-based interfaces, workflow builder modules, and D3 graph representations directly in the conversation stream.

> [!NOTE]
> **Resolved with [028 A2UI Integration](../028-a2ui-integration-feasibility/RESEARCH.md)**: This initiative now focuses on *implementing* the A2UI protocol within the chat surface, rather than inventing a new embed protocol.

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
| **Renderer** | `ChatEmbedRenderer` | Shared `A2UIRenderer` (Dioxus) |
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
3.  **Chat UI**: The `A2UIRenderer` encounters `type: "NostraGraph"` and looks up the D3 implementation from the registered catalog (023).

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

The `ChatMessage` type in specific needs to accommodate A2UI.

```rust
enum ChatContent {
    Text(String),               // Standard Markdown
    A2UISurface(String),        // Surface ID reference
}

struct ChatMessage {
    id: String,
    content: Vec<ChatContent>,  // Mixed text and UI
    a2ui_payload: Option<String>, // The actual JSONL payload (stored separately or inline)
}
```

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

1.  **Wait for 028 Phase 2**: The core A2UI renderer is a prerequisite. Do not build a "lite" version for chat; usage of the shared renderer is mandatory.
2.  **Register Nostra Components**: Ensure `NostraGraph` (D3), `WorkflowTrace` (Engine), and `LogStream` (Registry) are registered in the global `n-catalog.json`.
3.  **Agent-Side Logic**: Update the "Librarian" system prompt to prefer generating A2UI JSON over plain text for structured tasks.

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

## 8. Open Questions

1.  **Persistence**: Do we save the A2UI JSON in chat history? (Rec: Yes, effectively snapshots the UI state at that moment).
2.  **State Sync**: If a user interacts with a form in an old message, is it still valid? (Rec: A2UI allows "expiration" or read-only modes).

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
