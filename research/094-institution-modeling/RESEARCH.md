---
id: 094
name: institution-modeling
title: 'Research Initiative 094: Institutional Modeling & Evolution'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-01'
updated: '2026-02-01'
---

# Research Initiative 094: Institutional Modeling & Evolution

> **Status**: Active | **Phase**: Research & Design
> **Created**: 2026-02-01 | **Last Updated**: 2026-02-01

## Executive Summary

This research investigates how Nostra can support organizational cooperation, institutional legitimacy, and long-term governance evolution without introducing premature rigidity or centralized authority. The aim is not to build a DAO framework, but to establish a **minimal, composable institutional substrate** that allows any collective structure—formal or informal—to emerge, evolve, fork, and dissolve over time.

> [!IMPORTANT]
> **Design Philosophy**: Institutions are named, not controlled. We add semantics, not enforcement. This research explicitly defers governance modules, voting systems, and treasury primitives to later phases.

---

## 1. Problem Statement

### Current State: Strong Coordination, Implicit Institutions

Nostra already provides robust primitives for organizational cooperation:

| Capability | Current Status | Location |
|------------|----------------|----------|
| **Organizational Containers** | ✅ Spaces with membership, roles, visibility | [007-spaces](../007-nostra-spaces-concept/REQUIREMENTS.md) |
| **Unified Contribution Model** | ✅ Proposals, Reviews, Decisions, Projects, Initiatives | [008-contribution-types](../008-nostra-contribution-types/PLAN.md) |
| **Governance Hooks** | ✅ Proposals → Decisions flow | [spec.md](/nostra/spec.md) |
| **Temporal Workflows** | ✅ Long-running, multi-year processes | [047-temporal](../047-temporal-architecture/RESEARCH.md) |
| **Forking & Lineage** | ✅ Universal forking with history preservation | [spec.md](/nostra/spec.md) |

**However, institutions are implicit rather than first-class:**

1. **No explicit Institution schema** — Organizations exist only as reference entities, not operational actors
2. **Governance is configurable, not programmable** — You can enable/disable contribution types, but cannot define quorum, delegation, or role rotation
3. **Roles are static** — Owner/Member/Viewer, no councils, committees, delegates, or offices
4. **No institutional state machine** — Institutions evolve (startup → nonprofit → DAO → foundation) but that evolution is not modeled

### Why This Matters

Without explicit institutional semantics:
- Authority is assumed, not auditable
- Organizational evolution happens through ad-hoc workarounds
- Cross-institutional relationships lack formal representation
- AI agents cannot reason about institutional context

---

## 2. Core Research Questions

### 2.1 Institutional Representation
- How should institutions be modeled as first-class Contributions without embedding governance logic?
- What information must be explicit vs inferred from graph relationships?
- How do we distinguish an Institution from a Space?

### 2.2 Evolution & Forking
- How can institutions transition across phases (emergent → formal → dormant) without enforcement?
- How should institutional forks preserve legitimacy and historical continuity?
- What constitutes a valid institutional lineage chain?

### 2.3 Separation of Concerns
- How do we keep institutions, governance mechanisms, and execution workflows cleanly decoupled?
- What must remain constitutional vs configurable?
- Where do governance modules attach (Proposals? Decisions? Institutions? Spaces?)

### 2.4 Compatibility Validation
Can this model represent:
- DAOs
- Nonprofits / Foundations
- Research labs / Academic institutions
- Open-source projects (Mozilla, Apache, Linux)
- Informal collectives
- Nested institutions (councils within foundations)
- Multi-jurisdictional entities

### 2.5 Future Hooks
- Where should governance modules attach?
- How might Temporal workflows formalize institutional processes (elections, audits, charter amendments)?
- What integration is needed with the economic substrate (093)?

---

## 3. Architectural Context

### 3.1 Relationship to Existing Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Constitutional Layer                        │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │ Spaces           │  │ Contribution     │  │ Governance     │ │
│  │ Constitution     │  │ Types (008)      │  │ Framework      │ │
│  │ (034)            │  │                  │  │ (034)          │ │
│  └────────┬─────────┘  └────────┬─────────┘  └───────┬────────┘ │
└───────────┼──────────────────────┼────────────────────┼─────────┘
            │                      │                    │
            ▼                      ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Institutional Layer (This Research)         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │ Institution      │  │ Lifecycle        │  │ Relationship   │ │
