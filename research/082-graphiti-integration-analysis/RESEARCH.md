---
id: 082
name: graphiti-integration-analysis
title: 'Research Findings: Graphiti Architecture'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-29'
---

# Research Findings: Graphiti Architecture

## 1. Executive Summary
Graphiti provides the **Reference Implementation** for a Temporal Knowledge Graph.
-   **Extraction**: Uses a "Reflexion" loop (Extract -> Check for Misses -> Finalize) which is superior to single-pass extraction.
-   **Search**: Implements **Hybrid Search** using Reciprocal Rank Fusion (RRF) to merge Vector + Keyword + Graph Traversal results.
**Recommendation**: Port the "Reflexion" prompt patterns to our OneKE strategy and implement the RRF algorithm in the `NostraGraph` canister.

> [!IMPORTANT]
> **Consolidation Note**:
> - **Reflexion pattern** → implements in [037-nostra-knowledge-engine](../037-nostra-knowledge-engine/RESEARCH.md) (Reflect stage)
> - **RRF algorithm** → implements in NostraGraph canister search method

## 2. Prompt Engineering Analysis

### 2.1 The "Reflexion" Pattern (`extract_nodes.py`)
Graphiti doesn't just ask to "extract entities". It uses a multi-stage process:
1.  **Extract**: `extract_text` prompt asks for explicit/implicit entities.
2.  **Reflexion**: `reflexion` prompt asks: *"Given extracted entities, determine if any have been missed."*
3.  **Classification**: `classify_nodes` maps them to strict Schema types.

### 2.2 Relevance to Nostra
We should adopt this 3-step loop in the **Knowledge Workbench**.
-   Step 1: Raw OneKE extraction.
-   Step 2: LLM "Reflexion" pass (using Graphiti's prompt).
-   Step 3: Schema mapping (using Nostra's `040` Enums).

## 3. Search Algorithm Analysis

### 3.1 Hybrid Search Logic (`search.py`)
Graphiti's `search()` function is a **Scatter-Gather** operation:
1.  **Scatter**: Run parallel searches:
    -   `node_search` (Vector + Keyword)
    -   `edge_search` (Vector + Keyword)
    -   `community_search` (Vector)
2.  **Graph Expansion**: If `BFS` is enabled, expand from the top nodes to find neighbors.
3.  **Rerank (RRF)**: Merge the lists using **Reciprocal Rank Fusion**:
    `score = 1.0 / (k + rank_i)`
    This normalizes scores between disparate methods (BM25 vs Cosine).

### 3.2 Adaptation
The `NostraGraph` canister should expose a `search()` method that implements this logic.
-   **Vector Index**: Already planned (`041`).
-   **Graph Traversal**: Needs an adjacency list in the canister.
-   **RRF**: Simple math to implement on the result sets.

## 4. Adaptation Recommendations

| Feature | Graphiti Implementation | Nostra Port |
| :--- | :--- | :--- |
| **Extraction** | `examples/extract_nodes.py` (Reflexion) | **Knowledge Workbench** (Python Worker) implementing the 3-step loop. |
| **Search** | `search.py` (Python asyncio) | `NostraGraph` (Rust Canister) implementing parallel lookups + RRF. |
| **Storage** | Neo4j/FalkorDB | `NostraGraph` (Custom Rust/IC implementation). |
