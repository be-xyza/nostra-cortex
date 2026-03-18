---
id: "127"
name: "cortex-native-repo-ingestion"
title: "Research: Cortex-Native Repository Ingestion"
type: "research"
project: "nostra"
status: draft
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "memory-fs", "ingestion", "temporal-boundary", "mvk", "world-model"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Research: Cortex-Native Repository Ingestion

## 1. The Core Problem: Systemic Drift

Historically, external repositories were ingested by manually executing `git clone` or `git pull` directly into the `reference/topics/` directories of the workspace. This established a dangerous anti-pattern:
*   **Temporal Violation**: External codebases are fundamentally volatile, yet they were persisted as static files within the primary Notion/Obsidian-esque workspace graph.
*   **Decoupled Assertions**: Canonical `RESEARCH.md` or `ANALYSIS.md` artifacts analyzing these external codebases rapidly lost context. As the external upstream evolved, our local assertions drifted, rendering evaluations stale and functionally misleading.
*   **Pollution**: The global graph became polluted with ungoverned, unversioned state.

As a high-functioning engineering organization, **Systemic Drift** is unacceptable. If an LLM or an engineering team relies on a stale `RESEARCH.md` whose underlying reference codebase has shifted three minor versions upstream, architectural damage is inevitable.

## 2. Cortex-Native Ingestion Architecture

To remediate this, repository ingestion and analysis must become governed, deterministic, native capabilities of Cortex.

The architecture aggressively enforces three constitutional principles:

### A. The Temporal Boundary (Cortex Memory FS)
External repositories must never mingle with the canonical DPub graph. They are **volatile runtime state**. 
*   **Rule**: Repositories are fetched exclusively into strictly isolated temporal sandboxes within `cortex-memory-fs/sandboxes/`.
*   **Why**: This maintains a pure execution environment where the agent can explore, analyze, and diff the repository without accidentally indexing or committing external noise into the `ICP` project environment.

### B. Minimal Viable Kernel (MVK)
Ingestion is not a monolithic generic web-crawler. It is composed of highly scoped, deterministic evaluators executed on-demand by the **Cortex Agent Runtime Harness (`126`)**.
*   **Rule**: Ingestion targets are explicitly defined by Configuration-as-Code via a `DPubManifest` (e.g., `ingestion_registry.toml`).
*   **Why**: Provides a governed API surface defining exactly *what* is allowed into the Memory FS, *when* to poll it, and *which branches* are relevant.

### C. Authority Guard Enforcement
Agents operating within the temporal sandbox exist purely at **L0 (Read/Observe)** and **L1 (Plan/Draft)** authority levels.
*   **Rule**: Agents generate **Semantic Diffs**—evaluating architectural shifts rather than just typographical delta—and output an `ExecutionRecord`.
*   **Rule**: To update a canonical `RESEARCH.md` artifact based on new external state, the agent must escalate a formal **Proposal (L2)** for human/maintainer review.
*   **Why**: Prevents autonomous, stochastic "auto-commits" to our World Model. The Temporal state evaluates truth, but Governance Canonicalizes it.

## 3. The 4-Stage Ingestion Pipeline

1.  **Manifest Evaluation**: A local daemon (or Temporal workflow) reads `ingestion_registry.toml`. If the upstream hash differs from the registered hash, it triggers a sync task.
2.  **Sandbox Isolation**: The upstream delta is fetched into the ephemeral `cortex-memory-fs/sandbox/{repo-id}`.
3.  **Semantic Diffing**: An `AgentTask` evaluates the delta. It asks: "Did the architecture change? Were core dependencies updated? Are the original assumptions in our `RESEARCH.md` invalidated?"
4.  **Governance Escalation**: The agent drafts a structured `Proposal` encapsulating the semantic diff and the recommended updates to the canonical research artifacts for maintainer review.

## 4. Next Steps Implementation
1. Develop the `AgentTask` protocol for executing semantic diffs natively.
2. Draft the JSON/TOML schemas for `ingestion_registry.toml`.
3. Standardize the `AgentExecutionRecord` output for the semantic evaluation.
