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

## DEC-003: Data Structure (KIP Compliance)
*   **Context**: How to store the relationships between ideas and code.
*   **Decision**: Use a **Semantic Knowledge Graph** compliant with **LDC Labs KIP**.
    *   Nodes: `Idea`, `Library`, `Function`, `Concept`.
    *   Edges: `Implements`, `Solves`, `Requires`.
    *   **Constraint**: All nodes must implement the `KIP::Entity` interface to ensure compatibility with Nostra Agents (See `014`).
*   **Status**: DECIDED

## DEC-004: Execution Environment (Pending)
*   **Context**: "Code Verification" workflow requires sandboxing.
*   **Decision**: TBD. Evaluating Off-chain (Firecracker MicroVMs) vs On-chain (Ephemeral Canisters).
*   **Status**: PROPOSED
