---
id: 008
name: nostra-contribution-types
title: 'Decision Log: Nostra Contribution Types'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-01'
---

# Decision Log: Nostra Contribution Types

Track architectural decisions regarding contribution types.

---

## DEC-001: Expanded Contribution Types adoption
**Date**: 2026-01-15
**Status**: ✅ Decided

**Options Considered**:
1. Stick to the original 10 types
2. Add "Decision", "Poll", and "Bounty" as first-class types
3. Model these as subtypes of "Concept" or "Task"

**Decision**: Option 2 - Add "Decision", "Poll", and "Bounty"

**Rationale**:
- **Decisions**: Explicitly tracking decisions is critical for knowledge management.
- **Polls**: Structured consensus gathering is distinct from open questions.
- **Bounties**: Explicit incentives bridge the gap between "Issue" and "Execution".

**Implications**:
- `ContributionType` variant updated to include `#decision`, `#poll`, `#bounty`
- New UI components for each
- Graph edges defined for these types (Issue -> Decision -> Project)

---

## DEC-002: Blogs modeled as views, not contribution objects
**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Create a `Blog` contribution type
2. Model blogs as curated views/policies over existing contributions
3. Use external blogging platform with Nostra integration

**Decision**: Option 2 - Blogs as views/policies

**Rationale**:
- Avoids content duplication
- Blogs are curation patterns, not knowledge objects
- Enables flexible publication rules (by author, tag, space)
- Example: "Research Blog" = Essays + MediaEssays tagged `#research`

**Implications**:
- Blog configuration stored at Space level
- No new `ContributionType` variant needed
- UI implements blog as filtered/ordered contribution feed

---

## DEC-003: Adopt Contribution Phase dimension
**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Continue adding contribution types for each use case
2. Introduce orthogonal Phase dimension (Exploratory → Archival)
3. Use tags for phase tracking

**Decision**: Option 2 - Phase dimension

**Rationale**:
- Prevents type bloat while adding semantic richness
- Makes authority and commitment explicit
- Enables governance rules based on phase transitions
- Phases: Exploratory, Deliberative, Decisive, Executable, Archival

**Implications**:
- Add `phase` metadata field to base Contribution type
- Default phase assigned based on contribution type
- Phase transitions can trigger workflows

---

---

## DEC-005: Separate Entity Category
**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Model Person/Organization/Library as Contribution Types
2. Model them as distinct Graph Entities (non-contributions)

**Decision**: Option 2 - Distinct Entities

**Rationale**:
- Contributions represent *actions* or *outputs* within the system.
- People, Organizations, and External Libraries are *referents* or *actors*, not contributions.
- Keeps the "Activity Stream" clean of static noun-nodes.

**Implications**:
- New `EntityType` enum: `Person` (non-user), `Organization`, `Library`.
- Graph edges link Contributions to Entities (e.g., `Artifact` -> `depicts` -> `Person`).

---

## DEC-006: Confidence Metadata Standard
**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Confidence only on Ideas
2. Confidence on all Contributions
3. Confidence as separate "Rating" contribution

**Decision**: Option 2 - Metadata on Base Contribution

**Rationale**:
- Plausibility is relevant across many types (Idea, Report, Review, even historical Person).
- Metadata field (float 0-1) enables uniform filtering and "Truthiness" visualization.

**Implications**:
- Add `confidence: Float` to base `Contribution` struct.

---

## DEC-007: Functional & Deliberative Extensions
**Date**: 2026-01-16
**Status**: ✅ Decided

**Decision**: Adopt `Proposal`, `Review`, `Pledge`, `Service`, `Event`, `Report`.

**Rationale**:
- **Proposals**: distinct from Ideas; formal request vs conceptual seed.
- **Reviews**: critical for verification; distinct from Comments.
- **Pledges**: inverse of Bounties; needed for fundraising/family use cases.
- **Services**: monetizable units; distinct from general Projects.
- **Events**: time coordination is a core collaboration primitive.
- **Reports**: structured findings distinct from generic Artifacts.

**Implications**:
- Schema updates for all new types.
- UI components for "Review" (score/pass-fail) and "Event" (calendar).

---

## DEC-008: Institution as Contribution Type
**Date**: 2026-02-01
**Status**: ✅ Decided
**Reference**: [094-institution-modeling](../094-institution-modeling/RESEARCH.md)

**Options Considered**:
1. Institution as Space Subtype (template specialization)
2. Institution as Contribution Type (first-class semantic object)
3. Institution as External Entity only (reference node)

**Decision**: Option 2 - Institution as Contribution Type

**Rationale**:
- Leverages existing Contribution infrastructure (versioning, lineage, forking)
- Allows multiple institutions to exist within or across Spaces
- Maintains separation between operational container (Space) and organizational identity (Institution)
- Enables institutional evolution via standard contribution lifecycle

**Institution Properties**:
- `intent`: Why this institution exists
- `scope`: What it governs/coordinates
- `lifecycle_phase`: Emergent | Provisional | Formalized | Operational | Dormant | Archived
- `charter_refs`: References to constitutions/charters
- `parent_institution_id`: Lineage reference
- `affiliated_spaces`: Lateral relationships
- `stewards`: Non-permanent leadership (informational, not authoritative)

**Graph Relationships**:
- `Institution → governs → Space`
- `Institution → derives_from → Institution`
- `Institution → charters → Contribution`
- `Institution → operates_through → Project/Initiative`
- `Institution → affiliated_with → Space`

**Implications**:
- Phase 1: Semantic layer only (no governance logic)
- Phase 2: Governance modules (voting, quorum, delegation) attach at Institution level
- Phase 3: Temporal workflows for institutional processes

**Validation**: Stress-tested against Apache Foundation, UN, Mondragon, Ethereum Foundation, informal hackerspaces. All passed.

---

## DEC-009: Prioritize Bounty Contribution Type
**Date**: 2026-02-24
**Status**: ✅ Decided
**Reference**: [016-nostra-skills-sync-service-use-case](../016-nostra-skills-sync-service-use-case/RESEARCH.md)

**Context**: The "Incentivized Telemetry" loop necessary for agent skill reinforcement requires a Bounty mechanism.
**Decision**: Prioritize `Bounty` implementation to Phase 1 instead of "Phase 3 (Future)".
**Rationale**: Essential for monetized agent telemetry loops.
