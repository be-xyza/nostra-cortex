# Capability Matrix: Nostra/Cortex vs TrustGraph

**Context**: Cross-reference of capabilities between TrustGraph and our Nostra/Cortex ecosystem.
**Purpose**: Track covered components, analogous patterns, gaps, and prioritized adoption opportunities.
**Last Updated**: 2026-03-23
**Related**: [TrustGraph Analysis](./trustgraph.md) | [078 PLAN Phase D](../../078-knowledge-graphs/PLAN.md)

---

## 1. Component Capability Matrix

Status legend: ✅ Production | 🧪 Pilot/Research | ⚠️ Partial | 🔲 Planned | ❌ Gap | 🚫 N/A

*Note: **Pilot/Research** indicates the capability has passed research gates (e.g., 037 DEC-009) or exists as a reference implementation (e.g., ELNA) but is not yet in the main production canisters.*

### Storage & Data

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Graph storage** | ✅ Polyglot graph/query substrate (Cassandra/Scylla + RDF triples, with vector/object companions) | ⚠️ Motoko Map (`motoko-graph`, watch-first) | We have the model (`Contribution`/`Relation`) but no production graph store. M21–24 addresses contracts, not a full runtime port. |
| **Vector embeddings** | ✅ Qdrant (dedicated vector DB) | 🧪 Pilot (ELNA) + ✅ Gen | **Generation**: Full `EmbeddingProvider` (worker-live). **Storage/Similarity**: `ELNA` vector DB is reference-active and passed 037/042 research gates. Not yet a core production service. |
| **Object/file storage** | ✅ Garage (S3-compatible) | 🔲 Planned (`nostra-media`) | `nostra-media` canister planned but not production. TrustGraph validates S3-compatible approach. |
| **Key-value store** | ✅ Cassandra KV mode | ✅ Canister stable memory | Functional parity via different substrate. |
| **Document store** | ✅ Cassandra document mode | ⚠️ Canister blob storage | Limited — no document indexing or chunking pipeline. |
| **Multi-model unification** | ✅ Service mesh across graph, vector, object, workflow, and agent services | ❌ Fragmented across canister types | Each data model is a separate canister; no unified query layer. |
| **Collections/namespacing** | ✅ Collection metadata management | ✅ Spaces (governance-scoped) | Analogous. Spaces provide richer governance than TrustGraph collections. |

### Knowledge Modeling

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Schema definitions** | ✅ Custom schemas (structured data) | ⚠️ JSON schemas (scattered, no registry) | Schemas exist but need consolidation → M21. |
| **Ontology definitions** | ✅ Custom ontologies (unstructured data) | ❌ None | **Critical gap.** No formal domain vocabulary → M22. |
| **Schema/ontology duality** | ✅ Explicit dual model | ❌ Only schemas | Need to formalize the distinction → M22. |
| **Schema registry** | ✅ Config service type management | ❌ No registry | Schemas scattered across `shared/standards/` subdirs; M21 should add an index before any file migration. |
| **Schema versioning** | ✅ Context Core versioning | ❌ None | No versioned schema management. |
| **RDF triple model** | ✅ Full S-P-O with named graphs and RDF-star / quoted triples | ⚠️ `Contribution`/`Relation` (equivalent structure) | Structural parity, but no RDF serialization or named graph scoping by default. |
| **Knowledge bundles** | ✅ Context Cores (portable, versioned) | ❌ None | **High-priority gap** → M23. |
| **Provenance tracking** | ✅ Source manifests in Context Cores | ⚠️ `GlobalEvent.source` + `lineage_record` | Partial — events capture source, but no formal provenance chain bundled with data. |
| **Retrieval policies** | ✅ Per-core traversal rules, freshness, authority ranking | ❌ None | No formalized retrieval strategies for graph data. |

