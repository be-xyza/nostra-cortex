# Cortex Execution Layer Specification

## Overview
Cortex is the **execution runtime layer** of Nostra Cortex on the Internet Computer Protocol (ICP). While Nostra acts as the platform authority (defining data, contributions, and governance), Cortex defines how work is executed, rendered, and automated.

This specification documents the boundaries, components, and integration contracts for Cortex.

---

## Architectural Boundaries

Per `DEC-2026-02-17-013` (Nostra & Cortex Canonical Split):
- **Cortex Owns**: Worker execution, AI agents, frontend web/desktop shells, A2UI rendering, external system bridges (git, simulators), and the runtime domain model.
- **Nostra Owns**: The Knowledge Graph, Contribution schemas, Space governance, and Access Control.
- **Contract**: Cortex components interact with Nostra via defined API boundaries and do not bypass platform authority or write directly to protected memory without authorization.

### Provider Runtime and Operator Surfaces
- Cortex owns provider inventory, runtime host inventory, auth bindings, execution bindings, provider discovery diagnostics, and resolved runtime status as execution-infrastructure surfaces.
- Detailed reads for these surfaces are operator-only. Canonical operator contracts are split across:
  - `/api/system/provider-inventory`
  - `/api/system/runtime-hosts`
  - `/api/system/auth-bindings`
  - `/api/system/execution-bindings`
  - `/api/system/provider-discovery`
  - `/api/system/provider-runtime/status`
- `/api/system/providers` remains a compatibility aggregate derived from the canonical split operator reads. It is not the long-term source-of-truth contract.
- Discovery may materialize inventory-only provider records, but execution may bind only providers that satisfy server-side eligibility checks: enabled, execution-routable, supported by the transport/runtime, and backed by satisfiable auth requirements.

---

## Cortex Components

### Applications (`apps/`)
1. **`cortex-desktop`**: The headless intelligent daemon (Temporal worker/gateway). It provides secure native integrations and AI integrations.
2. **`cortex-web`**: The web-based execution host (React/Vite), serving as the primary browser interface and conversation surface.
3. **`cortex-worker`**: Rust-based background workers executing the Serverless Workflow DSL and Temporal durable execution contracts.
4. **`cortex-gateway`**: API gateway handling routing, WebSocket connections, and external traffic.
5. **`cortex-eudaemon`**: Rust runtime host/gateway implementation that currently serves Workbench APIs, `/ws/chat`, provider-runtime dispatch, and heap-backed conversation projection behavior.
6. **`cortex-git-adapter`**: System bridge for synchronizing Nostra state with standard Git repositories.

### Libraries (`libraries/`)
1. **`cortex-domain`**: The core execution data structures, distinct from Nostra's platform definitions.
2. **`cortex-runtime`**: The central execution engine coordinating workers, memory FS, and external calls.
3. **`cortex-agents`**: The Agent logic, implementing specific AI behaviors and integrations.
4. **`cortex-ic-adapter`**: The adapter layer abstracting ICP-specific canister calls.
5. **`cortex-workbench-contracts`**: Definitions and traits for integrating tools into the Cortex execution environment.

---

## Execution Mechanisms

### Durable Execution (Hybrid Workflow Authority)
Per **Initiative 134**, the canonical workflow substrate is structured as an **Artifact Pipeline plus Execution Adapter Layer**.
- **`nostra-workflow-core`**: Cortex executes State Machines natively using `nostra-workflow-core` patterns rather than relying on the official Temporal SDK. This ensures WASM compatibility and offline parity (DEC-005). Serverless Workflow DSL acts as a deterministic projection, not the source-of-truth loop.
- **Execution Adapters**: Cortex abstracts worker execution behind Live Cognition Adapters and Batch Audit Adapters, operating dynamically behind the gateway without bypassing the Nostra-defined governance boundaries.

### Meta Workbench (Cross-Space Operations)
Per **Initiative 144**, Cortex incorporates the Meta Workbench architectural layer to enable global context.
- **Global Context**: Primitives (such as the Heap or Action Zones) are not strictly bound 1:1 to a `space_id`. A Meta Workbench mode (`space_id == "meta"`) enables unified cross-space projection, aggregating blocks and intent plans across all spaces a user actively participates in.

### Frontend UI & Rendering
Frontend behavior is exclusively a Cortex concern.
- **Protocol**: A2UI (Abstract Agent UI) defines how Agents render interfaces.
- **Resources**: Frontend hosts (Web) use React/Vite primitives and the shared React A2UI interpreter to handle form validation, error handling, and immediate invalidation without page reloads.
- **Action Dispatch**: User intents translate to execution actions within Cortex before mutating Nostra state.

### Conversation Surfaces
- The canonical live chat transport is `/ws/chat`.
- Before runtime dispatch, the gateway resolves canonical heap context bundles and persisted thread history.
- Provider-runtime Responses is the primary chat generation path; `workflow-engine.process_message` is compatibility fallback only.
- Conversation persistence is heap-backed via polymorphic conversation blocks, with server-backed projection routes:
  - `/api/cortex/chat/conversations`
  - `/api/cortex/chat/conversations/:threadId`
- Host-local caches may retain recent thread state for UX, but they are not the source of truth for conversation history.

---

## Infrastructure Bridges

### Vector & Semantic Intelligence
Cortex provides the embedding infrastructure for Nostra's knowledge graph.
- Embeddings are generated automatically for new contributions.
- Real-time semantic search (cosine similarity) is orchestrated by Cortex vector services using micro-batching to respect ICP instruction limits (DTS).

### Gaming & Simulation (Godot Bridge)
Cortex provides the interfaces for interactive states beyond documents.
- The Godot Bridge (JSON-RPC) allows game clients running in WASM to communicate with the Cortex host.
- Game states are translated to Knowledge Graph entities (Player as Entity, Inventory as Relationships).
