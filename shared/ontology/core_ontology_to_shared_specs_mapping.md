# Core Ontology to Shared Specs Mapping

Date: 2026-04-03
Status: Draft
Authority Mode: recommendation_only

## Purpose

This note maps the core ontology manifest in `/Users/xaoj/ICP/shared/ontology/core_ontology_v1.json`
to the constitutional types and graph-contract sections in `/Users/xaoj/ICP/shared/specs.md`.

The goal is to keep the ontology layer subordinate to the constitutional model:
- `shared/specs.md` remains the semantic source of truth,
- the ontology manifest is a portable graph-contract projection,
- and future interoperability formats such as JSON-LD stay downstream of both.

## Mapping Summary

| Core Ontology Term | Shared Specs Anchor | Relationship |
|---|---|---|
| `Space` | `ResourceRef` naming and Space Knowledge Bundle sections | `Space` is a graph-contract class for the sovereign container already assumed by Nostra platform semantics. |
| `Contribution` | Constitutional graph invariant and `KnowledgeBundle.spaceId` / query facade sections | `Contribution` is the core graph subject for governed knowledge, work, and evidence. |
| `Relation` | Constitutional graph invariant and read-only triple facade section | `Relation` is the ontology-level name for typed graph links over the existing graph substrate. |
| `Capability` | Capability references in `ResourceRef` examples and platform governance surfaces | `Capability` is a graph-contract class for exposed features and declared surfaces. |
| `ProvenanceScope` | `7.1 Provenance Scope` | Direct semantic match. The ontology term is intentionally aligned to the constitutional graph contract. |

## Detailed Mapping

### `Space`

- Ontology role: sovereign graph container for policies, contributions, and knowledge bundles.
- Shared specs relationship:
  - implied by Nostra platform semantics throughout the repo,
  - referenced structurally by `KnowledgeBundle.spaceId`,
  - routed through canonical identifiers using `ResourceRef`.
- Constraint: the ontology class must not redefine Space semantics beyond what the platform already means.

### `Contribution`

- Ontology role: governed unit of knowledge, work, or evidence.
- Shared specs relationship:
  - named directly in the constitutional graph invariant table,
  - used as the practical subject of bundle, relation, and query work.
- Constraint: ontology extensions may specialize `Contribution`, but must not replace its governed-unit meaning.

### `Relation`

- Ontology role: typed link between governed graph subjects.
- Shared specs relationship:
  - named directly in the constitutional graph invariant table,
  - operationalized by the read-only triple facade and graph-query contracts.
- Constraint: relation semantics must remain stable and additive across ontology revisions.

### `Capability`

- Ontology role: declared ability or feature exposed by a Space or contribution surface.
- Shared specs relationship:
  - appears in canonical capability `ResourceRef` examples,
  - ties the ontology layer back to governance, discovery, and execution surfaces.
- Constraint: ontology usage should keep `Capability` descriptive, not execution-authoritative by itself.

### `ProvenanceScope`

- Ontology role: named graph scope used to partition evidence and traversal.
- Shared specs relationship:
  - directly defined in `7.1 Provenance Scope`,
  - aligned with `GlobalEvent` partitions and graph query scoping.
- Constraint: scope names must stay aligned with `System`, `Actor`, and `Agent` constitutional meanings.

## Property Mapping Notes

| Core Property | Shared Specs Relationship |
|---|---|
| `label` | Human-readable descriptor for graph-facing entities; compatible with user-facing rendering and reference displays. |
| `description` | Short explanatory text used across contract artifacts and knowledge bundles. |
| `space_id` | Projects the platform container identity into graph-facing manifests and contributions. |
| `authority_mode` | Mirrors the governance ceiling language used across initiatives, references, and bundle-like artifacts. |

## Bundle and Query Alignment

- `core_ontology_v1.json` is referenced by the `KnowledgeBundle` contract through `ontologyRef`.
- `ProvenanceScope` aligns directly with the named graph scope used by the read-only triple facade.
- The ontology layer should therefore remain descriptive and portable, while bundle and query contracts remain the operational transport surfaces.

## Guardrail

If a future ontology change conflicts with `shared/specs.md`, the constitutional spec wins and the ontology must be revised, not the other way around.

