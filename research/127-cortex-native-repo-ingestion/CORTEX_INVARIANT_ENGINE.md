---
id: "127B"
name: "cortex-invariant-engine"
title: "Architecture: Cortex Invariant Governance Engine"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "memory-fs", "ingestion", "invariants", "siq", "governance", "graph-projection", "wasm-policy"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Architecture: Cortex Invariant Governance Engine

## 1. The Core Motivation: Local Repo Orchestration

Historically, workspace invariants in Nostra/Cortex were enforced via imperative, hardcoded Python/Bash scripts traversing the filesystem (`check_research_portfolio_consistency.py`). While sufficient for early bootstrapping, this model fails the constitutional split: `Nostra` defines *what exists*, while `Cortex` defines *how it runs*.

If Cortex is evolving into a generalized **Agent Harness (126)** capable of ingesting arbitrary repositories into the **Memory FS (121)**, the rule-checking paradigm must mature. **The repo is not just a folder tree; it is a materialized graph projection.**

The Cortex Invariant Engine is not a "better CI linter." It is a **general-purpose Policy Execution Substrate for Local Knowledge Systems** (applicable to DAOs, Research spaces, Games, and Codebases alike).

## 2. Theoretical Foundations & Inspirations

To build a world-class execution substrate, the Invariant Engine absorbs patterns from the strongest open-source/academic ecosystems:

1. **Open Policy Agent (OPA)**: Policy-as-code via WASM. Invariants are declarative rules decoupled from the runtime.
2. **Earthly & Bazel**: The repo is a deterministic build DAG inside a hermetic sandbox (Cortex Memory FS).
3. **Nx / Turborepo**: Incremental evaluation. The engine computes exactly the *change graph* and only runs impacted validators to scale Memory FS efficiently.
4. **Code Property Graph (CPG)** / *Fraunhofer AISEC*: Merging AST, control flow, and data flow into one queryable graph.
5. **Dagger**: CI pipelines defined as composable DAGs in code (*Execution is First-Class*).

## 3. The Multi-Layer Graph Architecture

If Cortex only evaluates the file structure, it becomes "CI 2.0." Instead, Cortex materializes multi-layer structural invariants over a **Repo Projection Spec**.

### 3.1 The Repo Projection Spec
When a repository is ingested into the Memory FS Sandbox, Cortex computes a unified graph projection:

```yaml
RepoProjection:
  fileGraph: [Directories, Artifacts, Markdown Links]
  symbolGraph: [AST, Traits, Structs, Functions]
  dependencyGraph: [Upstream Packages, Peer Modules]
  testGraph: [Fixtures, Mocks, Coverage Edges]
  workflowGraph: [Temporal Signals, Bash Tasks]
  metadata: [Stewardship, Status, Authors]
```

### 3.2 The WASM Policy Engine (OPA-Style)
A `Governance Profile` is dynamically bound to the graph projection. It dictates the invariant policies that the projection must pass.

*Example Invariants:*
* `must_have_stewardship_metadata` (File Graph + Metadata Layer)
* `no_unpinned_dependencies` (Dependency Layer)
* `research_folder_required` (File Graph Layer)
* `no_orphaned_initiatives` (Symbol + File Graph Layer)

These invariants are executed by a **WASM Policy Engine** running natively within the Cortex Runtime, entirely decoupled from Python/Bash dependencies, fulfilling the WASM-First Portability principle.

## 4. The End-to-End Pipeline

1. **Ingest & Sandbox (L0)**: A `DPubManifest` targets a repo. Cortex fetches it into an ephemeral `cortex-memory-fs/sandbox/{id}`. The Memory FS enforces Bazel-style deterministic, hermetic conditions.
2. **Materialize Projection (L1)**: Cortex parses the sandbox state to generate the unified `RepoProjection` (the CPG).
3. **Bind & Execute Policies (L1)**: An OPA-style WASM evaluator binds the `Governance Profile` to the Repo Projection. Nx-style incremental evaluation ensures only changed nodes trigger re-evaluation.
4. **Emit Events (L2)**: Invariant evaluation does not simply "fail the build." It emits structured, replayable events to the Global Event Log:
   ```yaml
   GlobalEvent:
     type: InvariantViolation
     resource: nostra://library/cortex-dev
     severity: P0
     details: "Initiative 034 is missing physical directory."
   ```
5. **Surface System Integrity Quality (SIQ)**: The A2UI frontend listens to these `InvariantViolation` events to render a living SIQ scorecard. In the Capability Navigation Graph (`CapabilityMatrixMap`), nodes with violations are visually highlighted (e.g., pulsing red borders) to map architectural debt directly onto the system's dependency tree. The sandbox is marked as "structurally corrupt" and quarantined from the canonical DPub graph until the violations are resolved via Agentic Proposals or manual Steward intervention.

## 5. Summary

This architecture explicitly models: **Invariant = Governance Rule applied to Graph Projection**. 

By treating the local repository as a queryable Code Property Graph governed by deterministic policies and visualizing the resulting SIQ on a living node map, Cortex scales beyond source code management into a constitutionally enforced engine for all Nostra knowledge spaces.
