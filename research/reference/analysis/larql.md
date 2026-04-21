# Placement

- Artifact ID: `repo-larql`
- Type: `repository`
- Topic: `cross-topic`
- Placement path: `research/reference/repos/larql/`
- Status: `reviewed`

## Scorecard

| Field | Score | Rationale |
|---|---:|---|
| ecosystem_fit | 2 | `larql` shares an interest in structured knowledge access and typed query surfaces, but its primary substrate is transformer weights rather than Nostra authority data or Cortex runtime state. |
| adapter_value | 3 | The repo offers useful reference patterns for a semantic query or intent adapter, especially around browse, mutate, patch, and remote execution boundaries. |
| component_value | 1 | The vindex, weight extraction, and inference pipeline are too ML-specific to adopt directly into Nostra or Cortex. |
| pattern_value | 5 | Strongest value: Rust-native grammar/executor split, explicit statement families, immutable base plus patch overlays, and CLI/REPL/server separation. |
| ux_value | 3 | The REPL and command taxonomy are clear and teachable, though the UX remains technical and model-oriented rather than governance- or workspace-oriented. |
| future_optionality | 4 | The design could inform a future Nostra/Cortex query or intent DSL, patch proposals, and deterministic agent-facing command surfaces. |
| topic_fit | 3 | It touches `data-knowledge`, but not strongly enough to justify topic placement; the best fit in this checkout is a cross-topic repo reference. |

## Intent

Assess the live `chrishayuk/larql` repository as a reference intake for Nostra/Cortex, while explicitly validating and correcting earlier assumptions. The earlier analysis treated `larql` as a lightweight Python SPARQL parser. The live repository inspected on 2026-04-14 is instead a Rust workspace for querying and editing transformer-model knowledge through `.vindex` artifacts and the Lazarus Query Language (`LQL`).

The goal of this intake is not to recommend adoption of `larql` as a dependency. The goal is to determine which architecture and product patterns are genuinely reusable for Nostra platform authority and Cortex runtime design, and which ideas are domain-specific or misleading.

## Possible Links To Nostra Platform and Cortex Runtime

1. `nostra/spec.md` defines a contribution graph, typed relationships, and durable execution as first-class platform capabilities. `larql` is not operating on the same authority graph, but it does show how a dedicated query language can make a graph-shaped substrate explorable, mutable, and inspectable without collapsing everything into raw APIs.
2. Initiative `015-nostra-open-source-library` explicitly calls for repository ingestion, graph mapping, and a graph explorer for external libraries. `larql` fits that initiative as a reference object for how a technical system can expose browse, mutation, and introspection operations through a compact language surface.
3. Initiative `118-cortex-runtime-extraction` emphasizes strict runtime/adaptor boundaries. `larql`'s split between core crates, LQL engine, CLI, and remote server is a helpful reference for keeping a language/runtime surface distinct from host-specific transport or tooling.
4. Initiative `123-cortex-web-architecture` requires deterministic, governed runtime interactions and graph projections. `larql`'s statement families and immutable base plus patch overlay model are useful analogies for steward-gated actions and reversible edits over authoritative state.
5. Initiative `127-cortex-native-repo-ingestion` frames repo intake as a governed, evidence-backed activity. This intake also matters as a process example: it demonstrates why direct primary-source inspection must precede pattern extraction, especially when package names and repo names collide across ecosystems.

## Initiative Links

- `015` Nostra Open Source Library: relevant for external repository analysis, graph mapping, and queryable library references.
- `097` Nostra/Cortex Alignment: relevant for preserving the Nostra authority versus Cortex runtime boundary in the analysis.
- `118` Cortex Runtime Extraction: relevant for adapter/runtime separation and deterministic boundaries.
- `123` Cortex Web Architecture: relevant for deterministic interaction surfaces, graph projections, and steward-gated actions.
- `127` Cortex Native Repo Ingestion: relevant for governed reference intake and evidence-backed repository analysis.

## Pattern Extraction

### Validated Corrections

