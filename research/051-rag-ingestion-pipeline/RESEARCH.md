---
id: '051'
name: rag-ingestion-pipeline
title: 'Research 051: RAG & Ingestion Pipeline Analysis'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research 051: RAG & Ingestion Pipeline Analysis

> **Status**: IN PROGRESS
> **Focus**: Deconstructing `graphiti` (Entity Extraction) and `elna` (Vector Storage) to build the Nostra Ingestion Pipe.

## 1. Source Analysis

### A. Graphiti (Python)
Graphiti allows for "Episodic" ingestion.
*   **Key Flow**: `add_episode` -> `_extract_and_resolve_nodes` -> `add_nodes_and_edges_bulk`.
*   **Entity Extraction**: Uses `LLMClient` to parse text into `EntityNode` and `EntityEdge`.
*   **Resolution**: Critical step `dedupe_nodes_bulk` merges new entities with existing ones using semantic similarity.
*   **Storage**: Neo4j (Graph) + Vector Index (Embeddings).

### B. Elna (Rust/Canister)
Elna provides a Vector Database on the IC.
*   **Interface**: `insert(index, vectors, metadata)` and `query(index, vector, k)`.
*   **Limitations**: Purely vector storage. No graph logic or entity resolution.

### C. OneKE / OpenSPG (Inspiration)
Located in `OpenSPG/OneKE`. A schema-guided extraction framework.
*   **Concept**: Uses a "Schema Repository" (Pydantic models) to strictly define what to extract.
*   **Agents**:
    *   **Schema Agent**: Deduced schema from text.
    *   **Extraction Agent**: Direct extraction or Case Retrieval.
    *   **Reflection Agent**: Self-consistency checks.
*   **relevance**: We should adopt the **Schema Repository** concept. Instead of asking the LLM to "extract everything", we should pass it a `NostraSchema` (e.g. `EntityType::Project` vs `EntityType::Idea`) to guide the extraction.

## 2. The "Nostra Gap"
To bring Graphiti's power to Nostra (Rust/Motoko), we need to port the **Extraction & Resolution** logic.

| Component | Graphiti (Python) | Nostra Target (Rust/Motoko) |
| :--- | :--- | :--- |
| **Extractor** | LLM-based `extract_nodes` | **New**: `nostra_ingest` (Rust) using LLM Canister |
| **Resolver** | `dedupe_nodes_bulk` (Python) | **New**: `EntityResolver` Actor (Motoko) |
| **Storage** | Neo4j + Vector | **New**: `Graph` Module (Motoko) + `Elna` (Canister) |

## 3. Proposed Architecture: The "Ingestion Tube"

We will treat Ingestion as a specialized Workflow (`013-workflow-engine`):

1.  **Trigger**: DPub/Chapter Published (from `080-dpub`).
2.  **Step 1: Chunking** (Rust/Wasm): Split text into windows.
3.  **Step 2: Extraction** (LLM): Call `LLM` to identify Entities/Relations.
4.  **Step 3: Resolution** (Backend):
    *   Query `Graph` for existing entities.
    *   Merge or Create new IDs.
5.  **Step 4: Indexing** (Elna):
    *   Store `(Vector, ChunkID)` in Elna.
    *   Store `(EntityID, Vector)` in Elna.

## 4. Next Steps
1.  Define the **Ingestion Workflow Schema**.
2.  Prototype the **Resolution Logic** in Motoko (the hardest part).
