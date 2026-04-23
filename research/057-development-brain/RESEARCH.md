---
id: '057'
name: development-brain
title: 057 - Nostra / Cortex Desktop (formerly DevBrain)
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# 057 - Nostra / Cortex Desktop (formerly DevBrain)

> **Status**: ACTIVE
 / PARTIALLY IMPLEMENTED
> **Owner**: User (Nostra Architect)
> **Linked Initiatives**:
> - [046 - System Standards](../046-nostra-system-standards/RESEARCH.md)
> - [047 - Temporal Architecture](../047-temporal-architecture/RESEARCH.md)
> - [052 - Agent Zero Analysis](../052-agent-zero-analysis/RESEARCH.md)
> - [053 - Elasticsearch Analysis](../053-elasticsearch-analysis/RESEARCH.md)
> - [056 - Logic Layer Architecture](../056-logic-layer-architecture/RESEARCH.md)
> - [028 - A2UI Integration](../028-a2ui-integration-feasibility/RESEARCH.md)
> - [059 - Benchmarking System](../059-benchmarking-system/RESEARCH.md)
> - [060 - MemEvolve Integration](../060-memevolve-integration/RESEARCH.md)

## 1. Core Vision

**The Development Brain** is the meta-control plane for Nostra / Cortex. It is a single, cohesive application designed to understand state, time, execution, failure, and intent across both local interactions and the Internet Computer (ICP).

It acts as the **Universal Interface** for the entire Nostra stack:
*   **Universal Agent Runner** (Visually executing `052` Agent Zero patterns).
*   **Logic Layer Viewport** (Rendering `056` Graphs and Tables).
*   **Constitutional Guardian** (Visually enforcing `034` Sovereignty and Roles).
*   **Temporal Observer** (Deep debugging `047` Workflows).

This is a fusion of **Temporal + Kubernetes + Git + IDE + Blockchain Explorer + AI Ops**, opinionated specifically for Nostra's execution model.

## 2. Constitutional Alignment

The Development Brain is built upon the **Nostra Constitutional Framework** (`034`):

| Constitution | DevBrain Implementation |
| :--- | :--- |
| **Spaces are Sovereign** | The UI strictly respects Space boundaries. Deployment authority is scoped to the current Space. Cross-space actions require explicit bridging. |
| **Knowledge Integrity** | "Memory is Infrastructure." Cortex Desktop visualizes the *lineage* of deployments and errors, not just the current state. Negative knowledge (failed experiments) is preserved and explorable. |

| **Stewardship & Roles** | Role-Based UI. A "Steward" sees different controls than a "Participant". The UI reinforces responsibility. |
| **Execution-as-Knowledge** | Every significant action (Deploy, Rollback, Workflow Fix) is recorded as a "Contribution" in the Cortex graph. |
| **Time Legibility** | (UI/UX Manifesto) Time is a first-class citizen. Users can "scrub" the state of their system backward to understand causality. |

## 3. High-Level Architecture

### A. Control Plane vs Data Plane

| Plane | Component | Responsibilities |
|-------|-----------|------------------|
| **Control Plane** | **Cortex Desktop** | Runs locally (Desktop/Web). Observes, Deploys, Inspects, Simulates. Has authority to mute/pause/rollback. |

| **Data Plane** | **Cortex** | The running system: Canisters, Temporal Workflows, Vector Stores, Bridges. |

**Key Principle**: Cortex Desktop never replaces Cortex; it steers it.


### B. Technical Spine

| Layer | System / Tech | Integration Strategy |
|-------|--------------|-------|
| **Host** | **Dioxus** (Rust/WASM) | Matches Nostra Shell. Deploys to Desktop, Web, Mobile. |
| **UI Protocol** | **A2UI** (Agent-to-UI) | **Critical**. Canisters stream their own debug/admin UIs to the DevBrain via A2UI. |
| **IPC / Local** | **gRPC** / **JSON-RPC** | For local process communication (Godot bridge pattern). |
| **On-Chain** | **ICP Agent** + **Candid** | Introspection of running canisters. |
| **Workflows** | **Temporal API** | Visibility into long-running workflows (`047` integration). |
| **State Model** | **Graph-First** | Nodes = Systems, Edges = Dependencies (`056` integration). |
| **Visualization** | **D3.js** | Visualizing the dependency graph and error trees (`022` integration). |

## 4. Core Subsystems & Plan Resolution

### 1. Universal Agent Runner UI (Resolves `052`)
*   **Tree View**: Visualizes the Recursive Delegation tree defined in Agent Zero analysis. "Agent A waiting for Agent B".
*   **Prompt Graph**: View and edit the Composable Prompt DAG.
*   **Sandboxed Terminal**: A safe UI wrapper around the local Dockerized runner.

### 2. Logic Layer Viewport (Resolves `056`)
*   **Grid View**: High-performance virtualized grid (NocoDB style) for editing Data stored in Logic Canisters.
*   **Formula Debugger**: Step-through debugging of AST-based formulas.
*   **Schema Visualizer**: D3.js interactive graph of the underlying `SpgType` definitions.

