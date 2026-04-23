---
id: '042'
name: vector-embedding-strategy
title: 'FEEDBACK.md: Vector Database & Embedding Model Strategy'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# FEEDBACK.md: Vector Database & Embedding Model Strategy

This document captures questions, concerns, and resolutions for the 042 initiative.

---

## 2026-01-20: Initial Strategy Review

**Source**: Research Team
**Topic**: ELNA vs ArcMind Selection

**Question/Concern**:
Should we wait for ArcMind's multi-modal support instead of committing to ELNA?

**Resolution**:
ArcMind's multi-modal roadmap is promising but unproven. ELNA is production-ready now. Decision: Proceed with ELNA, monitor ArcMind for future multi-modal needs.

**Decision**: → DEC-042-001 (ELNA Selection)

---

## 2026-01-20: Embedding Dimension Trade-off

**Source**: Research Team
**Topic**: 384 vs 1536 dimensions

**Question/Concern**:
OpenAI's `text-embedding-3-large` (3072 dim) offers better quality. Why standardize on 384?

**Resolution**:
- 384-dim models achieve ~95% quality of larger models at 4x lower cost.
- 384-dim is feasible for future on-chain inference (Ignition).
- Quality difference is negligible for typical Nostra use cases (notes, entities).

**Decision**: → DEC-042-002 (384 Dimension Standard)

---

## 2026-01-20: Hybrid Path Trigger Point

**Source**: Research Team
**Topic**: When to migrate to hybrid architecture

**Question/Concern**:
At what scale should we trigger migration to Path B (Hybrid)?

**Resolution**:
- ELNA performs well up to ~100K vectors per canister.
- Set alert threshold at 80K (80% of limit).
- Begin Path B preparation when any Space approaches this threshold.

**Decision**: → DEC-042-003 (Path A with Path B fallback)

---

## Open Questions

*All initial open questions have been resolved. See below.*

---

## Resolved Feedback

| Date | Topic | Resolution | Decision ID |
|:-----|:------|:-----------|:------------|
| 2026-01-20 | ELNA vs ArcMind | ELNA selected, monitor ArcMind | DEC-042-001 |
| 2026-01-20 | Embedding dimension | 384-dim standard | DEC-042-002 |
| 2026-01-20 | Scaling trigger | 80K threshold, Path B fallback | DEC-042-003 |
| 2026-01-20 | Cross-Space Search | Yes for public (configurable), parent spaces for private | DEC-042-007 |
| 2026-01-20 | Embedding Model Upgrades | Dual-index migration with lazy re-embedding | DEC-042-008 |
| 2026-01-20 | ICP Ignition Benchmark | No public benchmarks; prototype required | DEC-042-009 |

---

## 2026-01-20: Cross-Space Semantic Search (Resolved)

**Source**: User Feedback
**Topic**: OQ-042-001 - Cross-Space Semantic Search

**Question**: Should semantic search span multiple Spaces (with permissions)?

**User Guidance**:
- **YES** for **public Spaces** (configurable per Space)
- **YES** for **private parent Spaces** (hierarchical access)

**Resolution**:
Implement federated semantic search with the following rules:
1. **Public Spaces**: Opt-in flag `allow_cross_space_search: true` in Space settings.
2. **Private Spaces**: Search includes parent Space vectors (inheritance model).
3. **Privacy**: Only vectors the user has read access to are included.
4. **Architecture**: Query router fans out to authorized Space canisters.

**Decision**: → DEC-042-007 (Cross-Space Search Policy)

---

## 2026-01-20: Embedding Model Upgrades (Resolved)

**Source**: User Feedback + Web Research
**Topic**: OQ-042-002 - Embedding Model Upgrade Strategy

**Question**: What happens when `nostra-embed-v1` is superseded by `nostra-embed-v2`?

**Research Findings** (Industry Best Practices 2025):
1. **Full Re-embedding is Required**: Mixing embeddings from different models degrades search quality significantly.
2. **Dual-Index Migration**: Run parallel indexes (old + new) during transition.
3. **Lazy Re-embedding**: Re-embed documents on access or in background batches.
4. **Version Metadata**: Store `model_id` with every vector for tracking.
5. **Rollback Capability**: Keep old index until new index is fully validated.

**Resolution - Recommended Strategies**:

