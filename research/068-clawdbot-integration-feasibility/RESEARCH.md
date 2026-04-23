---
id: 068
name: clawdbot-integration-feasibility
title: 'Research: Clawdbot Integration Feasibility'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Clawdbot Integration Feasibility

## Metadata
- **ID**: 068
- **Title**: Clawdbot
- **Author**: Antigravity
- **Status**: ⏸️ MAPPED (See Research 004 / 067)
- **Date**: 2024-12-14
- **Created**: 2026-01-25
- **Related**: 067-unified-protocol, 013-nostra-workflow-engine, 046-nostra-system-standards

> [!NOTE]
> **Resolution**: This initiative is mapped to the **Control Layer** of the [Unified Protocol](../067-unified-protocol/README.md).
> Implementation is deferred until **Phase 2** of the [Execution Plan](../066-temporal-boundary/PLAN.md).
>
> **See**: [Missing Links Report](../004-unified-architecture-gaps/MISSING_LINKS_REPORT.md) for the prioritization strategy.

---

## Executive Summary
This deep dive analyzes Clawdbot's internal architecture to identify patterns, standards, and capabilities that should be integrated into Nostra/Cortex. The goal is **not** to run Clawdbot as a service, but to **absorb its architectural wins** into our native Rust/Motoko stack, specifically identifying gaps in our current `nostra-gateway`, `nostra-worker`, and Agent Protocol structures.

## Architectural Mapping & Gaps

| Feature Area | Clawdbot Implementation | Nostra/Cortex Equivalent | Status / Gap |
| :--- | :--- | :--- | :--- |
| **Control Plane** | `src/gateway` (WebSocket Server) | `nostra-gateway` + ICP Canisters | **Gap**: Clawdbot's local WebSocket gateway handles *device-local* events (camera, screen) which are missing in our cloud-centric model. We need a **Local Control Plane** in Cortex Desktop. |
| **Agent Runtime** | `src/agents` (Pi Agent / System Prompt) | `nostra-workflow-engine` | **Gap**: Clawdbot uses a **dynamic system prompt builder** (`system-prompt.ts`) that adapts to available tools. Nostra currently lacks a standard for *dynamic* agent context generation based on capabilities. |
| **Modularity** | `src/plugin-sdk` (Typed Zod Schemas) | ICP Canister Interfaces (Candid) | **Partial Gap**: We have Candid for inter-canister calls, but we lack a **Unified Plugin SDK** for *local* client capabilities (e.g., extending the Desktop App). |
| **Communication** | `src/auto-reply` (Dispatch Logic) | `nostra-worker` Job Queue | **Gap**: Clawdbot has extensive logic for **Message Normalization** (unifying Slack/WhatsApp/etc. into a single `MsgContext`). Nostra needs a standard **"Rogue Message" Normalization Layer**. |
| **Subagents** | `src/agents/subagent-registry.ts` | Agent-to-Agent Protocol | **Critical Gap**: Clawdbot has a native, local registry for spawning and tracking sub-tasks (`subagentRuns`). Nostra needs a local **Task Registry** for sub-agents running on the user's machine. |

## Detailed Analysis

### 1. The Gateway Pattern (Local vs Cloud)
Clawdbot runs a local WebSocket server (`gateway/server.ts`) that acts as the "nervous system" for the user's device.
*   **Implication for Nostra**: `nostra-apps/cortex-desktop` must implement a similar **Local WebSocket Gateway** to allow local tools (VS Code extension, Browser Extension) to talk to the local Agent Runtime without round-tripping to the ICP blockchain for every keystroke.
*   **Action**: Create a spec for `Cortex Local Gateway` in `067-unified-protocol`.

### 2. The "Pi Agent" Loop
Clawdbot's agent loop (`auto-reply/reply/get-reply.ts`) is highly robust:
1.  **Normalization**: Converts any channel input to `MsgContext`.
2.  **Directive Resolution**: Checks for "Directives" (commands like `/reset`, `/status`).
3.  **Prompt Construction**: Builds the system prompt using `system-prompt.ts`.
4.  **Sandbox Staging**: Moves files into a Docker sandbox if needed.
5.  **Execution**: Runs the LLM loop.
*   **Gap**: Nostra's `workflow-engine` focuses on long-running Sagas but lacks this " conversational/interactive" session loop for immediate user interaction.

### 3. Dynamic System Prompts
`system-prompt.ts` is a standout feature. It dynamically assembles the agent's "Soul" based on:
*   Available Tools (filtered by policy)
*   User Identity
*   Runtime Capabilities (e.g., "Can I react to messages?")
*   Installed Skills
*   **Adoption**: Cortex needs a `PromptBuilder` trait that standardize how agents construct their self-image from their capabilities.

### 4. Plugin SDK & Schema
Clawdbot uses `zod` schemas in `plugin-sdk` to enforce strict contracts for extensions.
*   **Adoption**: We should map these Zod schemas to **Candid** interfaces for our "Nostra Modules". The `clawdbot.plugin.json` manifest is equivalent to our canister metadata.

## Integration Plan (The "Absorb" Strategy)

We will integrate these features into Nostra by creating their **Rust equivalents**:

1.  **Cortex Local Gateway**: Implement a WebSocket server in `cortex-desktop` (Rust) that mimics Clawdbot's `gateway/server.ts` logic for handling local tool connections.
2.  **Prompt Builder**: Port the logic of `system-prompt.ts` to a Rust library `nostra-prompt-builder` for use by Cortex agents.
3.  **Subagent Registry**: Implement a local `sqlite` or `sled` based registry in Cortex to track local sub-agent runs, mirroring `subagent-registry.ts`.
4.  **Message Normalization**: Define a standard `NostraMessage` struct that is a superset of Clawdbot's `MsgContext`, ensuring we can handle arbitrary inputs (Slack, Matrix, etc.) uniformly.
