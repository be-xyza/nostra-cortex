---
id: '121'
name: cortex-memory-fs
title: Requirements for Cortex Memory FS
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-20'
updated: '2026-02-20'
---

# Requirements for Cortex Memory FS

## 1. File Structure
The memory FS MUST maintain strict physical boundaries between an agent's reasoning trajectory and external code execution contexts.

### 1.1 The Reasoning Directory (`.GCC/`)
Contains logs, roadmaps, and thoughts.
- `.GCC/main.md`: Global roadmap and overarching objectives.
- `.GCC/branches/<name>/commit.md`: Milestone summaries.
- `.GCC/branches/<name>/log.md`: Raw execution traces and Observation-Thought-Action (OTA) cycles.
- `.GCC/metadata.yaml`: Meta-level context like file structures or environment configuration.

### 1.2 The Execution Sandbox (`/sandbox/`)
Contains ingested repositories and execution targets (managed by `127-cortex-native-repo-ingestion`).
- Agents MUST perform file analysis and CLI tool execution exclusively within this bounded directory.

## 2. Agent Commands (Tool Execution)
To comply with the `126 Agent Harness Architecture`, Memory FS commands are **NOT** raw POSIX filesystem writes available to the LLM.

Instead, they are explicitly defined as `CortexTools`. The LLM emits a JSON `ActionTarget` which is evaluated by the `AuthorityGuard` before the Temporal Worker mutates the local Git-backed FS.

Agents MUST have access to the following deterministic `CortexTools`:
- `cortex.commit(summary)`: Checkpoint current progress in the active branch, regenerating summaries.
- `cortex.branch(intent)`: Create a sandbox for experimentation without polluting the main reasoning path.
- `cortex.merge(branch)`: Synthesize a successful branch back into the main timeline.
- `cortex.context(...)`: Retrieve memory state at varying resolutions (from macroscopic roadmap down to specific OTA blocks).

## 3. Integration with Nostra DPubs and Graph
- **Ingestion**: Agents MUST ingest `DPubManifests` for canonical semantic grounding before attempting complex tasks.
- **Publication**: Upon completion of a workstream (successful `merge`), agents SHOULD format their findings as a new DPub `Edition` or `Chapter` update for formal publication to the Nostra Graph.
- The boundary between "messy thinking" (Memory FS) and "truthful assertion" (DPub) MUST remain rigorously enforced.
