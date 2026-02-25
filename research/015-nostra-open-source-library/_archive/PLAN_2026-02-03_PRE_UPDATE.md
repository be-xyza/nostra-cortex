# Plan: Nostra Open Source Library

**Status**: PROPOSED
**Version**: 1.0
**Context**: This plan outlines the development steps to create the Nostra Open Source Library, a "playground" for analyzing and mapping open source technologies to ideas.

## User Review Required
> [!IMPORTANT]
> **Scope Restriction**: This initiative explicitly **excludes** code management features such as Pull Request creation, Issue tracking, or bug reporting. The system is Read-Only/Analysis-Only by design.

## Economic Alignment
> [!NOTE]
> This Open Source library represents the **"Free Tier" / Public Good** reference implementation of the Nostra Library Economy.
> See **[003-nostra-library-economics](../003-nostra-library-economics/PLAN.md)** for the broader pricing model.

## Phased Implementation

### Phase 1: Ingestion Pipeline (Foundation)
- [ ] **Define Schema**: Create `nostra.library.yaml` specification for manual overrides.
- [ ] **Implement Ingestion Workflow**:
    -   [ ] URL Validation & De-duplication.
    -   [ ] Shallow Clone logic (Git).
    -   [ ] Metadata Extraction (README, manifest files).
- [ ] **Data Storage**: Determine and set up storage for Repository Nodes and Metadata.

### Phase 2: Analysis Engine (Deep Dive)
- [ ] **Integrate Tree-Sitter**: Implement universal parser for symbol extraction (Classes, Functions).
- [ ] **LLM Integration**: Connect "Summarize Module" prompt to the ingestion pipeline.
- [ ] **Knowledge Graph Visualization**:
    -   [ ] Implement initial Force-Directed Graph using D3.js or Sigma.js (Decision Pending).
    -   [ ] Visualize `Library` -> `Concept` edges.

### Phase 3: The Playground (UI/UX)
- [ ] **Design "Laboratory" Interface**: Create high-fidelity mocks for the graph explorer (distinct from file browsers).
- [ ] **Feasibility Check Workflow**:
    -   [ ] Implement "Idea Decomposition" via LLM.
    -   [ ] Implement "Search & Match" against stored Library nodes.

### Phase 4: Gap Detection & Advanced Features
- [ ] **Gap Analysis**: logic to identify query clusters with no matching libraries.
- [ ] **Code Verification Sandbox** (Optional/Later):
    -   [ ] Research Firecracker vs On-chain sandboxing (See `WORKFLOWS.md`).

## Reference Workflows
- [Ingestion Workflow](./WORKFLOWS.md#1-repository-ingestion-workflow)
- [Feasibility Check Workflow](./WORKFLOWS.md#2-feasibility-check-is-this-possible-workflow)
- [Code Verification](./WORKFLOWS.md#3-code-verification--execution-workflow)
