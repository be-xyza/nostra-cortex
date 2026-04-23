---
id: 094
name: institution-modeling
title: 'Decisions: Institution Modeling'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-01'
updated: '2026-02-02'
---

# Decisions: Institution Modeling

> **Initiative**: 094-institution-modeling
> **Status**: Draft | **Created**: 2026-02-01

---

## DEC-094-001: Institution as Contribution Type

**Status**: Accepted
**Date**: 2026-02-01

### Context

Nostra needs explicit institutional semantics to support organizational cooperation beyond ad-hoc Space configurations. The question is how to represent institutions in the system.

### Options Considered

1. **Institution as Space Subtype** — Create a specialized Space template for institutions
2. **Institution as Contribution Type** — Model institutions as first-class contributions within Spaces
3. **Institution as External Entity** — Keep institutions as reference entities only (current state)

### Decision

**Option 2: Institution as Contribution Type**

### Rationale

- Leverages existing Contribution infrastructure (versioning, lineage, forking)
- Allows multiple institutions to exist within or across Spaces
- Maintains separation between operational container (Space) and organizational identity (Institution)
- Enables institutional evolution via standard contribution lifecycle

### Consequences

- Institution inherits all Contribution metadata (version, contributors, timestamps)
- Institutions can be forked, merged, and have lineage chains
- Space configuration remains separate from institutional identity

---

## DEC-094-002: Descriptive vs Enforced Lifecycle

**Status**: Accepted
**Date**: 2026-02-01

### Context

Institutions evolve through phases (emergent → formalized → dormant). Should these transitions be enforced by the system or merely descriptive?

### Options Considered

1. **Enforced Lifecycle** — State machine with required transitions and validation
2. **Descriptive Lifecycle** — Informational field that can be updated freely
3. **Hybrid** — Core system descriptive, governance modules can add enforcement

### Decision

**Option 3: Hybrid approach** — Core lifecycle is descriptive; enforcement is a governance module concern.

### Rationale

- Avoids premature rigidity in v0.1
- Allows different institutions to have different transition rules
- Governance modules can add state machine semantics when needed
- Preserves "naming, not controlling" philosophy

### Consequences

- `LifecyclePhase` updates require no validation in core system
- Governance modules can subscribe to phase changes and enforce rules
- Historical phase changes are tracked via contribution versioning

---

## DEC-094-003: Stewards vs Roles

**Status**: Accepted
**Date**: 2026-02-01

### Context

Should Institutions have their own role system separate from Spaces?

### Options Considered

1. **Dedicated Institution Roles** — Owner, Governor, Council Member, etc.
2. **Steward Field Only** — Simple list of principals without role hierarchy
3. **Defer to Space Roles** — Rely entirely on Space membership and roles

### Decision

**Option 2: Steward Field Only** with explicit non-permanence semantics.

### Rationale

- Avoids creating a parallel permission system
- "Steward" is semantically richer than "owner" for institutional contexts
- Role complexity belongs in governance modules, not core schema
- Space roles remain the source of truth for access control

### Consequences

- Stewards are informational, not authoritative for access control
- Governance modules can implement councils, committees, delegates
- No conflict between Space permissions and Institution roles

---

## DEC-094-004: Charter References Not Embedding

**Status**: Accepted
**Date**: 2026-02-01

### Context

Institutions need constitutions/charters. Should these be embedded in the Institution object or referenced?

### Decision

**References only** — `charter_refs: Vec<ContributionId>` pointing to Essays, Proposals, or dedicated Constitution contributions.

### Rationale

- Charters are themselves contributions that should fork independently
- Embedding would create data duplication and version drift
- Referenced charters can be shared across institutions (inheritance)
- Aligns with Spaces Constitution §18 (every Space references a Constitution)

### Consequences

- Charter changes are tracked via their own version history
- Multiple institutions can share the same charter (federation)
- Charter forks create constitutional divergence with lineage

---

## DEC-094-005: No Governance Logic in v0.1

**Status**: Accepted
**Date**: 2026-02-01

### Context

Should v0.1 include any governance primitives (voting, quorum, delegation)?

### Decision

**Explicitly no.** Governance modules are deferred to Phase 2.

### Rationale

- Follows phased approach: name first, control later
- Prevents premature lock-in to specific governance models
- Allows different institutions to experiment with different approaches
- Reduces implementation complexity for initial release

### Consequences

- Phase 1 is purely semantic/descriptive
- Governance behavior must be handled via Proposals/Decisions or external tooling
- Phase 2 will introduce modular, pluggable governance primitives

---

## DEC-094-006: Modular Governance Implementation

**Status**: Accepted
**Date**: 2026-02-02

### Context

Following the establishment of the Institution schema, we needed a concrete implementation for governance logic (Phase 2).

### Decision

**Implement Standard Governance Modules** (`voting.mo`, `governance.mo`) where:
1.  **Strategies are Pluggable**: `Institution` entities reference a `GovernanceStrategy` (e.g., `#owner_dictator`, `#voting`).
2.  **Voting is Abstracted**: `VotingSystem` interface allows different kernels (Simple Majority, Token Weighted).
3.  **Lifecycle is Governance-Driven**: Phase transitions (`#lifecycle_change`) are side-effects of successful proposals.

### Rationale

-   Provides a "batteries included" default for new institutions.
-   Decouples the *mechanism* (voting logic) from the *entity* (Institution).
-   Allows for future expansion (e.g., quadratic voting) without schema migrations.