### 3. Canister Intelligence Panel (Resolves `046`)
*   **Live Connection**: Status (Healthy/Degraded), Cycles, Memory.
*   **A2UI Integration**: Renders bespoke admin UIs streamed from canisters.
*   **Drift Detection**: Compares running Candid interface vs local source.

### 4. Local Environment Orchestrator (Resolves `012`, `043`)
*   **Process Control**: Starts/Stops `dfx`, Temporal workers, Vector services.
*   **Hot-Swapping**: Bind local dev canisters to remote mainnet canisters.
*   **Snapshotting**: "Save Game" for infrastructure state.

### 5. Temporal / Execution Observer (Resolves `047`)
*   **Timeline Scrubber**: "What was the state at T-3 hours?"
*   **Workflow Visualization**: Active, Sleeping, Failed trees.
*   **Replay**: Deterministic replay of failed workflows.

### 6. Error Intelligence & Causality Engine (Resolves `053`, `054`)
Errors are **Graph Nodes**, not log lines.
*   **Causality Chains**: Trace error roots across system boundaries.
*   **Data Source**: Powered by Elasticsearch / Opentelemetry.

## 5. Enhancements (Nostra Differentiators)

### A. Execution-as-Knowledge
Every incident and resolution is stored as a **Contribution** in the Cortex Knowledge Graph. The system "learns" from failures.

### B. Simulation Mode (The "What-If" Engine)
Before a deploy, run a simulation:
- "If I upgrade Canister A, will Canister B break due to interface change?"
- Run "Forked Timelines" using Temporal + State Snapshots.

### C. AI Copilot (Agent Zero Integration)
Leveraging **Agent Zero** (Research 052):
- **Explain**: "Why is the payment workflow stuck?"
- **Suggest**: "Increase cycle limit on canister X."
- **Action**: "Fix it" (requires specific approval).
- **Constraint**: AI cannot mutate state without explicit user confirmation (The "Human-in-the-Loop" rule).

### D. Governance-Aware DevOps
The DevBrain respects **Spaces** and **Governance**:
- "You cannot deploy to 'Production' without 2/3 multisig approval."
- Seamless UI for gathering those signatures.

### E. Unified Configuration Framework (Resolves "Production Fallbacks")
To ensure resilience across Local, Testnet, and Mainnet environments, DevBrain exposes the **057 Config Matrix**:

1.  **Service Resolution**:
    *   *Local*: Uses mock vectors, local `dfx` replica, in-memory graph.
    *   *Testnet*: Uses shared "Staging" canisters, remote vector service.
    *   *Production*: Uses dedicated isolated canisters, high-availability indexers.
2.  **Fallback Strategy**:
    *   **Vector Service**: If ELNA is unreachable -> Fallback to Keyword Search (Solr/Bleve) -> Fallback to Raw Scan.
    *   **LLM Provider**: If OpenAI is rate-limited -> Fallback to Anthropic -> Fallback to Local (Ollama/Ignition).
3.  **Auditability**:
    *   Every configuration change is an event in the `SystemLog`.
    *   "Who switched the LLM provider to 'Local'?" is a queryable fact.

## 6. Knowledge Engine Integration (Resolves "Full Book" & 050)

### The "Full Book" Node
DevBrain treats a **Book** not just as text, but as a **Compiled Software Artifact** that defines the Knowledge Graph.

*   **Structure**: See `SPECS/FULL_BOOK_SCHEMA.json` for the validated V2 schema.
*   **Memory Tiers**:
    *   **Hot**: Current Chapter loaded in Wasm Heap.
    *   **Warm**: Vector Indices of the whole book (for RAG).
    *   **Cold**: Full history and assets in Stable Memory / Asset Canister.
*   **Visualizer**:
    *   DevBrain includes a **Graph Explorer** specific to Books.
    *   "Show me all concepts defined in Chapter 3 and their downstream dependencies."

## 7. Evolution Engine (Resolves 060 - MemEvolve)

**DevBrain acts as the "Outer Loop" for Agent Evolution.**
*   **Concept**: Powered by [060 - MemEvolve Integration](../060-memevolve-integration/RESEARCH.md), the DevBrain doesn't just run agents, it *evolves* them.
*   **The Genotype Config**: A JSON/YAML schema to define memory architectures (Encode -> Store -> Retrieve).
*   **Experiment Runner**: A background process that forks an agent, tweaks its memory genotype, runs it through the `059` Benchmark Arena, and proposes the improved version to the user.

## 8. Guidelines & Recommendations

### Design Principles
1.  **Graph-First**: Tables are secondary. relationships are primary.
2.  **Time-Travel**: Always allow scrubbing backward.
3.  **Errors as Entities**: Never just text.
4.  **Local-First, Chain-Aware**: Fast local interaction, reliable chain syncing.

### Build Order (Practical Path)
1.  **Canister Discovery**: Connect to local `dfx` and list canisters.
2.  **Status & A2UI**: Render simple status + A2UI stream from a demo canister.
3.  **Local Orchestration**: Start/Stop `dfx` from the UI.
4.  **Temporal Visibility**: Connect to Temporal server and graph workflows.
5.  **Error Graph**: Ingest logs and build the error node system.
6.  **Simulation**: The advanced "what-if" engine.
