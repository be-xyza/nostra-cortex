# Docs DPub & Space Strategy

## 1. Goal Description
The objective is to establish an architecture for generating and maintaining human- and machine-readable documentation that accurately reflects the current state of a Space or Institution. This documentation must be continuously updated, universally accessible, and cryptographically verifiable across both Nostra (backend/canister) and Cortex (local/desktop) environments.

## 2. Context & Existing Coverage
This requirement is heavily supported by existing research initiatives, specifically:
- **080-dpub-standard**: Defines the architecture for Decentralized Publishing (VFS-managed, Graph-backed). Creates immutable `SnapshotManifest` and `DocsBundle` contracts.
- **007-nostra-spaces-concept**: Defines Spaces as heavyweight containers and Views as lightweight queries over Polymorphic Blocks.
- **094-institution-modeling**: Models Institutions as first-class Contributions within Spaces, with descriptive lifecycles and Charter References.
- **124-polymorphic-heap-mode**: Introduces Universal Polymorphic Blocks which can blend text, widgets, data, and pointers.
- **042-vector-embedding-strategy** & **051-rag-ingestion-pipeline**: Specifies Agent-Aware Embeddings, Logical Form-Guided Hybrid Similarity Scoring (stacking RRF and Schema Planning), Query-Adaptive scoring weights, Bidirectional Chunk-Entity Cross-Indexing, and the multi-step Schema-Guided Reflexion Extraction pipeline for semantic search.
- **eval_driven_orchestration_deep_dive.md**: Introduces empirical A/B subagent benchmarking and `feedback.json` iteration loops for deterministic agent behavior.

## 3. High-Level Strategy: "The Institutional Docs DPub"

To maintain sync between the live state of a Space/Institution and its documentation, we should adopt a **"Docs DPub as a Materialized View"** paradigm.

### The Mechanism
1. **The Source of Truth**: The Space/Institution's current state lives in the Nostra Graph (Contributions, Decisions, Member Roles, Event Logs).
2. **The Docs Space (View)**: Instead of a separate isolated Space, the "Docs" are a specialized **Space View**. This View dynamically queries the Graph for the Institution's Charter References, Governance Decisions, and AI Agent Logs.
3. **The Docs DPub (Snapshot)**: Periodically (or upon a release/workflow trigger), the State of the Docs View is compiled into a formal **DPub Edition**. This process captures the dynamic graph data and serializes it into:
   - *Human-Readable*: Markdown documents with inline Polymorphic Blocks (`@[id]`).
   - *Machine-Readable*: A JSON `SnapshotManifest` and schema-aligned metadata representing the exact state at that time.

## 4. How this works in Cortex & Nostra

### Nostra (The Network & Storage Layer)
- **VFS Integration**: The Space's documentation resides in the Virtual File System (e.g., `/lib/spaces/<space-id>/docs/`).
- **Glass Box Agents & Eval-Driven Drafting**: "Librarian" orchestrators run periodic hygiene workflows. Instead of blindly drafting, they use **A/B Subagent Testing**: generating multiple drafts, dropping benchmark telemetry (`benchmark.json`), and using a Grader to select the one that best aligns with the Institution's Charter.
- **Workflow Governed**: These updates must pass a `publish-edition` workflow gate (implemented in `nostra_contribution_cli`). Human approval will operate via a planned asynchronous `feedback.json` contract where stewards provide structured feedback before the immutable DPub snapshot is compiled.

### Cortex (The Desktop & Interaction Layer)
- **Artifacts Editor**: Contributors use the Cortex Artifacts Editor to draft docs. Since Cortex mounts the Nostra VFS natively, editing a document locally maps directly to a Contribution graph update in Nostra.
- **Local-First Consumption**: Cortex ingests the `DocsBundle` ZIP/Snapshot. For reading, Cortex renders the Polymorphic Blocks locally, pulling down rich context (like an interactive governance voting block) embedded right inside the human-readable text.
- **Semantic Search Strategy (Logical Form Hybrid)**: Cortex local agents (or users interacting via the UI) will query the Docs DPub using the **Query-Adaptive Hybrid Similarity Framework (DEC-042-012)**. This search dynamically blends 384-dimensional semantic vectors (`text_vectors` namespace) with graph distance, tags, and shared lineage, with planned support for bidirectional cross-indexing (`source_chunk_ids` per DEC-042-020) and Schema-Constrained Logical Forms (DEC-042-005) for complex multi-hop queries. All embeddings produced during docs digestion are **Agent-Aware (DEC-042-013)**, recording which agent chunked/extracted the docs.

## 5. Aggregation & Orchestration
How do we aggregate institutional state into docs?

1. **Tagging & Ontologies**: When a governance decision is made (e.g., `DEC-094-001`), it is tagged appropriately in the graph.
2. **Dynamic Assembly (`assembleEdition`)**: The DPub Manifest acts as a blueprint. A chapter like "Current Governance" isn't a static markdown file; it is a query resolving out to `#[Governance] && status:active`.
3. **Agent Synthesis**: For human-readable summaries, an LLM agent consumes the raw Graph results (Machine Readable) and synthesizes a prose summary (Human Readable) as a new `NostraBlock`. The agent traces are saved alongside the docs in the VFS (`/agents/<agentId>/trace.json`), ensuring Glass Box transparency.

## 6. Resolving with Principles (The Optimal Path Forward)

To provide guidance on implementing this, we apply Nostra/Cortex core principles:

- **Everything is a Contribution**: The Docs DPub itself is a Contribution. It has versioning, lineage, and can be forked if an institution splits.
- **Empirical Execution & Glass Box Agents**: Do not let agents silently update docs. Agents *propose* document updates using A/B subagent strategies, emitting telemetry and using the `feedback.json` asynchronous loop to empower Institutional Stewards to explicitly approve or steer the new DPub Edition.
- **VFS as Glue**: Map the documentation logically in the VFS so standard tools work, but back it entirely by the Graph so it remains richly queryable.
- **Capability Containment**: Agents aggregating cross-space documentation must possess explicit capability tokens to read beyond their host space, as governed by the cross-space search policy (DEC-042-007), preventing data leakage.
- **VFS Portability for Embeddings**: Embedding metadata manifests should be stored as VFS artifacts (`/lib/spaces/<space-id>/vectors/manifest.json`) to ensure CEI records remain portable for offline re-indexing in Cortex Desktop, addressing the current gap where embeddings exist outside VFS.
- **Attributed Graph Knowledge**: As per `051-rag-ingestion-pipeline` and `DEC-042-019`, DPub Docs must pass through the Ingestion Tube consisting of a Schema-Guided Reflexion Loop (Chunking -> Extract -> React/Check for Misses -> Schema Classify -> Index). This produces cleaner entities and establishes the foundation for the cross-indexed Logical Form-Guided Hybrid Search to function correctly against Institutional docs.

### Next Steps / Action Items
1. Define a specific `DocsBundle` schema extending the existing `080-dpub-standard` specifically tailored for institutional state.
2. Abstract the Grader Logic from the eval-driven pattern into a specific "Docs Librarian Architect" subagent template.
3. Establish the formal "Docs Space View" default layout for the Artifacts Editor.
4. Ensure the Cortex Ingestion pipeline routes Institutional DPub chapters through the multi-stage `ReflectStage` graph extractor, establishes the bidirectional cross-index between vectors and entities (`DEC-042-020`), and tags vectors with Agent Provenance.
5. Upgrade the Cortex Search Router to route complex factual queries through Logical Form definitions rather than relying on pure RRF vector similarity.
