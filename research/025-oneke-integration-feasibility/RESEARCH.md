---
id: '025'
name: oneke-integration-feasibility
title: OneKE Integration Feasibility
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-29'
---

# OneKE Integration Feasibility

> [!IMPORTANT]
> **Consolidation Note**: This research provides the **Extract stage** pattern for [037-nostra-knowledge-engine](../037-nostra-knowledge-engine/RESEARCH.md). Implementation target is 037, not a standalone OneKE deployment.

## Objective
Analyze OneKE (One-shot Knowledge Extraction) for integration into the Nostra/ICP ecosystem, specifically focusing on its ability to address unaddressed needs and enhance future feature planning. Compare its features with other extraction agents.

## Methodology
1. Analyze OneKE's capabilities via local installation and documentation.
2. Cross-reference with existing research initiatives (`002`, `013`, `021`).
3. Comparative analysis with other extraction agents.
4. Feasibility assessment and recommendations.

## Analysis of OneKE

### Capabilities
OneKE is a schema-guided extraction framework that orchestrates three agents:
1.  **Schema Agent**: Deduced or Retrieved schemas (Pydantic models). Matches KIP's `Concept` definitions.
2.  **Extraction Agent**: Performs the extraction, optionally using "Case Retrieval" (few-shot examples from a vector DB) to improve accuracy.
3.  **Reflection Agent**: Verifies and corrects output.

### Architectural Fit
1.  **Workflow Engine (`013`)**: OneKE fits perfectly as an `AsyncExternalOp` worker.
    -   **Input**: Unstructured text (Description, Chat, PDF).
    -   **Process**: OneKE Pipeline (Schema -> Extract -> Reflect).
    -   **Output**: Structured KIP `UPSERT` commands or `Entity` objects.
2.  **KIP Integration (`021`)**:
    -   OneKE's `TripleExtraction` mode maps directly to KIP's `Subject -> Predicate -> Object` model.
    -   The `schema_repository.py` can be auto-generated from `N-Lib` manifests (`.kip` files).
3.  **Nostra V2 (`002`)**:
    -   Bridges the "Temporal View" (Activity Stream) and "Relational View" (Graph).
    -   Can run as a "Shadow Agent" listening to the stream and proposing graph updates.

### Integration Feasibility
-   **Local Install**: Already present at `/Users/xaoj/ICP/OneKE`.
-   **Configuration**: `yaml` based, easy to generate dynamically.
-   **Model Support**: Supports DeepSeek and LLaMA, suggesting it can run on local hardware or via API, aligning with Nostra's "Local First" or "Hybrid" AI approach.

## Comparative Analysis

| Feature | OneKE | Standard LLM Prompting | REBEL / Babelscape | GraphRAG (Microsoft) |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Goal** | Schema-Guided Extraction | General Generation | Relation Extraction | Retrieval / Q&A |
| **Structure** | Enforced via Pydantic & Schema Agent | Loose (JSON mode helps) | Fixed Ontology | Community Detection |
| **Accuracy** | High (Reflection + Case Retrieval) | Variable | High (for specific relations) | N/A (Retrieval focus) |
| **Customization** | Python-based Agent/Schema Logic | Prompt Engineering | Finetuning required | Dynamic |
| **Integration** | **High**: Python API, Modular Agents | Medium: Requires wrapper | Low: Model-only | Medium: Complex stack |
| **Nostra Fit** | **Best**: Fits "Worker" pattern & KIP Schema | Good for simple tasks | Too rigid | Overkill / Different purpose |

## Recommendations
1.  **Adopt OneKE as the standard "Extraction Worker"**.
2.  **Develop a "KipSchemaGenerator"**: A tool to convert KIP `.kip` definitions into OneKE Pydantic models.
3.  **Deploy as `AsyncExternalOp`**: Wrap OneKE in a Python worker that listens for Nostra Workflow tasks.