### Query & Retrieval

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Triple pattern query (S-P-O)** | ✅ Full API (`/triples`) | ❌ None | No official programmatic graph query interface yet → M24. |
| **GraphQL over graph data** | ✅ Rows query service | ❌ None | No GraphQL layer over entity/relation data. |
| **NLP → structured query** | ✅ NL→GraphQL with confidence scores | ❌ None | Agent natural language querying of the knowledge graph. |
| **Structured query (all-in-one)** | ✅ NLP→execute→results pipeline | ❌ None | Combined NLP + execution pipeline. |
| **Search (Semantic)** | ✅ Vector similarity search | 🧪 Pilot (ELNA-backed) | 037/042 Benchmarks show semantic search (dense top-k) operational in pilot check paths. |
| **Graph embedding search** | ✅ Cosine similarity over entities | ❌ Gap | Can generate embeddings (worker), but no production vector index for indexed entity retrieval. |
| **Document embedding search** | ✅ Chunk-level similarity search | 🧪 Pilot (042/051) | 042 + 051-rag-ingestion-pipeline research covers bidirectional chunk-entity cross-indexing. |
| **Row embedding search** | ✅ Structured data semantic search | ❌ None | No semantic search over structured data. |

### RAG Pipelines

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Document RAG** | ✅ Chunk-based retrieval from docs | 🧪 Pilot (037 DEC-009) | `037` marks local-first knowledge engine as `ready` for pilot. Includes `POST /knowledge/ask` grounded response. |
| **Graph RAG** | ✅ Entity/subgraph multi-hop with tunable limits | ❌ Gap | **Key Phase D objective.** No relationship-aware RAG pipeline yet. |
| **Ontology RAG** | ✅ Ontology-constrained retrieval | ❌ Gap | Requires ontology layer first (M22). |
| **RAG parameter tuning** | ✅ entity-limit, triple-limit, max-subgraph-size, max-path-length | 🚫 N/A | Would need these knobs if we build RAG. |

### Agent & AI Services

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Conversational agent** | ✅ Stateful agent with reasoning chains | ⚠️ `cortex-eudaemon` (gateway chat, early) | TrustGraph agent is graph-aware. Our agent is chat-only without graph grounding. |
| **Multi-agent system** | ✅ Collaborative agent groups | 🔲 Planned (126-agent-harness) | Designed but not implemented. |
| **MCP tool integration** | ✅ Full MCP server + tool execution | ✅ `ic-rmcp` MCP server | Comparable; our MCP integration is ICP-native. |
| **Text completion** | ✅ Direct LLM generation | ⚠️ Via gateway LLM proxy | Functional but not graph-contextualized. |
| **Prompt management** | ✅ Template-based with runtime variables | ❌ None | No prompt management or template system. |
| **Multi-LLM provider support** | ✅ 7+ providers, local inference options | ⚠️ Single provider per deployment | Limited provider abstraction. |
| **Agent tool definitions** | ✅ Collections + knowledge cores + MCP + tool groups | ⚠️ MCP tools only | No tool grouping, collection-scoped tools, or knowledge core binding. |

### Workflow & Orchestration

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Flow engine** | ✅ Blueprint/instance lifecycle | ⚠️ Workflow authority model (designed, partial) | TrustGraph's blueprint/instance matches our authority model concept. |
| **Flow classes / presets** | ✅ Workflow preset configurations | ❌ None | No workflow templates or presets. |
| **Runtime LLM tuning** | ✅ Adjust parameters during execution | ❌ None | Cannot adjust LLM parameters mid-workflow. |
| **Flow lifecycle (start/stop/list)** | ✅ Full CRUD | 🔲 Planned (Temporal + worker) | Designed around Temporal but not exposed via API. |

### Data Ingestion

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **Text document loading** | ✅ Text → chunks → embeddings → graph | ❌ None | No text ingestion pipeline. |
| **Binary document loading** | ✅ PDF/etc. → OCR → embeddings → graph | ❌ None | No document processing pipeline. |
| **Structured data diagnostics** | ✅ Auto-detect format, generate schema | ❌ None | No format detection or schema inference. |
| **Bulk import/export** | ✅ MessagePack streaming (triples + embeddings) | ❌ None | No bulk data import/export for graph data. |
| **Document library management** | ✅ Staging area for processing | ❌ None | No document library concept. |

