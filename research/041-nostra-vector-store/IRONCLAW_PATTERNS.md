---
source: research/reference/topics/agent-systems/ironclaw
intake_date: 2026-02-21
status: adopted_patterns
---

# IronClaw Memory Search Patterns — Nostra Adaptation

Patterns extracted from ironclaw `src/workspace/search.rs` and adapted for Nostra's knowledge graph retrieval semantics. IronClaw operates on flat local documents; Nostra operates on a structured contribution version graph with governance constraints. These patterns inform the vector store layer only (`vector_db` side per §1.6 Visibility Decoupling).

---

## Pattern 1: BM25 + Vector Hybrid with Reciprocal Rank Fusion (RRF)

**IronClaw approach**: Run BM25 full-text search and vector similarity search independently, then merge results using Reciprocal Rank Fusion — each document's final score is `Σ 1/(k + rank_i)` across all ranking lists.

**Nostra adoption**: Applicable directly to contribution retrieval. The existing `SearchResult` in `cortex-runtime/src/agents/types.rs` has a `score: f32` field — the merged RRF score maps there. No type changes required.

**Priority**: High — this is the foundational retrieval architecture. IronClaw's implementation confirms RRF is practical at this layer.

**Open question**: What is the right `k` constant for Nostra's contribution domain? IronClaw uses `k=60` (standard). May need tuning against contribution graph depth.

---

## Pattern 2: Temporal Decay Scoring

**IronClaw approach**: Applies a time-based weight factor to RRF scores, boosting recent documents. Score is multiplied by `exp(-λ * age_days)` where λ controls decay steepness.

**Nostra adaptation**: Use `version_chain_age` (age of the contribution version in the chain) rather than document age. A recently proposed contribution should score higher in agent retrieval context even if the root entity is old.

**Priority**: Medium — needed once the agent retrieval pipeline is live. Not needed until baseline hybrid search is proven.

**Constraint**: Decay must respect contribution governance — a locked/archived contribution version should not be boosted by recency. The decay weight must be applied after a governance-filter pass.

---

## Pattern 3: MMR Re-ranking (Maximal Marginal Relevance)

**IronClaw approach**: Post-RRF diversity pass. Given a result set R, iteratively select the next result that maximizes `λ·sim(doc, query) − (1−λ)·max(sim(doc, selected))`. Penalizes near-duplicate results.

**Nostra adaptation**: Applicable to knowledge graph traversal where multiple nodes from the same contribution cluster might otherwise dominate results. Reduces contribution-cluster redundancy in agent context windows.

**Priority**: Low — meaningful only once multi-node graph retrieval is in use. Single-node lookups don't benefit.

---

## Pattern Not Adopted: LLM Query Expansion

**IronClaw approach**: Rewrites the user query via a secondary LLM call before FTS/vector search.

**Verdict**: Deferred indefinitely. Adds a full LLM round-trip latency to every search. Justified only if retrieval recall is measurably insufficient — and that measurement requires a working baseline first.

---

## Integration Notes

- All three adoptable patterns operate on the `vector_db` side (§1.6) — they are indexing and retrieval concerns, not source-of-truth mutations.
- RRF and temporal decay integrate into `SearchResult.score: f32` — no struct changes needed at adoption time.
- MMR runs as a post-processing pass over a `Vec<SearchResult>` — pure function, no state.
- Governance filter must precede any scoring: archived or locked contributions should be excluded before RRF, not penalized after.
