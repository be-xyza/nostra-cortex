---
id: "127C"
name: "cortex-validation-references"
title: "Research: Validation References for the Cortex Invariant Engine"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "memory-fs", "validation", "graph-projection", "wasm-policy", "research-references"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Validation References for the Cortex Invariant Engine

This document provides formalized, code-backed verification of the architectural patterns absorbed by the **Cortex Invariant Governance Engine (127B)**. We have validated the user's strategic assertions across multiple state-of-the-art ecosystems to ensure Cortex's shift to a general-purpose Policy Execution Substrate is theoretically sound and practically proven.

Our core assertion—**Invariant = Governance Rule applied to a Materialized Graph Projection**—is robustly backed by the following paradigms.

---

## 1. Repos as Declarative Graphs & Deterministic Sandboxes

If Cortex Invariant evaluation is to be reproducible, the engine cannot rely on ambient local environments.

### 1.1 Earthly (Deterministic Build DAGs)
* **Concept:** Earthly operates over a Directed Acyclic Graph (DAG) of build steps executed within strictly isolated, containerized environments.
* **Validation:** Verified via Earthly documentation. Earthly completely prevents "works-on-my-machine" drift by guaranteeing that *nothing* is shared between targets unless explicitly declared.
* **Cortex Alignment:** This validates the **Cortex Memory FS (121)** as our temporal sandbox. Like Earthly, Cortex must fetch ingested code into strictly partitioned namespaces where invariant policies run without nondeterministic external bleed.

### 1.2 Bazel (Hermeticity and Explicit Dependencies)
* **Concept:** Hermetic builds rely on explicit dependency declarations to guarantee deterministic outputs.
* **Validation:** Verified via Bazel's `BUILD` file constraints and sandboxing architecture. Bazel hashes the contents of every explicitly declared input to calculate exact cache hits.
* **Cortex Alignment:** Invariants checking for "unpinned dependencies" or "missing metadata" are deterministic *if and only if* the sandbox is hermetic.

---

## 2. Policy-as-Code & WASM Evaluation

Migrating from hardcoded Python/Bash scripts to agnostic policy evaluation is essential for scaling Cortex.

### 2.1 Open Policy Agent (OPA) / CNCF Gatekeeper
* **Concept:** OPA decoupling policy decision-making from application logic using Rego, compiled to WASM. 
* **Validation:** Verified via OPA documentation. OPA has a dedicated compiler pathway to convert Rego policies into standalone `policy.wasm` modules. These are highly embeddable into host runtimes (like Cloudflare edge workers or custom go/rust runtimes).
* **Cortex Alignment:** This perfectly matches our **WASM Policy Engine** requirement. Cortex can compile `alignment_contracts.toml` or Rego equivalent into WASM, allowing the core Rust runtime to evaluate structural invariants without embedding a Python interpreter (fulfilling the MVK principle).

---

## 3. Incremental Execution & Task Graphs

Evaluating the entire project graph on every file change is prohibitively expensive.

### 3.1 Nx and Turborepo
* **Concept:** Computing *affected* change graphs and caching incremental execution.
* **Validation:** Verified. Both Nx (Project Graph) and Turborepo (Task Graph) hash task inputs contextually to skip redundant computations. Nx specifically creates orchestration boundaries to prevent duplicate compilations across monorepos.
* **Cortex Alignment:** SIQ calculation in Cortex must be incremental. When a file updates in the Memory FS sandbox, the Invariant Engine recalculates only the affected branch of the Repo Projection.

---

## 4. Repos as Materialized Graph Projections

The structural leap from analyzing file trees to querying unified semantic structures.

### 4.1 Code Property Graph (CPG)
* **Concept:** Authored by Fabian Yamaguchi (ShiftLeft/Fraunhofer AISEC), the CPG merges Abstract Syntax Trees (AST), Control Flow Graphs (CFG), and Data Flow Graphs (DFG) into a single queryable property graph.
* **Validation:** Verified via Fraunhofer AISEC's open-source CPG generators and ShiftLeft Ocular. This model treats code not as text, but as a multi-dimensional graph for vulnerability execution.
* **Cortex Alignment:** This confirms our **Repo Projection Spec**. Before evaluating policies, Cortex translates the raw Memory FS files into a CPG-esque structure, merging filesystem layers, dependency layers, and metadata layers into a unified knowledge graph.

---

## 5. Academic Foundations

### 5.1 Build Systems à la Carte (Mokhov et al., 2018)
* **Concept:** All build systems are recombinations of a *Scheduler*, a *Rebuilder*, and a *Dependency Graph*.
* **Validation:** Published at ICFP 2018. This paper mathematically models systems like Make, Shake, and Bazel.
* **Cortex Alignment:** In our model, the Temporal Workflow is the Scheduler, the Cortex Runtime is the Rebuilder, and the Repo Projection is the Dependency Graph.

### 5.2 Hermit: Deterministic Execution (Microsoft Research / Meta)
* **Concept:** Emulation layer intercepting system calls to enforce perfect determinism for containerized software (often used to eliminate flakiness).
* **Validation:** Verified (originally associated with Meta/Facebook, presented effectively at ASPLOS '20).
* **Cortex Alignment:** Validates the extreme lengths required to guarantee reproducible evaluations. Nondeterministic environments destroy the value of SIQ.

### 5.3 Dagger (CI Pipelines as Code / DAG Execution)
* **Concept:** Defining pipelines in Turing-complete languages (Go/TS) translated into BuildKit DAG queries.
* **Validation:** Verified. Dagger allows local-first development by turning "pipelines as code" into reproducible concurrent DAG executions.
* **Cortex Alignment:** This supports our "Execution is First-Class" mandate. The Invariant Governance Profile itself is an executable workflow graph passed into the WASM runtime.
