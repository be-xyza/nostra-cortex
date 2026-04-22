# Semantic Primitives Standard

**Status**: Active
**Owner**: Constitutional Steward
**Scope**: Naming and lifecycle governance for high-signal semantic primitives across Nostra and Cortex.

## Purpose

This standard keeps user-facing and system-facing terms principle-aligned,
layer-aligned, and future-safe.

It exists to prevent:
- accidental promotion of local experimental vocabulary into canonical meaning
- overloaded terms that confuse environment, maturity, and authority
- short-term labels that block future standards with stronger user expectations

## Naming Principles

1. **Principle First**
   A term should name the primary thing it means, not the convenience metaphor
   currently attached to it.

2. **Layer Aligned**
   Terms must match the Nostra/Cortex boundary:
   - Nostra defines what exists.
   - Cortex defines how work runs.

3. **Expectation Safe**
   If a reasonable user would strongly expect a different meaning from a term,
   do not repurpose that term for a narrower experiment.

4. **Definition Plus Non-Definition**
   Every governed primitive must record what it is and what it is not.

5. **Future Reservation**
   Broad, high-expectation terms may be reserved for later canonical use rather
   than consumed early by local experiments.

6. **Promotion by Explicit Governance**
   A term is not canonical because it appears in code. It becomes canonical only
   when its meaning, owner, and governing reference are recorded.

## Two-Axis Model

Semantic primitives use two independent axes.

### `surface_scope`

This answers where the term lives and for whom.

- `labs`: bounded experimental surfaces intended for evaluation and iteration
- `internal`: implementation-facing or adapter-facing system terminology
- `user_facing`: stable user-visible product or platform terminology
- `developer_only`: structural repository and operator vocabulary that must not
  leak into user-facing surfaces

### `semantic_status`

This answers maturity and governance state.

- `experimental`: local or provisional usage; not yet a shared commitment
- `proposed`: reserved or intended for broader use, but not yet canonical
- `canonical`: approved shared meaning
- `deprecated`: retained only for migration, history, or explicit doctrine

## Scope vs Maturity Rule

`labs` is a scope marker, not a maturity marker.

This means:
- a Labs surface may host `experimental` or `proposed` terms
- a canonical term may appear inside Labs
- `labs` must not be used as a synonym for draftness, publication status, or
  legitimacy

## Reserved-Term Policy

Use `reserved_for` when a term should remain available for a future canonical
concept with stronger user expectations.

Reserved terms:
- may appear in doctrine, audit, registry, or historical decision records
- must not be promoted as canonical for a conflicting local use
- should define a replacement term for current local experiments when possible

Example:
- `gallery` may be reserved for a future user-facing browsing or collection
  surface, while a current experimental comparison surface uses `catalogue`

## Deprecation Policy

Deprecated terms:
- must remain documented until migration is complete
- must not be introduced into new canonical docs or contracts
- should declare a replacement term when a stable replacement exists

## Promotion Rule

A term may move to `canonical` only when:
- `definition` and `non_definition` are recorded
- `owner` is assigned
- `reserved_for` conflicts are resolved
- `decision_ref` points to the governing source for the promotion

## Validator Expectations

The semantic primitive validator should ensure:
- registry entries use valid scope/status values
- reserved terms are not reused in canonical docs for conflicting meanings
- deprecated terms do not re-enter canonical docs
- required high-signal terms remain registered

## Current Decision Default

For the current layout experiment stage:
- `gallery` remains reserved for a future user-facing collection concept
- the current Cortex layout comparison surface is treated as an experimental
  `catalogue` concept
- no route or label renames are required in this phase
