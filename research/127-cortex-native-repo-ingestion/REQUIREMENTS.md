---
id: '127'
name: cortex-native-repo-ingestion
title: Requirements for Cortex-Native Repo Ingestion
type: general
project: nostra
status: draft
authors:
- User
- Antigravity
tags: ["cortex", "memory-fs", "ingestion", "temporal-boundary", "mvk"]
created: '2026-02-25'
updated: '2026-02-25'
---

# Requirements for Cortex-Native Repo Ingestion

## 1. Governance Boundary (The Manifest)
The ingestion mechanism MUST be governed by a Configuration-as-Code manifest (e.g., `ingestion_registry.toml`).
*   **No Ad-Hoc Fetching**: The agent MUST NOT autonomously execute structural `git clone` or `git pull` commands outside of the authorized URIs defined in the manifest.
*   **Registry Format**: The manifest MUST detail the designated repository URI, target branch/commit hash, polling frequency, and the canonical local `RESEARCH.md` artifact it maps to.

## 2. Temporal Sandbox Isolation
External codebases MUST be treated as volatile runtime state.
*   **Location**: All ingested repositories MUST be cloned/fetched strictly into the ephemeral Cortex Memory FS boundary (e.g., `.GCC/sandboxes/` or `cortex-memory-fs/sandboxes/`).
*   **Pollution Prevention**: Ingested code MUST NOT mingle with the persistent `reference` or `topics` directories in the workspace graph. The agent explores the repository *within the sandbox*.

## 3. Minimal Viable Kernel (MVK) Execution
The ingestion logic MUST avoid monolithic crawlers in favor of highly-scoped evaluators.
*   **Execution**: Delta detection and semantic diffing MUST be executed deterministically by the **Cortex Agent Runtime Harness (`126`)**.
*   **Evaluation Mode**: The agent task MUST perform **Semantic Diffing** (analyzing architectural changes, shifting principles, or invalidated assumptions) rather than simplistic string-matching delta.

## 4. Authority Guard Enforcement
The agent's interaction with the ingested repository and its output to the canonical graph MUST adhere to strict L0/L1/L2 escalation paths.
*   **L0 (Read/Observe)**: The agent operates within the `cortex-memory-fs/sandbox/` to read and parse the fresh delta.
*   **L1 (Plan/Draft)**: The agent generates an `ExecutionRecord` containing the semantic diff and drafts a proposal to update the corresponding local `RESEARCH.md` artifact.
*   **L2 (Propose/Commit)**: The agent MUST NOT autonomously push, commit, or directly modify the canonical Nostra `RESEARCH.md` document. It MUST officially submit a `Proposal` for human/maintainer review. The "Commit" only occurs upon approval.

## 5. Artifact Lineage
If a `Proposal` is accepted, the resulting modification to the canonical `RESEARCH.md` artifact MUST retain cryptographic lineage back to the executed `AgentTask`, the temporal sandbox sync, and the specific version hash of the upstream repository it analyzed.
