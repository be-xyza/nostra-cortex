---
id: '127'
name: cortex-native-repo-ingestion
title: Execution Plan for Cortex-Native Repo Ingestion
type: general
project: nostra
status: draft
authors:
- User
- Antigravity
tags: ["cortex", "memory-fs", "ingestion", "temporal-boundary"]
created: '2026-02-25'
updated: '2026-02-25'
---

# Execution Plan for Cortex-Native Repo Ingestion

## Phase 1: Foundation (Currently Active)
1.  **Draft Manifest Schema**: Define `ingestion_registry.toml` establishing the governed ruleset for pulling repositories. (Completed)
2.  **Define Architecture Constraints**: Draft `RESEARCH.md`, `REQUIREMENTS.md`, and `DECISIONS.md` linking ingestion strongly to the MVK and Temporal Boundary. (Completed)
3.  **Register Initiative**: Update `RESEARCH_INITIATIVES_STATUS.md` with `127-cortex-native-repo-ingestion`. (Pending)

## Phase 2: Orchestration (L0/L1)
1.  **Develop System Watcher Daemon**: Implement a Temporal sync loop ensuring `ingestion_registry.toml` targets are fetched cleanly into isolated `cortex-memory-fs/sandboxes/`.
2.  **Author Semantic Diff Evaluator**: Create the `AgentTask` instructing the agent how to evaluate upstream drift in the isolated sandbox. Define the structure of the `AgentExecutionRecord`.

## Phase 3: Governance Escalation (L2)
1.  **Draft Proposal Schema**: Design the L2 `Proposal` the agent submits when it determines `RESEARCH.md` artifacts require updating.
2.  **Test Integration**: Run the full lifecycle: Daemon detects upstream delta -> Agent Sandbox semantic diff -> Agent constructs `Proposal` -> Human merges to `.md`.

## Phase 4: Remediation
1.  **Deprecate `reference/topics/`**: Progressively remove manually cloned repositories and replace them with governed manifest entries.