│  │ Contribution     │  │ Phases           │  │ Model          │ │
│  │                  │  │                  │  │                │ │
│  └────────┬─────────┘  └────────┬─────────┘  └───────┬────────┘ │
└───────────┼──────────────────────┼────────────────────┼─────────┘
            │                      │                    │
            ▼                      ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Governance Module Layer (Future)            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │ Voting           │  │ Delegation       │  │ Treasury       │ │
│  │ Module           │  │ Module           │  │ Module (093)   │ │
│  └──────────────────┘  └──────────────────┘  └────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Why Spaces Are Proto-Institutions

From [NOSTRA_SPACES_CONSTITUTION](../034-nostra-labs/NOSTRA_SPACES_CONSTITUTION.md):

> "A Space is a bounded epistemic and operational context with its own culture, its own norms, its own authority structure, its own memory."

Spaces already:
- Define sovereignty (§2)
- Establish boundaries (§4)
- Scope authority (§5)
- Support forking (§9)
- Carry temporal identity (§12)
- Reference constitutions (§18)

**Insight**: The `Institution` type enriches Spaces with explicit naming, lineage, and lifecycle semantics—it does not replace them.

### 3.3 Integration with Temporal Architecture (047)

Institutions benefit from durable execution patterns:

| Institutional Process | Temporal Pattern |
|----------------------|------------------|
| Annual review cycles | Workflow-as-Scheduler |
| Leadership elections | Human-in-the-Loop signals |
| Charter amendments | Staged approval with rollback |
| Budget cycles | Completion callbacks |
| Conflict resolution | Provisional state + lifecycle awaiting |

---

## 4. Proposed Design (Draft v0.1)

### 4.1 Institution Contribution Type

**Design Goal**: Make institutions explicit and inspectable without hard-coding governance or freezing evolution.

```rust
/// Classification
/// - Category: Deliberative / Structural
/// - Phase: Constitutional → Operational → Dormant (non-linear)
/// - Forkable: Yes
/// - Mergeable: Yes (via Proposal → Decision)

struct Institution {
    id: Text,
    space_id: Text,

    // Identity
    title: Text,
    summary: Text,
    intent: Text,                    // Why this institution exists
    scope: Text,                     // What it governs / coordinates

    // Lifecycle
    lifecycle_phase: LifecyclePhase,

    // References
    charter_refs: Vec<ContributionId>,    // Essays, Constitutions, Specs
    parent_institution_id: Option<Text>,
    affiliated_spaces: Vec<SpaceId>,

    // Stewardship (non-exclusive, non-permanent)
    stewards: Vec<Principal>,

    // Standard Contribution Fields
    confidence: Float,               // 0.0–1.0
    version: Nat,
    previous_version_id: Option<Text>,
    created_at: Time,
    updated_at: Time,
}

enum LifecyclePhase {
    Emergent,      // Informal, experimental
    Provisional,   // Structuring, gaining legitimacy
    Formalized,    // Chartered, recognized
    Operational,   // Active governance
    Dormant,       // Paused but preservable
    Archived,      // Historical record
}
```

### 4.2 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **No hard roles** | "Steward" ≠ owner ≠ authority. Authority emerges via Decisions, not fields. |
| **Charter is referenced, not embedded** | Constitutions remain forkable documents. Institution points to them, never absorbs them. |
| **Lifecycle is descriptive, not enforced** | Phase changes are informational. Governance modules may react later, but core stays neutral. |
| **Stewards are principals, not roles** | Avoids creating a parallel permission system. Space roles remain authoritative. |

### 4.3 Graph Relationships

```
Institution ── governs ──────────> Space
Institution ── derives_from ─────> Institution
Institution ── charters ─────────> Proposal / Constitution / Essay
Institution ── operates_through ──> Project / Initiative
Institution ── forked_into ──────> Institution
Institution ── affiliated_with ───> Space (lateral)
```

### 4.4 What an Institution IS (and is NOT)

| IS | IS NOT |
|----|--------|
| A named coordination construct | A DAO |
| A historical lineage anchor | A legal entity |
| A semantic container for authority claims | A permissions engine |
| A bridge between ideas and execution | A role hierarchy |

---

## 5. Constraints & Non-Goals

### Explicit Non-Goals for v0.1

- ❌ Fixed role hierarchies
- ❌ Mandatory voting systems
- ❌ Financial / treasury logic
- ❌ Legal entity assumptions
- ❌ Collapsing Institutions into Spaces
- ❌ Assuming on-chain governance

### Future Work (Explicitly Deferred)

