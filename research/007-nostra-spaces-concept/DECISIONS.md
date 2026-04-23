---
id: "007"
name: "nostra-spaces-concept"
title: "Decisions: Spaces Concept"
type: "decisions"
project: "nostra"
status: active
authors:
  - "User"
tags: ["spaces", "templates", "permissions", "workflows"]
created: "2026-01-14"
updated: "2026-02-25"
---

# Decisions: Spaces Concept
<!-- id: dec-spaces -->

## Decided

### DEC-Spaces-001: Hybrid Space Identity (Templates)
**Date**: 2026-01-14
**Status**: ✅ Decided

**Context**: User clarified that Sprints/Meetings can be both Views or fully permissioned Spaces.

**Decision**: Implement a **Template System** where any "Container" can be instantiated as:
1.  A **View** (lightweight, inherits parent permissions, just a filter/query).
2.  A **Space** (heavyweight, distinct permissions, distinct config, lifecycle).

**Rationale**: Flexibility. A quick "Morning Standup" might just be a View filter on "Today's Tasks", while a "Q1 Strategy Meeting" needs its own permissions, agenda, and frozen artifacts (Space).

---

### DEC-Spaces-002: Modular Lifecycle & Voting
**Date**: 2026-01-14
**Status**: ✅ Decided

**Context**: Voting and interactions happen across all lifecycle stages (Pre-meeting icebreakers, Post-meeting ratification).

**Decision**: **Decouple Voting from Lifecycle States**. State controls *visibility/highlighting*, but voting modules can be active in any state if configured.

**Example**: A "Pre-Meeting" state can still host an active "Ice Breaker Vote".

**Integration**: Uses `Vote` primitive from `013-nostra-workflow-engine` with configurable strategies.

---

### DEC-Spaces-003: Multiple Personal Spaces
**Date**: 2026-01-14
**Status**: ✅ Decided

**Context**: "Users can have many personal / private spaces not just one profile space."

**Decision**: **Unlimited Private Roots**.
- Users can create multiple independent Personal Spaces (e.g., "My Portfolio", "Private Journal", "Sandbox A").
- One of these may be designated as the "Primary Profile Space" for discovery.

---

### DEC-Spaces-004: Composable Permissions & Workflows
**Date**: 2026-01-14
**Status**: ✅ Decided

**Context**: Dynamic configuration needs to include processes.

**Decision**: **Workflows are a Configuration Module**.
- Owners can attach specific "Process Templates" from `013-nostra-workflow-engine` (e.g., "Ratification Flow") to a Space.
- Permissions can be "Programmable" (e.g., "Guest" becomes "Contributor" during a specific workflow step).

---

### DEC-Spaces-005: AI Agent Integration
**Date**: 2026-01-16
**Status**: ✅ Decided

**Context**: Spaces need to define how AI agents can interact with them, per `014-ai-agents-llms-on-icp` research.

**Decision**: **Adopt `014` 3-tier Architecture**.
- Personal Spaces act as **Layer 1 (Brain)** containers storing world model and agent identity.
- Spaces define AI agent permission levels:
    - **ReadOnly**: AI can query and summarize (v2.0 default).
    - **Draft**: AI proposes contributions, user confirms (v2.1).
    - **Delegate**: AI has scoped write permissions (v2.2+).

**Rationale**: Aligns with `002` DEC-006 (AI Agent Access Model) and enables progressive adoption.

---

### DEC-Spaces-006: Monetization Model
**Date**: 2026-01-16
**Status**: ✅ Decided

**Context**: Spaces need to support paid access and services, per `013-nostra-workflow-engine` monetization requirements.

**Decision**: **Integrate with `013` Financial Primitives**.
- Space Owners can define access as "Free", "Paid", or "Token-Gated".
- Paid spaces use `PaymentGate`, `EscrowLock`, `Payout` primitives.
- Support ICRC-1/ICRC-2 compliant tokens for multi-token payments.

**Rationale**: Enables Space Owners to monetize workflows as services (e.g., "Code Audit Service" for 50 ICP).

---

### DEC-Spaces-007: Cross-Initiative Alignment
**Date**: 2026-01-16
**Status**: ✅ Decided

**Context**: Spaces concept must align with all related research initiatives.

**Decision**: **Explicit Dependency Documentation**.
This research explicitly depends on and aligns with:
- `002-nostra-v2-architecture` - Unified Contribution Model, AI Access Model
- `004-unified-architecture-gaps` - ResourceRef, Event Bus standards
- `008-nostra-contribution-types` - Extended contribution taxonomy
- `013-nostra-workflow-engine` - Process primitives, monetization
- `014-ai-agents-llms-on-icp` - 3-tier AI architecture

**Rationale**: Ensures coherent implementation across the Nostra ecosystem.

---

### DEC-Spaces-008: Contextual Profiles
**Date**: 2026-01-14
**Status**: ✅ Decided

**Context**: "A user could have various profiles but only one per joined (parent) space."

**Decision**: **Space-Scoped Identity**.
- Users can customize how they appear in a specific Parent Space (e.g., "Anon" in DAO Space, "Real Name" in Professional Space).
- The "member" record in a Space links to a specific Profile configuration.

---

### DEC-Spaces-009: Polymorphic Block Integration
**Date**: 2026-02-25
**Status**: ✅ Decided

**Context**: Initiative 124 introduced the Universal Polymorphic Block (`a2ui`, `rich_text`, `media`, `structured_data`, `pointer`). Space Views and Dashboards need a canonical rendering unit.

**Decision**: **Space Views iterate over Polymorphic Blocks.** A Space "View" is a filtered query that renders an ordered stream of Polymorphic Blocks. Dashboards do not use custom layout engines; they compose heterogeneous block types (text, widgets, media) in a single feed component.

**Rationale**: Eliminates bespoke rendering pipelines per content type. A "Morning Standup" view natively interleaves `RichText` updates, `A2UI` polls, and `Media` charts without special-casing.

**Cross-Reference**: `124-polymorphic-heap-mode` (Polymorphic Block schema), `080-dpub-standard` (Chapter composition).

---

## Cross-Project References

| Topic | Related Research |
|-------|------------------|
| Unified Contribution Model | [002-nostra-v2-architecture](../002-nostra-v2-architecture/DECISIONS.md) |
| AI Agent Access Model | [002-nostra-v2-architecture DEC-006](../002-nostra-v2-architecture/DECISIONS.md#dec-006-ai-agent-access-model) |
| ResourceRef Standard | [004-unified-architecture-gaps](../004-unified-architecture-gaps/PLAN.md) |
| Contribution Taxonomy | [008-nostra-contribution-types](../008-nostra-contribution-types/PLAN.md) |
| Workflow Primitives | [013-nostra-workflow-engine](../013-nostra-workflow-engine/REQUIREMENTS.md) |
| Financial Primitives | [013-nostra-workflow-engine FR-15/16](../013-nostra-workflow-engine/REQUIREMENTS.md#25-monetization--services) |
| 3-Tier AI Architecture | [014-ai-agents-llms-on-icp](../014-ai-agents-llms-on-icp/RESEARCH.md) |
