# Owlish Prototype Note

Date: 2026-04-02
Status: Draft
Authority Mode: recommendation_only

## Purpose

This note defines a narrow experiment for evaluating `Owlish` against the existing Nostra ontology manifest in `shared/ontology/core_ontology_v1.json`.

The goal is not to let an external OWL library define the platform contract. The goal is to test whether `Owlish` is a practical off-canister tool for:
- loading the current ontology manifest,
- projecting it into a constrained JSON-LD form,
- and comparing that projection against future interoperability needs.

## Current Baseline

The current ontology baseline already exists:
- Manifest: `/Users/xaoj/ICP/shared/ontology/core_ontology_v1.json`
- Schema: `/Users/xaoj/ICP/shared/standards/knowledge_graphs/ontology_manifest.schema.json`

The current manifest is intentionally small:
- classes: `Space`, `Contribution`, `Relation`, `Capability`, `ProvenanceScope`
- relations: `contains`, `relates_to`, `has_capability`, `scoped_by`
- properties: `label`, `description`, `space_id`, `authority_mode`

## Prototype Question

Can `Owlish` support a minimal Nostra ontology workflow without forcing the project into full semantic-web complexity?

That means answering three practical questions:
1. Can we map the current JSON manifest into a constrained OWL or JSON-LD representation with little ceremony?
2. Can we round-trip core terms without semantic drift?
3. Does the resulting toolchain remain simpler than the richer `Horned OWL` lane for day-to-day ontology authoring and export?

## Proposed Scope

In scope:
- off-canister Rust tooling only
- fixture loading from `core_ontology_v1.json`
- export to a constrained JSON-LD projection
- comparison of class, relation, and property identifiers
- additive extension experiments only

Out of scope:
- canister embedding
- reasoner integration
- SPARQL execution
- SHACL execution
- full OWL compatibility claims
- automatic migration of existing graph artifacts

## Minimal Mapping Target

The experiment should preserve only the semantics the current manifest already has:
- ontology id and version metadata
- class identifiers and descriptions
- relation identifiers with source and target bindings
- property identifiers, target classes, and value types
- additive extension rules
- compatibility policy fields

If `Owlish` requires materially more ontology structure than this to stay useful, that is evidence against making it the primary tooling lane.

## Success Criteria

The experiment is successful if all of the following are true:
- The current core ontology can be loaded or represented without expanding the term model.
- A constrained JSON-LD export can be produced with stable identifiers.
- The export remains human-reviewable by project contributors.
- A side-by-side comparison with `Horned OWL` shows that `Owlish` is meaningfully lighter for the narrow use case.

## Failure Signals

Treat the prototype as failed or downgraded if:
- the mapping requires broad OWL constructs beyond the current manifest,
- the output becomes harder to review than the source JSON,
- the tool encourages implicit semantic commitments the project has not approved,
- or the comparison with `Horned OWL` shows little ergonomic advantage.

## Current Prototype Artifact

The initial fixture-driven adapter is now represented by:
- generator: `/Users/xaoj/ICP/scripts/generate_core_ontology_jsonld.py`
- generated projection: `/Users/xaoj/ICP/shared/ontology/core_ontology_v1.experimental.jsonld`

## Next Step

Use the generator output as the baseline comparison artifact, then record any lossy or ambiguous mappings explicitly when the `Owlish` implementation lane begins.