| Strategy | Use Case | Pros | Cons |
|:---------|:---------|:-----|:-----|
| **1. Dual-Index + Gradual Migration** (Recommended) | Large datasets, production | Zero downtime, easy rollback, A/B testing | 2x storage during migration |
| **2. Lazy Re-embedding** | Frequently accessed content | Cost-efficient, prioritizes active data | Incomplete coverage, mixed results |
| **3. Full Re-embedding (Big-Bang)** | Small datasets, staging | Clean slate, simplest | Downtime, high compute burst |
| **4. Version-Aware Search** | Transition period only | No re-embedding needed | Score normalization complexity |

**Best Path Forward**:
> **Primary**: Dual-Index + Gradual Migration (Strategy 1)
> **Secondary**: Lazy Re-embedding for cold data (Strategy 2)

**Implementation**:
1. Create new index namespace: `vectors_v2`.
2. Route new inserts to both `vectors_v1` and `vectors_v2`.
3. Background job re-embeds existing vectors to `vectors_v2`.
4. Track migration progress (% of vectors migrated).
5. Alert at 95% completion → switch query routing to `vectors_v2` only.
6. Archive `vectors_v1` after validation period (30 days).

**Decision**: → DEC-042-008 (Embedding Upgrade Strategy)

---

## 2026-01-20: ICP Ignition Benchmark (Resolved)

**Source**: User Request + Web Research
**Topic**: OQ-042-003 - ICP Ignition AI Benchmarks

**Question**: Has anyone benchmarked embedding models on ICP Ignition?

**Research Findings**:

### Available ICP AI Benchmarks (2024-2025)

| Benchmark | Task | Performance | Source |
|:----------|:-----|:------------|:-------|
| Neural Network Image Classification | Image → Label | ~20B Wasm instructions, ~10 seconds | DFINITY Demo (2024) |
| GPT-2 Inference | Text generation | Demonstrated on-chain | Cyclotron Milestone |
| IC Performance General | Update calls | 11,500/sec, 1s finality | DFINITY Benchmarks |
| IC Performance General | Query calls | 250,000+/sec, 1ms latency | DFINITY Benchmarks |

### Embedding Model Specific

**No public benchmarks** exist for embedding models (e.g., `all-MiniLM-L6-v2`) running on ICP Ignition as of 2026-01-20.

**Why?**
- Ignition (completed Aug 2025) focused on LLM inference (prompts → responses).
- Embedding models are simpler but may not be prioritized in public demos.
- Community projects (ELNA, ArcMind, Kinic) use off-chain embedding generation.

### Cost Estimation for Embedding Models

Based on ICP cycle costs:
- 1 Wasm instruction ≈ 1 cycle
- Embedding model forward pass (384-dim, ~22M params) ≈ 50-100M instructions
- **Estimated Cost**: 50-100M cycles per embedding (~$0.00005-0.0001 USD)
- **Latency**: Likely 1-5 seconds for single embedding (based on NN demo)

### Best Path Forward

1. **Short-Term (Now)**: Continue using off-chain embedding (OpenAI/Local). Cost: ~$0.0001/1K tokens, Latency: ~100ms.

2. **Mid-Term (Q2-Q3 2026)**: Prototype `all-MiniLM-L6-v2` on ICP.
   - Use `dfx llm` or Ignition AI Worker pattern.
   - Benchmark: cycles cost, latency, accuracy.
   - Target: < 500ms latency, < 100M cycles.

3. **Long-Term (2026+)**: If ICP embedding is viable:
   - Create `IcpNativeEmbedder` provider.
   - Migrate embedding generation on-chain for privacy-critical Spaces.

**Recommendation**:
> **Do not block on ICP native embeddings.** The off-chain approach (OpenAI → Worker → ELNA) is proven and cost-effective. Revisit when DFINITY publishes embedding benchmarks or when Nostra has a privacy-critical use case requiring on-chain embedding.

**Action Items**:
- [ ] Monitor DFINITY forums for embedding model benchmarks.
- [ ] Q2 2026: Prototype `all-MiniLM-L6-v2` ONNX on ICP canister.
- [ ] Document benchmark results in `research/042-vector-embedding-strategy/benchmarks/`.

**Decision**: → DEC-042-009 (ICP Embedding Strategy)
