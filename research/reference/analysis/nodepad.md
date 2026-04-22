# Nodepad Analysis

**Repository**: [mskayyali/nodepad](https://github.com/mskayyali/nodepad)
**Analyzed**: 2026-04-08
**License**: MIT
**Primary Language**: TypeScript (Next.js, React 19)
**Steward**: User Strategy

---

## 1. Intent / Executive Summary

Nodepad is a **spatial, AI-augmented thinking canvas** designed to move away from sequential chat interfaces toward a spatial/associative knowledge layout. It automatically classifies notes into 14 distinct types (e.g., claim, question, idea, task, entity), infers connections between notes conceptually, and uses LLMs in the background to surface novel syntheses or emergent insights across the canvas. Everything is strictly local-first and API keys are stored in the browser.

> [!IMPORTANT]
> Nodepad's most significant contribution to Nostra Cortex is its **spatial UX paradigm for sensemaking** combined with **agentic background synthesis**. This perfectly aligns with our vision for A2UI's "Heap Mode" and our goals for graph-based knowledge interactions, moving away from ephemeral chatbot UIs toward durable, spatial substrates.

---

## 2. Placement

Path: `research/reference/repos/nodepad`
Kind: `repo`
Topic Fit: Cross-topic (Visualization, Agent Systems, UI Substrate)

## 3. Possible Links To Nostra Platform and Cortex Runtime

### 3.1 Spatial vs Conversational UI (HIGH VALUE)
Nodepad replaces the chat log with a three-view canvas (Tiling, Kanban, Graph). For Cortex apps, navigating user knowledge visually in a unified space (like a "Space" or "Heap") is a primary goal. Nodepad gives a strong reference for building a D3/React spatial UI that scales conceptually.

### 3.2 Background Agent Synthesis (HIGH VALUE)
Instead of blocking interaction to wait for LLMs, Nodepad's AI acts quietly in the background—classifying notes upon creation, identifying implicit links, and volunteering syntheses when critical mass is reached. This is an excellent orchestration pattern for our background execution agents.

### 3.3 Semantic Note Ontology (MEDIUM VALUE)
Nodepad uses 14 explicit categories. Nostra currently defines the foundational `Contribution` graph entity in `shared/ontology/core_ontology_v1.json`, but currently lacks formal domain sub-types. Nodepad provides a strong practical reference for bridging this ontological gap and meaningfully categorizing graph blocks inside a Space.

### 3.4 Local-First Multi-Provider Implementation (MEDIUM VALUE)
Browser-to-LLM direct connection minimizes backend footprint. While Cortex relies on `cortex-eudaemon` for provider orchestration, Nodepad's client-side provider management patterns (switching APIs, OpenRouter + direct providers) can inform developer/operator settings UIs.

---

## 4. Pattern Extraction

| Nodepad Pattern | Nostra Analog | Status |
|-----------------|---------------|--------|
| Multi-View Canvas (Graph/Tiling/Kanban) | A2UI Heap Mode / Spatial catalogue experiments | Experimental in `cortex-web` |
| Background Semantic Linking | Eudaemon Graph Traversal | Scaffolded concept, not runtime-wired |
| 14-Type Classification | Contribution semantic hint comparison | Experimental reference mapping only |
| Emergent Synthesis Prompts | Agentic Workflow Synthesis | Conceptual |

---

## 5. Scorecard

| Criterion | Score (0-5) | Rationale |
|-----------|------------|-----------|
| `ecosystem_fit` | 4 | React/Next.js/D3 stack perfectly aligns with our `cortex-web` frontend. |
| `adapter_value` | 3 | UI components (like BSP tiling) might be cleanly portable or translatable to Lit/React. |
| `component_value` | 5 | Very strong UI components for spatial canvases and graph visualization. |
| `pattern_value` | 5 | Background synthesis and auto-classification are highly relevant architectural patterns. |
| `ux_value` | 5 | A premium UX reference for AI-assisted spatial knowledge work. |
| `future_optionality` | 4 | Supports future initiatives involving graph views and PKM semantics. |
| `topic_fit` | 3 | Crosses multiple domains (visualization, UI, agents) without perfectly capturing just one. |
| **Total** | **29/35** | Passes intake threshold (≥12). |

---

## 6. Adoption Decision

**Decision**: Adopt as a formal UX & Pattern Reference.
Nodepad will serve as a foundational reference repo for the `124-polymorphic-heap-mode` and UI visualization initiatives. We will not adopt its core repository entirely, but will study its source to extract components, layout algorithms (BSP grid, d3 force-directed), and user-interaction patterns.

---

## 7. Known Risks

- **Browser-bound Local State**: Nodepad relies on `localStorage` which doesn't map to our rigorous Nostra data model or CRDT synced state architectures.
- **Client-Side LLM Execution**: Nostra Cortex typically routes through an execution engine (Eudaemon / Temporal), so direct API calls from the client aren't directly portable to our standard architecture.
- **Upstream Drift**: It is described as a "design experiment" and may be abandoned or rapidly changed.

---

## 8. Current Status Snapshot

### Implemented Now
- `cortex-web` already contains a React `SpatialHeapGrid` renderer and a `/labs/layout-catalogue` route for layout comparison.
- The current hardening phase keeps these surfaces experimental and Cortex-local rather than promoting a shared runtime contract.

### Experimental Proposal
- `shared/ontology/nodepad_to_contribution_mapping.md` remains a reference comparison document only.
- Nodepad note types may inform local renderer hints and UX experiments, but they are not canonical Nostra ontology terms in this phase.

### Deferred Contract Candidate
- A Cortex presentation-level `layoutPrimitive` contract remains a valid future direction, but only once at least two live surfaces or hosts need the same vocabulary.
- Background synthesis should only be promoted after it is wired into an active Eudaemon runtime entry point rather than an orphan scaffold.

## 9. Suggested Next Experiments

1. **Harden Existing Spatial Experiments**: Keep the React `SpatialHeapGrid` path, normalize catalogue vocabulary to topology-oriented names inside `cortex-web`, and maintain narrow renderer tests.
2. **Keep Ontology Mapping Non-Canonical**: Continue comparing Nodepad's 14 note types against `Contribution` semantics without introducing a Nostra platform enum or shared metadata contract yet.
3. **Defer Shared Layout Contract Promotion**: Reserve a future Cortex-only `layoutPrimitive` concept for the point where two real consumers need the same layout-family vocabulary.
4. **Revisit Background Synthesis After Runtime Wiring Exists**: Use Nodepad's background agent pattern as a reference only after Eudaemon has a real trigger and worker registration path for this class of synthesis.

---

## 10. Initiative Links

- `124-polymorphic-heap-mode` (Direct UI relation)
- `115-cortex-viewspec-governed-ui-synthesis` (A2UI Componentry)
- `075-cortex-visualization-graph` (D3 usage)

---

## 11. Authority & Governance

| Field | Value |
|-------|-------|
| `primary_steward` | User Strategy |
| `authority_mode` | `recommendation_only` |
| `escalation_path` | `steward_review -> owner_decision` |
| `lineage_record` | `research/REFERENCE_MIGRATION_LINEAGE.md` |
| `initiative_refs` | `124-polymorphic-heap-mode`, `115-cortex-viewspec-governed-ui-synthesis`, `075-cortex-visualization-graph` |