| Module | Intent | Phase |
|--------|--------|-------|
| Voting Module | Structured decision-making | 2 |
| Quorum Module | Threshold requirements | 2 |
| Delegation Module | Representative governance | 2 |
| Role Rotation Module | Term limits, elections | 2 |
| Treasury Module | Budget cycles, escrow | 3 (see [093](../093-economic-substrate/)) |
| Inter-institution Treaties | Cross-org agreements | 3 |
| Constitutional Automation | Temporal workflow integration | 3 |

---

## 6. Validation Strategy

### 6.1 Real-World Institution Stress Tests

To validate the model, we will map against:

| Institution Type | Example | Key Patterns to Capture |
|------------------|---------|------------------------|
| **Global NGO** | United Nations | Nested bodies, delegated authority, charter-based |
| **Open Source Foundation** | Apache, Linux, Mozilla | Meritocratic, project-based, emergent leadership |
| **DAO** | MakerDAO, Gitcoin | Token-weighted, on-chain proposals, treasury |
| **Academic** | Research consortium | Peer review, grant cycles, publication governance |
| **Cooperative** | Mondragon | Member-owned, democratic, economic integration |
| **Informal Collective** | Hacker spaces, art collectives | Emergent, low-governance, high-autonomy |

### 6.2 Success Criteria

A successful design allows:

- [ ] Institutions to be **named and reasoned about**
- [ ] Authority to be **auditable, not assumed**
- [ ] Evolution to occur via **forks and decisions**
- [ ] Future governance systems to be **added without refactoring core architecture**
- [ ] AI agents to **understand institutional context**
- [ ] Cross-institutional relationships to be **formally represented**

---

## 7. Phased Implementation Path

### Phase 1: Make Institutions Explicit (Low Risk)

> **Scope**: Add semantic layer, no behavioral changes

- [x] Add `Institution` as a new Contribution Type
- [x] Define lifecycle phases (informational only)
- [x] Create graph edges for institutional relationships
- [x] Update schema definitions (Integrated into KIP)

### Phase 2: Governance Modules (Composable)

> **Scope**: Modular governance primitives

- [ ] Voting module (attaches to Proposals)
- [ ] Quorum module (configurable thresholds)
- [ ] Delegation module (representative voting)
- [ ] Role rotation module (term-based leadership)

### Phase 3: Institutional State Machines (Powerful)

> **Scope**: Temporal workflow integration

- [ ] Model institutional processes as workflows:
  - Annual review
  - Leadership election
  - Budget cycle
  - Charter amendment process
- [ ] Integration with [047-temporal-architecture](../047-temporal-architecture/)
- [ ] Integration with [093-economic-substrate](../093-economic-substrate/)

---

## 8. Cross-Research Integration

| Initiative | Relationship |
|------------|--------------|
| [007-nostra-spaces-concept](../007-nostra-spaces-concept/) | Spaces are proto-institutions; Institution inherits space context |
| [008-nostra-contribution-types](../008-nostra-contribution-types/) | Institution becomes a new Contribution Type |
| [013-nostra-workflow-engine](../013-nostra-workflow-engine/) | Governance processes modeled as workflows |
| [034-nostra-labs](../034-nostra-labs/) | Constitutional frameworks govern institutional behavior |
| [047-temporal-architecture](../047-temporal-architecture/) | Durable execution for institutional processes |
| [093-economic-substrate](../093-economic-substrate/) | Treasury module integration in Phase 3 |

---

## 9. Next Steps (Immediate)

1. **Review & Approval**: Validate this research framework with stakeholders
2. **Stress Test**: Map 3-5 real institutions against the proposed schema
3. **Schema Refinement**: Iterate on Institution Contribution Type based on stress tests
4. **Integration Design**: Define exact touchpoints with Spaces and Workflows
5. **Phase 1 Implementation**: Add Institution type to contribution taxonomy

---

## References

- [NOSTRA_SPACES_CONSTITUTION](../034-nostra-labs/NOSTRA_SPACES_CONSTITUTION.md)
- [NOSTRA_GOVERNANCE_ESCALATION_FRAMEWORK](../034-nostra-labs/NOSTRA_GOVERNANCE_ESCALATION_FRAMEWORK.md)
- [008-nostra-contribution-types/PLAN.md](../008-nostra-contribution-types/PLAN.md)
- [007-nostra-spaces-concept/REQUIREMENTS.md](../007-nostra-spaces-concept/REQUIREMENTS.md)
- [047-temporal-architecture/RESEARCH.md](../047-temporal-architecture/RESEARCH.md)
- [spec.md](/nostra/spec.md) — Nostra v2 Specification
