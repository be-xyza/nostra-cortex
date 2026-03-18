---
id: "127"
name: "cortex-native-repo-ingestion"
title: "Decisions: Cortex-Native Repo Ingestion"
type: "research"
project: "nostra"
status: draft
authors:
  - "User"
  - "Antigravity"
tags: ["decisions", "cortex", "memory-fs", "ingestion"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Decisions: Cortex-Native Repo Ingestion

## 1. Governance Boundary Enforcement
**Status**: Decided
**Context**: The team manually fetched repositories into `reference/topics/...`, creating a chaotic, undocumented graph of unversioned external states that constantly drifted from canonical reality.
**Decision**: Ingestion will be explicitly governed by `ingestion_registry.toml`. No ad-hoc clones are permitted. The manifest will dictate URI, polling frequency, and the intended target `RESEARCH.md` artifact.
**Consequences**: Eliminates uncontrolled repo fetching. Every imported topic must go through a formal Configuration-as-Code gate.

## 2. Temporal Sandbox Isolation
**Status**: Decided
**Context**: External codebases are fundamentally incompatible with the persistent `ICP` DPub reference ecosystem because they are inherently volatile.
**Decision**: Ingestion targets will clone strictly into `.GCC/sandboxes/` or equivalent ephemeral directories managed by **Cortex Memory FS (`121`)**. Agent tools are constrained to analyze only within this boundary.
**Consequences**: The primary workspace graph remains clean, uncluttered, and semantically pure. We effectively map an architectural "quarantine" over external repos.

## 3. Semantic Diff vs Text Delta Execution
**Status**: Decided
**Context**: Basic `git diff` outputs often clutter reasoning agents with irrelevant string matches (e.g., dependency bump versions or README spelling fixes).
**Decision**: The ingestion system will invoke an `AgentTask` to perform **Semantic Diffing** via the Cortex Agent Runtime Harness. The task outputs an `ExecutionRecord` answering high-level questions: "Did the paradigm change?", "Were core assumptions broken?".
**Consequences**: Agents process higher-order cognitive changes rather than simple syntactic noise. This directly impacts computational efficiency and logic accuracy.

## 4. Formal Escalation Pipeline (Authority Guard)
**Status**: Decided
**Context**: Autonomous agents that automatically sync and rewrite main canonical artifacts like `RESEARCH.md` present a significant "Auto-Commit" risk, potentially erasing carefully constructed prior analysis.
**Decision**: Agents must respect the *L1 -> L2 Escalation Path*. Agents will prepare draft updates (`Proposals`) containing the Semantic Diff and recommended changes. A Human or Maintainer committee must formally merge (Commit) the updates to `RESEARCH.md`.
**Consequences**: Preserves Nostra Authority. Evaluates Truth within the temporal boundary, but requires human governance to convert it to Canonical Assertions.
