# Awesome Ontology Qualification Matrix Resolution

**Source**: [ozekik/awesome-ontology](https://github.com/ozekik/awesome-ontology)
**Resolved**: 2026-04-02
**Mode**: `recommendation_only`

## Normalized Extraction Rule

Use this rule before any future exhaustive pass:

1. Count one item per leaf bullet in the upstream README.
2. Keep combined bullets combined unless the upstream README gives them separate leaf bullets.
3. Count nested bullets only when they are clearly named leaf items, not merely syntax examples or explanatory child bullets.
4. Mark see-also links and explainer links as `Skip (Doc/Link)`.
5. Do not split a source bullet into multiple matrix rows unless the source itself does.

Under this rule, the earlier 168-row plan is not validated. It should be treated as a draft normalization, not as proof of exhaustive parity.

## Validation Corrections

These items must be corrected before any exhaustive or parallel intake run:

| Item | Correction |
|---|---|
| `VocBench and Semantic Turkey` | Do not collapse to `VocBench` only. |
| `KIF` | Missing from the prior matrix. |
| `KL-ONE` | Missing from the prior matrix. |
| `WebVOWL` | Reclassify as `Intake (UI Reference)` instead of dependency. |
| `SciGraph` | Reclassify as `Intake (Datastore Comparator)` instead of dependency. |
| `JSON-LD` | Reclassify as `Intake (Reference Format)` instead of dependency. |
| `[protege-user...` | Repair malformed candidate label before any automation. |

## Dispatch-Ready Shortlist

This shortlist is safe to use for next-step intake work because each item has a clear role in the Nostra/Cortex graph stack.

| Category | Candidate | Disposition | Why it survives qualification |
|---|---|---|---|
| Ontology Editors | Protégé | Intake (UX Reference) | Dominant ontology editor benchmark for authoring workflows, inspection affordances, and plugin boundaries. |
| Ontology Editors | VocBench and Semantic Turkey | Intake (UX Reference) | Useful benchmark for collaborative vocabulary editing and governance-heavy ontology workflows. |
| Ontology Utilities | WebVOWL | Intake (UI Reference) | Web-based ontology visualization reference for graph inspection and rendering patterns. |
| Datastore | SciGraph | Intake (Datastore Comparator) | Useful comparator for ontology-backed graph storage and query boundaries; not a Wasm dependency. |
| Languages | Common Logic (CL) | Intake (Reference Pattern) | Useful as a formal semantics reference when evaluating expressivity boundaries. |
| Languages | OWL 2 Web Ontology Language | Intake (Reference Pattern) | Central semantic-web modeling standard for ontology compatibility decisions. |
| Languages | RDF (Resource Description Framework) 1.1 | Intake (Reference Pattern) | Foundational graph data model and interoperability contract. |
| Languages | JSON-LD | Intake (Reference Format) | Practical serialization target for a lightweight ontology layer and exchange format. |
| Languages | Turtle | Intake (Reference Pattern) | Human-readable RDF syntax useful for fixtures, examples, and compatibility testing. |
| Languages | RDF* | Intake (Reference Pattern) | Relevant to statement-level metadata and provenance experiments. |
| Ontologies and Vocabularies | BFO | Intake (Reference Pattern) | Upper ontology comparator for rigor around entity and relation categories. |
| Ontologies and Vocabularies | gUFO | Intake (Reference Pattern) | Lightweight upper ontology comparator with stronger conceptual overlap to modeling work. |
| Ontologies and Vocabularies | Common Core Ontologies (CCO) | Intake (Reference Pattern) | Mid-level ontology reference for operational modeling patterns. |
| Ontologies and Vocabularies | ConceptNet | Intake (Reference Pattern) | Useful comparator for commonsense graph structure and relation vocabulary breadth. |
| Ontologies and Vocabularies | DBpedia | Intake (Reference Pattern) | Useful linked-data reference for public graph modeling and entity typing patterns. |
| Ontologies and Vocabularies | Wikidata | Intake (Reference Pattern) | High-value comparator for entity identity, statements, qualifiers, and provenance-style metadata. |
| Ontologies and Vocabularies | WordNet | Intake (Reference Pattern) | Lexical graph comparator for synonym and concept-link modeling. |
| Ontologies and Vocabularies | Dublin Core Metadata Element Set | Intake (Reference Pattern) | Strong metadata baseline for contribution-level descriptive fields. |
| Ontologies and Vocabularies | DCMI Metadata Terms | Intake (Reference Pattern) | Extends Dublin Core with practical metadata terms relevant to contribution manifests. |
| Ontologies and Vocabularies | Schema.org Schemas | Intake (Reference Pattern) | Public-web vocabulary comparator for interoperable entity markup. |
| Ontologies and Vocabularies | SKOS | Intake (Reference Pattern) | Useful for controlled vocabularies, taxonomy layers, and concept schemes. |
| Logics | Description Logics (DLs) | Intake (Reference Pattern) | Important for understanding expressivity and reasoner complexity tradeoffs. |
| Logics | F-Logic | Intake (Reference Pattern) | Comparator for rule and object-style knowledge representation tradeoffs. |
| Reasoners | OWL-RL | Intake (Reference Pattern) | Practical reasoning profile for bounded inference experiments. |
| Querying | SPARQL 1.1 | Intake (Reference Pattern) | Baseline graph query language comparator. |
| Querying | SPARQL-DL | Intake (Reference Pattern) | Useful comparator at the ontology-query boundary. |
| Querying | SPARQL-OWL algorithm | Intake (Reference Pattern) | Relevant for query planning ideas around ontology-backed data. |
| Querying | SPARQL* | Intake (Reference Pattern) | Relevant to quoted triples and statement metadata. |
| Rule and Schema Definition | LinkML | Intake (Reference Pattern) | Strong schema and validation comparator for structured knowledge layers. |
| Rule and Schema Definition | SHACL | Intake (Reference Pattern) | High-value constraint language candidate for validation boundaries. |
| Rule and Schema Definition | ShEx | Intake (Reference Pattern) | Useful alternative schema and validation comparator to SHACL. |
| Programming | Horned OWL | Intake (Implementation Comparator) | Rust-native OWL library and reference implementation candidate for off-canister tooling. |
| Programming | Owlish | Intake (Dependency Candidate) | Rust OWL 2 library with Wasm support, directly aligned with greenfield Rust/Wasm constraints. |

## Hold or Reject Summary

These categories should not be batch-intaked now:

- Java-only editor, utility, and reasoner stacks should remain `Rejected` unless we need a specific UX or semantics comparator.
- Python utilities should remain `Rejected` as runtime dependencies, though some can stay available as reference implementations.
- Community links, mailing lists, and explainer links should stay `Skip (Doc/Link)`.
- Domain-specific ontologies should stay out of scope unless an active initiative explicitly needs them.

## Dispatch Gate

Parallel intake is reasonable only for the shortlist above, and only after:

1. We decide whether to mirror `awesome-ontology` locally or keep this as analysis-only.
2. We turn each shortlisted item into its own intake ticket or analysis slice.
3. We avoid reusing the unvalidated 168-row matrix as an automation source.