1. The live GitHub repository is not a SPARQL parser. `docs/lql-spec.md` explicitly states that LQL "is not SQL" and "is not SPARQL."
2. The live GitHub repository is not Lark-based. The language implementation target is the Rust `larql-lql` crate, and the repo README describes a Rust-native parser/executor/REPL layer.
3. The repo is not primarily Python. The workspace is a Rust 2021 multi-crate project. Python appears as secondary bindings (`larql-python`) and separate knowledge-pipeline assets, not as the core runtime.
4. The earlier "lightweight 7 KB package" characterization does not apply to this repository. That description matches an unrelated PyPI package named `larql`, which likely caused the confusion.

### Patterns We Should Retain

1. Grammar-first interaction. `larql` uses a dedicated language with explicit statement classes for lifecycle, browse, inference, mutation, patches, and introspection. That is a strong reference for any future semantic query or intent DSL in Nostra/Cortex.
2. Parser/executor boundary. The project clearly separates the language surface from the storage/query substrate and from network transport. That aligns well with Cortex adapter discipline.
3. Immutable base plus patch overlay. The base vindex remains readonly while mutations accumulate in reversible patch files. This is a very good analogy for steward-reviewed mutation proposals over authoritative Nostra state.
4. Multi-surface delivery from one contract. The same conceptual operations appear through CLI, REPL, Python bindings, and a remote HTTP/gRPC server. That is a useful model for keeping semantics stable across host surfaces.
5. Deterministic statement categories. The language makes user intent legible: browse versus infer versus mutate versus patch. That categorization could reduce ambiguity in agent-facing execution commands.

### Patterns To Treat Carefully

1. "The model is the database" is a useful metaphor for `larql`, but it should not be imported into Nostra. Nostra's authority surfaces remain contributions, spaces, relations, schemas, and governed history.
2. Layer bands, attention routing, vindexes, and model recompilation are ML-domain concepts, not Nostra/Cortex architecture primitives.
3. `larql` includes Python bindings, but that does not change the local project rule against adding Python agents. The transferable lesson is that a Rust core can expose multiple host bindings when needed.
4. The server surfaces are attractive, but Nostra/Cortex would still need governance, identity, and permission semantics that are outside `larql`'s problem space.

## Adoption Decision

Adopt as a cross-topic reference repository, not as a dependency candidate or direct architectural blueprint.

The highest-value takeaway is the language and mutation model: explicit statements, a parser/executor split, immutable base state, and reversible overlays. The weakest fit is the actual substrate. `larql` is fundamentally about transformer-weight introspection and editing, not platform-governed contributions or workflow execution. We should borrow the shape of the interface, not the model-internals ontology.

The intake should therefore remain recommendation-only. The repo is worth keeping in the reference catalog because it sharpens thinking about how a governed query layer might sit between Nostra authority and Cortex execution, but it should not drive schema or runtime decisions without further prototype evidence.

## Known Risks

1. Name collision risk is high. Future analysis can easily confuse this GitHub repo with the unrelated PyPI package `larql`, which really is a SPARQL/Lark parser.
2. The repo's product claims are ambitious and model-specific. It is easy to over-translate those claims into general knowledge-graph architecture.
3. The presence of Python bindings and research scripts could cause shallow readers to misclassify the repo as Python-led even though the core is Rust-native.
4. If we borrow the query-language framing too literally, we risk designing around model-internal metaphors instead of Nostra's governed contribution model.

## Suggested Next Experiments

1. Prototype a tiny Rust-native query or intent DSL for contribution-graph browse and mutate operations, borrowing `larql`'s statement-family clarity without importing its ML vocabulary.
2. Test immutable patch overlays for steward-reviewed contribution or workflow changes so proposed edits remain reversible until approved.
3. Compare `larql`'s CLI/REPL/server surface split against current A2UI and Cortex runtime actions to define a stable command taxonomy for agents and hosts.
4. Add a lightweight intake checklist item for "package-name collision" so future repo analysis explicitly checks whether GitHub and package-registry artifacts refer to the same project.
