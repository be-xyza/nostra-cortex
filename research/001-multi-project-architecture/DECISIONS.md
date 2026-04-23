---
id: '001'
name: multi-project-architecture
title: 'Decision Log: Multi-Project Architecture'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decision Log: Multi-Project Architecture

Track architectural decisions with rationale for future reference.

---

## DEC-001: Canister Funding Model
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. User-funded (users top up their own canisters)
2. Platform-subsidized (platform pays for canister creation)

**Decision**: Platform-subsidized cycles

**Rationale**:
- Lower friction for user onboarding
- Platform can implement rate limiting (5 free projects)
- Enables freemium business model

**Implications**:
- Registry Canister needs cycles wallet integration
- Need cost monitoring and abuse prevention

---

## DEC-002: ICP Knowledge Graph Migration
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Discard existing data, start fresh
2. Migrate to first user project
3. Create read-only "Canon" canister

**Decision**: ICP Canon Data Canister (Option 3)

**Rationale**:
- Preserves work already done (80+ entities, 90+ relationships)
- Serves as reference/seed data for new projects
- Users can fork as starting point

**Implications**:
- Need `fork()` function in Canon canister
- Current `main.mo` becomes Canon template
- Schema becomes "icp-ecosystem-v1"

---

## DEC-003: Schema Storage Strategy
**Date**: 2026-01-14
**Status**: 🟡 Proposed (pending implementation)

**Options Considered**:
1. Schema embedded in each KG canister
2. Separate Schema Registry canister
3. Schema stored off-chain (IPFS/HTTP)

**Decision**: Separate Schema Registry canister (Option 2)

**Rationale**:
- Enables schema sharing across projects
- AI agents can query schema for extraction rules
- Supports schema versioning and marketplace

**Implications**:
- Inter-canister calls from KG to Schema Registry
- Need caching strategy for performance

---

## DEC-004: Free Project Rate Limit
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Unlimited free projects
2. 3 free projects per user
3. 5 free projects per user
4. 10 free projects per user

**Decision**: 5 free projects per user (Option 3)

**Rationale**:
- Balances user flexibility with platform sustainability
- Sufficient for most personal/experimental use cases
- Higher tiers can be offered for power users (future monetization)

**Implications**:
- Registry Canister must track per-user project count
- UI should show remaining quota
- Need upgrade path for users who need more

---

## DEC-005: Canon Canister Update Cadence
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Fixed quarterly updates
2. Monthly community-driven updates
3. Dynamic governance via DAP process

**Decision**: Dynamic governance via DAP process (Option 3)

**Rationale**:
- More flexible than fixed schedule
- Enables community input on reference data
- Aligns with ICP's decentralized governance ethos
- Future-proofs for SNS/DAO integration

**Implications**:
- Separate DAP (Decentralized Application Protocol) governance process to be developed
- Canon data updates require governance proposal + approval
- Need to define proposal/voting mechanism (future research)

---

## DEC-006: Design Pattern Adoption
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Use Graphiti directly (impossible - Python/Neo4j dependency)
2. Use OpenSPG directly (impossible - Java/Python stack)
3. Extract design patterns, implement natively in Motoko/Rust
4. Hybrid with off-chain services

**Decision**: Native ICP implementation with pattern extraction (Option 3)

**Rationale**:
- Maintains decentralization (no off-chain dependencies)
- Extracts best ideas: bi-temporal model (Graphiti), semantic predicates (OpenSPG)
- ic-rmcp provides native MCP support for AI integration
- Full control over implementation

**Features Adopted**:
| Source | Pattern |
|--------|---------|
| Graphiti | Bi-temporal model (`eventTime`/`ingestedAt`) |
| Graphiti | Episodic ingestion with source tracking |
| OpenSPG | Semantic predicates (`inverseOf`, `transitivity`) |
| OpenSPG | Inference rules (simplified KGDSL) |
| ic-rmcp | Native ICP MCP server for AI tools |

**References**:
- FRAMEWORK_STRATEGY.md for full analysis
- PLAN.md updated with implementation details

---

## Template for New Decisions

```markdown
## DEC-XXX: [Title]
**Date**: YYYY-MM-DD
**Status**: 🟡 Proposed | ✅ Decided | ❌ Rejected

**Options Considered**:
1. ...
2. ...

**Decision**: [Which option]

**Rationale**: [Why this option]

**Implications**: [What this means for implementation]
```
