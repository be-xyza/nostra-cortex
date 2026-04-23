---
stewardship:
  layer: Systems
  primary_steward: Systems Steward
  domain: Agents & Execution
---

# Initiative 121: Cortex Memory FS

## Context & Rationale
Currently, agent memory exists as either unstructured vector embeddings (semantic context) or ephemeral session traces. As we scale agents to run long-horizon, durable workflows (via Temporal) and multi-agent swarms, the ability to trace, backtrack, and synthesize memory becomes a critical bottleneck.

Drawing from state-of-the-art constraints discussed in the "Git Context Controller" paper and industry patterns like Letta, we are standardizing agent episodic/working memory as a Git-backed local filesystem (Context FS).

## Goals
1. Establish a native Git-backed memory filesystem for all Cortex agents to use for reasoning, branching, and logging.
2. Formally decouple Episodic/Working Memory (Cortex Memory FS) from Semantic/Publication Memory (Nostra DPub).
3. Align memory management with Nostra constitutional requirements (preserve lineage, surface uncertainty, resist retrospective certainty).

## Phases
### Phase 0: Design & Proof of Concept
- Define the canonical file structure (`main.md`, `log.md`, `commit.md`).
- Define core tool APIs (`cortex.branch`, `cortex.commit`, `cortex.merge`, `cortex.context`).

### Phase 1: Ecosystem Alignment & Enrichment
- **118 (Cortex Runtime Extraction):** Memory FS must be implemented purely behind the generic `StorageAdapter` trait to maintain substrate neutrality (WASM and IC compatibility).
- **126 (Agent Harness Architecture):** Formally define FS interactions as JSON `CortexTools` passing through the `AuthorityGuard` before mutate. Prevent unprotected POSIX access.
- **127 (Cortex Native Repo Ingestion):** Support the `/sandbox` directory as the ingestion target for semantic diffing operations.
- **013 (Workflow Engine) & 041 (Vector Store):** Formally define the handoff. Cortex Memory FS acts as the "scratchpad" leading up to a formal `ExecutionEvent` (013) and Entity creation (which triggers Vector insertion in 041).
- **103 (Agent Client Protocol):** Bind core FS commands (`branch`, `merge`) as explicit ACP tools, and map `log.md` trace appends to ACP `session/update` events.
- **120 (Nostra Design Language - NDL):** Introduce A2UI components (e.g., `AgentContextViewer`, `TrajectoryBranch`) to visually differentiate ephemeral working memory from high-confidence, constitutional Graph data (DPub).

### Phase 2: Agent Runtime Integration
- Bind memory FS primitives to Cortex workers and the agent protocol.
- Test against multi-step workflows (e.g. testing/compilation loops).

### Phase 3: Swarm & Sleep-time Processes
- Implement "sleep-time" background jobs to defragment and summarize Memory FS.
- Enable parallel sub-agent branching for confident exploration of Godot simulation states or alternative algorithms.

## Milestone Advancement Gate
- Milestone/phase advancement for Initiative 121 is blocked when either `siq_governance_execution_contract` or `siq_host_parity_contract` is failing.
- Blocking evidence source is SIQ filesystem-canonical artifacts under `logs/siq/*`, not manual interpretation.
- `research/121-cortex-memory-fs/INTEGRITY_DEPENDENCIES.md` remains the governing dependency contract.
