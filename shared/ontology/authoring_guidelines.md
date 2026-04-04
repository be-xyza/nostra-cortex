# Core Ontology Authoring Guidelines

Date: 2026-04-03
Status: Draft
Authority Mode: recommendation_only

## Purpose

These rules keep the ontology layer small, additive, and subordinate to
`/Users/xaoj/ICP/shared/specs.md`.

## Authoring Rules

1. Core terms define shared platform semantics; Space-local terms must be namespace-prefixed.
2. Use a **class** when introducing a governed subject type.
3. Use a **relation** when introducing a typed edge between graph subjects.
4. Use a **property** only for portable descriptive or contract-level metadata.
5. `extends` may specialize `Contribution` or other namespace-prefixed additive classes, but must not redefine core meaning.
6. `ProvenanceScope` is a closed core set. Space-local ontologies must not extend or replace it.
7. Space-local relations may target core terms, but must not override their descriptions, endpoints, or compatibility semantics.
8. If ontology meaning conflicts with `/Users/xaoj/ICP/shared/specs.md`, the constitutional spec wins.

## Class vs Relation vs Property

- Class: a durable graph subject that can participate in bundle and query semantics.
- Relation: a typed edge that must declare valid source and target classes.
- Property: a portable descriptive field with explicit cardinality and target classes.

## Extension Boundaries

- Namespace prefix required for all additive Space-local terms.
- Additive only by default.
- Core-term redefinition is invalid.
- Breaking changes require steward review, a compatibility note, and a migration story.
