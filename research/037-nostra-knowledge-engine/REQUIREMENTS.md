---
id: '037'
name: nostra-knowledge-engine
title: 'Requirements: Nostra Knowledge Engine'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Nostra Knowledge Engine

## Functional Requirements
1.  **Multi-Format Ingestion**:
    - MUST support CSV (essential for bulk data).
    - MUST support JSON (structured imports).
    - SHOULD support Markdown (parsing frontmatter).
2.  **KIP Compliance**:
    - Output MUST be valid `ConceptNode` and `PropositionLink` objects as defined in `021-kip-integration`.
    - MUST support bootstrapping new Type definitions if the schema doesn't exist.
3.  **User Agency**:
    - Users MUST explicitly approve the mapping before conversion.
    - Users MUST be able to edit individual entities during the "Preview" phase.

## Non-Functional Requirements
1.  **Performance**: Parsing 10k rows should happen in < 2 seconds (Wasm).
2.  **Privacy**: Data conversion MUST happen client-side; raw data is not sent to server until committed as entities.
3.  **Reusability**: The conversion logic SHOULD be a reusable Rust crate (`nostra-loom-core`) also usable by CLI tools.
