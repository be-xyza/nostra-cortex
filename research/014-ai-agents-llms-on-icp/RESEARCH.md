---
id: '014'
name: ai-agents-llms-on-icp
title: 'Research: AI Agents and LLMs on ICP'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: AI Agents and LLMs on ICP

## User Request
Create a research project looking into utilizing AI agents or LLMs (both on-chain and externally) on ICP.

## Context & Objectives
This research aims to synthesize existing project initiatives (`nostra`) with the broader ICP AI ecosystem.
- **Nostra**: Needs AI for knowledge management, summaries, similarity detection, and potential "cooperative" agents.
- **Motoko Maps KG**: Already implements a basic "Knowledge Graph Agent" and chat interface.
- **Goal**: Define a unified approach for AI agents on ICP, leveraging both on-chain (canisters) and off-chain (MCP, external APIs) capabilities.

## Existing Project Analysis

### 1. Nostra (Collaboration Platform)
- **Status**: V2 Architecture planned.
- **AI Readiness**: Designed with "AI Integration Readiness" in mind (read-only agents initially).
- **Use Cases**: Thread summarization, merge conflict resolution, "thought" verification, similarity search.
- **Architecture**: Canister-based, event-sourced.

### 2. Motoko Maps KG (Knowledge Graph)
- **Status**: Functional demo with AI chat.
- **AI Integration**: Uses a legacy external Python script (`knowledge_graph_agent.py`) via OpenAI API (DEPRECATED: Migrating to `AsyncExternalOp` Workers).
- **Architecture**: Stores entities/relationships in stable memory. Has "Chat Conversations Map" and "Encrypted API Keys Map".

### 3. Multi-Project Architecture (Research 001)
- **Insight**: Evaluated Graphiti vs OpenSPG.
- **Recommendation**: "Native ICP Enhancement" (Option A) or "Hybrid Architecture" (Option B).
- **Key Takeaway**: External agents (MCP or off-chain services) are likely necessary for heavy lifting (inference, complex extraction) but ICP can handle storage and logic.

## External Resource Analysis

### 1. LDC Labs (Infrastructure & TEEs)
- **Core Tech**: **Anda Framework** (Rust-based agent framework), **IC-TEE** (Trusted Execution Environments on ICP).
- **Key Concepts**:
    - **KIP (Knowledge-memory Interaction Protocol)**: Standard for symbolic knowledge representation, making agent memory verifiable and governable.
    - **Tee-based Identity**: Anchors agent identity to hardware-secured environments.
- **Relevance**: Critical for "Trustworthy" agents. If Nostra agents need to sign transactions or hold secrets, TEEs (via Anda) are the path.

### 2. DFINITY LLM (Inference Architecture)
- **Core Tech**: **AI Workers** (stateless external nodes) that poll canisters for prompts.
- **Architecture**:
    - Canister queues prompts.
    - External "AI Worker" polls, executes inference (GPU/API), and returns response.
- **Relevance**: Standard pattern for "Heavy" AI tasks that cannot run in a canister. This is the "Gateway" approach.

### 3. Elna AI (Consumer & Tokenomics)
- **Core Tech**: On-chain Vector Database, Canister-hosted AI companions.
- **Key Concepts**:
    - **Privacy**: Fully on-chain (data doesn't leave ICP if utilizing on-chain models/vector stores).
    - **Monetization**: Token-based access to specialized agents.
- **Relevance**: Demonstrates viability of *on-chain* Vector DBs, which is crucial for `Cortex`.

### 4. ArcMind AI (Autonomous Agents)
- **Core Tech**: **Long-Running Agents** (Chain of Thought reasoning), **Vector Database on ICP**.
- **Architecture**:
    - Agents breakdown goals into subtasks.
    - Uses ICP as "Long Term Memory" (Vector DB).
- **Relevance**: Closest match to "Autonomous Agents". The pattern of "Canister as Brain + Memory" fits Nostra's goals.

## Synthesis: A Unified AI Architecture for Nostra

To "utilize AI agents both on-chain and externally" while incorporating Nostra and Motoko Maps, we propose a 3-Tier Layered Architecture:

### Layer 1: The "Brain" (Orchestration & State)
- **Location**: ICP Canisters (Nostra/Motoko Maps).
- **Role**:
    - Stores the "World Model" (Knowledge Graph).
    - Manages Agent "Identity" (Principals).
    - Queues tasks (Prompts/Goals).
    - **Tech**: `Cortex` (Stable Memory), `nostra` (Activity Stream).
    - **Tech**: Enhance `Cortex` with vector indices (rust-based canister or external service).

### Layer 2: The "Memory" (Context & Retrieval)
- **Location**: ICP (On-chain Vector DB) + External (Optional).
- **Role**:
    - Semantic Search (ArcMind/ELNA style).
    - Symbolic Knowledge (LDC Labs KIP style).
    - **Tech**: Enhance `motoko-maps-kg` with vector indices (rust-based canister or external service).

### Layer 3: The "Compute" (Inference & Action)
- **Location**: Hybrid.
    - **Lightweight**: On-chain (small models, rule-based inference) via Rust/Motoko.
    - **Heavyweight**: External "AI Workers" using the `AsyncExternalOp` protocol (from [013](../013-nostra-workflow-engine/RESEARCH.md)).
    - **Tech**:
        - **DFINITY **`llm`** Pattern** for connecting to GPT-4/Claude.
        - **LDC Labs **`Anda` for autonomous agents requiring private keys/signing.

### Layer 3.5: The "Behavior" (Roles & Process)
- **Concept**: "Agent-as-Code" (derived from BMAD Framework, see `research/017`).
- **Role**: Defines the *Persona* and *Workflow* the compute layer executes.
- **Components**:
    - **AgentPersona**: Stored in Motoko Maps (Name, Role, Principles).
    -   **ContextNode**: Linked memory nodes (Project Context, preferences) acting as the "Sidecar".

### Layer 3.6: The "Capabilities" (Skills & Tools)
-   **Concept**: Modular "Cartridges" of tools and prompts.
-   **Source**: **Nostra Library Registry** (`018`).
-   **Mechanism**: **Agent Tools Library** (`024`) enabling dynamic mounting of specialized skillsets (e.g., "Legal Research", "TypeScript Dev").


### Layer 4: The Client Protocol (New)
To support CLI tools (Skills Sync Service):
- **Auth**: Standardize SIWE or pre-generated SessionToken authentication.
- **Discovery**: `.well-known/nostra.json` standard for Agents to discover Space capabilities.
- **Contribution**: JSON-Schema standards for Agents to post `Reflection`, `Report`, and `Artifact` types.

## Strategic Recommendation

> [!NOTE]
> See [PLAN.md](./PLAN.md) for execution roadmap, [DECISIONS.md](./DECISIONS.md) for architecture, and [REQUIREMENTS.md](./REQUIREMENTS.md) for specs.

1.  **Adopt "AI Worker" Pattern for Nostra V2** (See Phase 2 in Plan).
2.  **Evolve Motoko Maps into "Agent Memory"**: Adopt **LDC Labs KIP** (See [STUDY_KIP_VS_ALTERNATIVES_MD](./STUDY_KIP_VS_ALTERNATIVES.md)).
3.  **Pilot arcMind-style Autonomous Agents**: The "Nostra Gardener" (Outputs aligned with `008` Types: `Report`, `Identity`).
