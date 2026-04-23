---
id: '015'
name: nostra-open-source-library
title: 'Decisions Log: Open Source Library'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-15'
updated: '2026-02-15'
---

# Decisions Log: Open Source Library

**Context**: Architectural and strategic decisions guiding the development of the Nostra Open Source Library.

## DEC-001: Read-Only Analysis Scope
*   **Context**: Use case confusion with GitHub/GitLab.
*   **Decision**: The system will **NOT** support PRs, Issues, or mutable code management. It is strictly a "Playground" for analysis and matching.
*   **Status**: DECIDED
*   **Consequences**: UI must avoid "File Browser" metaphors; focus on Graph/Laboratory visuals.

## DEC-002: Ingestion Strategy
*   **Context**: How to standardized data from diverse repo structures.
*   **Decision**: Treat repositories as "Data Sources". Use `tree-sitter` for AST parsing to genericize language structures.
*   **Status**: DECIDED
*   **Alternatives**: Language Server Protocol (LSP) - rejected due to complexity of setup per-repo.

## DEC-003: Data Structure (KIP Compliance & 008 Contribution Types)
*   **Context**: How to store the relationships between ideas and code.
*   **Decision**: Use a **Semantic Knowledge Graph** compliant with **LDC Labs KIP** and **Nostra Contribution Types (008)**.
    *   **Referent Nodes**: `Library` and `Function` map to `EntityType: Library`.
    *   **Contribution Nodes**: `Idea` maps to `ContributionType: Idea` (Exploratory Phase). Identified gaps map to `ContributionType: Proposal` or `Issue`.
    *   **Edges**: `Implements`, `Solves`, `Requires`.
    *   **Constraint**: All nodes must implement the `KIP::Entity` interface to ensure compatibility with Nostra Agents (See `014`).
*   **Status**: DECIDED
*   **Status**: DECIDED

## DEC-004: Execution Environment (Execution Delegated)
*   **Context**: "Code Verification" and ingestion workflow requires sandboxing.
*   **Decision**: Deferred to **Initiative 126 (Agent Harness)** and **Initiative 121 (Cortex Memory FS)**.
    *   Ingestion is explicitly bounded by `127-cortex-native-repo-ingestion` into Memory FS sandboxes.
    *   Execution and LLM summarization are run natively via `AgentTask` using Authority Guard (`L1/L2`).
*   **Status**: DECIDED