### Visualization & UI

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **3D graph visualization** | ✅ Built-in GraphViz | ✅ D3.js cosmic graph (075, 020, 136) | Both have graph viz; different technologies. Our D3 implementation is more mature for our needs. |
| **Schema/ontology editors** | ✅ Workbench UI | 🔲 Capability schema editor (136, @xyflow) | We have a capability schema node editor in development. |
| **Vector search UI** | ✅ Search through knowledge bases | ❌ None | No semantic search interface. |
| **Relationship analysis UI** | ✅ Deep relationship exploration | ⚠️ Graph hover/selection in cosmic graph | Limited to visualization; no deep analysis tools. |
| **Prompt editor UI** | ✅ Runtime prompt management | ❌ None | No prompt editing interface. |

### Infrastructure & Observability

| Capability | TrustGraph | Nostra/Cortex | Gap Analysis |
|:-----------|:-----------|:--------------|:-------------|
| **API gateway** | ✅ REST + WebSocket | ✅ `cortex-gateway` (REST + WS) | Comparable architecture. |
| **WebSocket multiplexing** | ✅ Single conn, all services | ⚠️ WebSocket per route | Not multiplexed; separate WS endpoints. |
| **Prometheus metrics** | ✅ LLM latency, errors, throughput | ⚠️ Basic canister metrics | No LLM-specific observability. |
| **Grafana dashboards** | ✅ Built-in with 12+ metrics | ❌ None | No dashboarding. |
| **Message bus** | ✅ Apache Pulsar (pub/sub) | ⚠️ `GlobalEvent` log (append-only) | Events but no pub/sub message fabric. |
| **TypeScript client library** | ✅ `@trustgraph/client` + React state/provider | ⚠️ Custom hooks in cortex-web | No standalone client library; tightly coupled. |
| **Docker/K8s deployment** | ✅ Full containerized stack | 🚫 ICP canister deployment | Different deployment model by design. |

---

## 2. Gap Priority Matrix

Gaps ordered by impact on Nostra/Cortex graph capability maturity:

| Priority | Gap | Impact | Effort | Initiative |
|:---------|:----|:-------|:-------|:-----------|
| 🔴 **P0** | No ontology layer | Blocks knowledge modeling, OntologyRAG, and domain-specific graph validation | Medium | M22 |
| 🔴 **P0** | No knowledge bundle format | Blocks portable, versioned knowledge packaging and context-as-code workflows | Medium | M23 |
| 🟠 **P1** | Schema registry unorganized | Technical debt; schemas scattered; no discovery or validation tooling | Low | M21 |
| 🟠 **P1** | No triple query interface | Agents cannot programmatically traverse Contribution/Relation graph | Medium | M24 |
| 🟡 **P2** | No vector storage/indexing | Embedding *generation* exists (042); missing vector DB for indexed similarity search | Medium | Extend 042 |
| 🟡 **P2** | No Graph RAG | Agents lack graph-grounded answers; limited to direct LLM calls | High | Requires P0 + P2 |
| 🟡 **P2** | No data ingestion pipeline | Cannot auto-extract knowledge from documents | High | Future initiative |
| 🔵 **P3** | No prompt management | Prompts hardcoded; no runtime editing or versioning | Low | Future enhancement |
| 🔵 **P3** | No NLP→query pipeline | Non-technical users can't query graph in natural language | Medium | Requires P1 + P2 |
| 🔵 **P3** | No LLM observability | No latency, error rate, token throughput dashboards | Medium | Future enhancement |

---

## 3. Deeper Insights

### 3.1 The Context Core as Architectural North Star

TrustGraph's Context Core is not just a feature — it's an **architectural organizing principle**. Everything in TrustGraph (schemas, ontologies, embeddings, provenance, retrieval policies) exists to *produce, version, and serve Context Cores*. This pattern maps cleanly to a Nostra concept:

> A **Space Knowledge Bundle** = ontology + graph data + embeddings + event provenance + traversal policies, versioned and exportable as a first-class Contribution.

This bundle becomes the unit of knowledge that agents consume, Spaces export, and governance tracks.

