---
id: '041'
name: nostra-vector-store
title: 'Feedback: Nostra Vector Store (041)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback: Nostra Vector Store (041)

## 2026-01-19: Semantic Incompatibility
- **Source**: User
- **Question/Concern**: How do we handle different embedding dimensions from different users?
- **Resolution**: Standardization of `nostra.vector` domain in Initiative 040; mandating specific dimensions for shared indices.
- **Decision**: → DEC-002 (Init 040).

## 2026-01-18: Performance vs Privacy
- **Source**: AI Agent
- **Question/Concern**: On-chain embedding generation is slow.
- **Resolution**: Recommendation to use off-chain inference (Ollama/OAI) while keeping vector *storage* on-chain via Elna.
- **Decision**: → DEC-001.
