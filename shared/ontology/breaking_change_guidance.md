# Core Ontology Breaking Change Guidance

Date: 2026-04-03
Status: Draft
Authority Mode: recommendation_only

## Purpose

This note defines what counts as a breaking change for the Nostra core ontology in
`/Users/xaoj/ICP/shared/ontology/core_ontology_v1.json`.

The goal is to keep the ontology additive by default and make contract risk explicit
before changes affect bundle, query, or interoperability surfaces.

## Breaking Changes

Treat any of the following as a breaking ontology change:

1. Removing a core class, relation, or property.
2. Renaming a published class, relation, or property identifier.
3. Changing the meaning of a core term in a way that would invalidate prior data.
4. Changing a relation source or target so previously valid links become invalid.
5. Narrowing a property target set or value type in a way that rejects prior uses.
6. Changing provenance scope semantics away from `system`, `actor`, or `agent`.
7. Replacing additive extension rules with rules that permit core-term redefinition.
8. Introducing required semantics that older bundles or query clients cannot satisfy.
9. Changing the current core relation set (`contains`, `relates_to`, `has_capability`, `scoped_by`) without steward review and migration guidance.

## Non-Breaking Changes

These changes are additive by default:

1. Adding a new namespace-prefixed Space-local class.
2. Adding a new namespace-prefixed Space-local relation.
3. Adding a new namespace-prefixed Space-local property.
4. Adding clarifying notes or descriptions without changing identifiers or meaning.
5. Adding optional metadata that does not invalidate older manifests.

## Governance Rule

Breaking changes require:
- steward review,
- a compatibility note,
- and an explicit migration story for affected bundles, fixtures, or query consumers.

If there is uncertainty, classify the change as breaking until reviewed.