### 3.2 Schema vs Ontology Is a Data Lifecycle Question

TrustGraph's distinction is not academic — it's operational:
- **Schemas** define what *structured data looks like* (table columns, JSON shape, GraphQL types)
- **Ontologies** define what *knowledge means* (entity types, relationship semantics, inference rules)

In Nostra terms:
- **Schemas** govern `Contribution` ingestion validation ("does this artifact have the right fields?")
- **Ontologies** govern `Relation` semantic integrity ("is 'implements' a valid relationship between Capability and Pattern?")

We currently conflate these. The TrustGraph model suggests splitting them cleanly.

### 3.3 Graph RAG Changes Agent Architecture

TrustGraph's Graph RAG pipeline (embed query → find entities → extract subgraph → pass to LLM) fundamentally changes how agents interact with knowledge:
- **Without Graph RAG**: Agent gets flat context window. Answers based on text similarity only.
- **With Graph RAG**: Agent gets structured relationships. Can answer "how are X and Y connected?" with multi-hop graph traversal.

This is the difference between a chatbot and a knowledge-aware reasoning system.

### 3.4 Named Graphs Enable Provenance Scoping

TrustGraph's triple queries support **named graphs** — scoping queries to specific provenance sources:
- `urn:graph:source` — Original source triples
- `urn:graph:retrieval` — Retrieval-augmented triples
- Null — Query across all graphs

This maps to our `EventSource` variants (`#System`, `#Actor`, `#Agent`) and suggests a graph-scoping mechanism where queries can filter by provenance origin.

---

## 4. Additional Reference Logging Guidance

The following TrustGraph resources should be captured from the cloned repo for long-term reference value:

### Must Capture (API & Architecture)

| Resource | Location in Repo | Why |
|----------|-----------------|-----|
| **REST API OpenAPI spec** | Search for `openapi.yaml` or `swagger` in repo | Full API contract for all 18+ endpoints |
| **Python client library source** | `trustgraph-base/` or similar | Study the client abstraction layer |
| **Flow blueprint schema** | Config service definitions | Blueprint/instance data contract |
| **Prompt template format** | Prompt management schemas | Template variable substitution patterns |
| **MessagePack import/export protocol** | Import/export handlers | Bulk data streaming protocol spec |
| **RDF triple schema** | Triple storage model | S-P-O data model implementation |
| **Ontology definition format** | Ontology service | How ontologies are structured internally |
| **Schema definition format** | Schema service | How schemas are structured internally |

### Should Capture (Patterns & UX)

| Resource | Location in Repo | Why |
|----------|-----------------|-----|
| **Workbench UI source** | `trustgraph-workbench/` or `ui/` | Schema/ontology editor UX patterns |
| **3D graph visualizer** | Workbench GraphViz component | Alternative 3D graph rendering approach |
| **Agent reasoning chain format** | Agent service internals | Thought/action/observation structure |
| **Flow engine lifecycle code** | Flow management service | Blueprint→instance state machine |
| **Graph RAG retrieval logic** | GraphRAG service | Entity selection → subgraph extraction algorithm |
| **NLP→GraphQL transformation** | NLP query service | NL→structured query with confidence scoring |
| **Config service type system** | Config management | Key-value config with typed namespaces |

### Discovery Commands

```bash
# Find API specs
fd -e yaml -e json "openapi\|swagger\|api" research/reference/topics/data-knowledge/trustgraph/

# Find schema/ontology definitions
fd "schema\|ontology" research/reference/topics/data-knowledge/trustgraph/ --type f

# Find Python client/SDK
fd "client\|sdk" research/reference/topics/data-knowledge/trustgraph/ --type d

# Find Workbench UI
fd "workbench\|ui\|frontend" research/reference/topics/data-knowledge/trustgraph/ --type d --max-depth 2

# Find Docker/deployment configs
fd "docker-compose\|Dockerfile\|k8s\|kubernetes" research/reference/topics/data-knowledge/trustgraph/

# Find tests (for contract understanding)
fd "test" research/reference/topics/data-knowledge/trustgraph/ --type d --max-depth 2
```
