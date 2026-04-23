---
id: 086
name: cobudget-integration-patterns
title: 'Research 086: Cobudget Integration Patterns - Decisions Log'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research 086: Cobudget Integration Patterns - Decisions Log

> **Initiative**: 086-cobudget-integration-patterns
> **Last Updated**: 2026-01-28

---

## DEC-001: Cobudget as Reference Architecture (Not Fork)

**Date**: 2026-01-28
**Status**: ✅ DECIDED
**Decision Maker**: Architecture Review

### Context
Cobudget is a production-grade collaborative budgeting platform. The question is whether to fork and adapt the codebase or use it as reference architecture.

### Decision
**Use Cobudget as reference architecture only.** Extract patterns and adapt them to Nostra's Rust/ICP stack rather than maintaining a Node.js fork.

### Rationale
1. **Stack Mismatch**: Cobudget is Next.js/PostgreSQL; Nostra is Dioxus/ICP Stable Storage
2. **Maintenance Burden**: Forking creates ongoing sync/merge work
3. **Pattern Value**: The architectural patterns (not code) are the primary value
4. **Constitutional Alignment**: Patterns must be adapted to fit Nostra's constitutional constraints

### Consequences
- Must translate TypeScript patterns to Rust
- Schema designs must target Candid, not Prisma
- UI components are inspiration only, not directly portable

---

## DEC-002: Computed vs Stored Status

**Date**: 2026-01-28
**Status**: ✅ DECIDED
**Decision Maker**: Technical Review

### Context
Cobudget computes bucket status from timestamps at query time rather than storing it. This ensures consistency but requires computation on every read.

### Decision
**Adopt computed status pattern.** Status should be derived from timestamps (`published_at`, `approved_at`, `funded_at`, `completed_at`, `canceled_at`) rather than stored directly.

### Rationale
1. **Single Source of Truth**: Timestamps are the authoritative data; status is derived
2. **No Sync Bugs**: Stored status can drift from reality
3. **Temporal Queries**: Can answer "what was status at time X" by replaying logic
4. **ICP Optimization**: Query compute is cheap; storage consistency is expensive

### Implementation
```rust
fn compute_status(proposal: &Proposal, funding_status: &FundingStatus) -> ProposalStatus {
    if proposal.completed_at.is_some() { return ProposalStatus::Completed; }
    if proposal.canceled_at.is_some() { return ProposalStatus::Canceled; }
    if proposal.funded_at.is_some() { return ProposalStatus::Funded; }
    if proposal.approved_at.is_some() {
        if funding_status.has_ended {
            return if total_contributions(proposal) > 0 {
                ProposalStatus::PartialFunding
            } else {
                ProposalStatus::Idea
            };
        }
        if funding_status.has_started { return ProposalStatus::OpenForFunding; }
        return ProposalStatus::Idea;
    }
    if proposal.published_at.is_some() { return ProposalStatus::Idea; }
    ProposalStatus::PendingApproval
}
```

---

## DEC-003: Double-Entry Accounting Layer

**Date**: 2026-01-28
**Status**: ✅ DECIDED
**Decision Maker**: Technical Review

### Context
Cobudget uses a simple but effective double-entry accounting model with Account and Transaction entities. Nostra needs financial tracking for treasury, grants, and bounties.

### Decision
**Create a dedicated Treasury Layer canister** implementing double-entry accounting with Account and Transaction primitives.

### Rationale
1. **Audit Trail**: Every financial movement is recorded as a transaction
2. **Balance Integrity**: Balance = sum(incoming) - sum(outgoing), always computable
3. **Multi-Currency**: Accounts can hold different token types
4. **Separation of Concerns**: Financial logic isolated from business logic

### Schema
```rust
type Account = record {
    id: text;
    owner_type: AccountOwnerType;  // Space, Member, Proposal
    owner_id: text;
    account_type: AccountType;     // Incoming, Status, Outgoing
    created_at: nat64;
};

type Transaction = record {
    id: text;
    from_account_id: text;
    to_account_id: text;
    amount: nat;
    currency: text;
    created_at: nat64;
    reference_type: opt text;      // "allocation", "contribution", "expense"
    reference_id: opt text;
};
```

---

## DEC-004: Event-Driven Architecture for Notifications

**Date**: 2026-01-28
**Status**: ✅ DECIDED
**Decision Maker**: Technical Review

### Context
Cobudget uses an in-process EventHub for pub/sub. Nostra runs on ICP with actor isolation.

### Decision
**Implement event routing via actor messaging** rather than in-process pub/sub. Events are emitted by business canisters and consumed by notification/workflow canisters.

### Rationale
1. **Actor Model Alignment**: ICP canisters are already actors
2. **Scalability**: Consumers can be added without modifying producers
3. **Reliability**: Actor messaging has delivery guarantees
4. **Workflow Integration**: Events can trigger workflow state transitions

### Event Types
```rust
type NostraEvent = variant {
    AllocationCreated: record { round_id: text; member_id: principal; amount: nat };
    ContributionMade: record { proposal_id: text; contributor_id: principal; amount: nat };
    ProposalFunded: record { proposal_id: text; total_raised: nat };
    ProposalPublished: record { proposal_id: text; title: text };
    FundingCanceled: record { proposal_id: text; reason: text };
};
```

---

## DEC-005: Balance Computation Strategy

**Date**: 2026-01-28
**Status**: ✅ DECIDED
**Decision Maker**: Technical Review

### Context
Cobudget computes balance on every read: `balance = allocations - contributions`. This ensures consistency but has performance implications.

### Decision
**Compute balance on read with optional epoch caching.** For high-frequency reads, cache balance at epoch boundaries.

### Rationale
1. **Correctness First**: Computed balance is always accurate
2. **ICP Constraints**: Query compute is cheap; update calls are expensive
3. **Epoch Optimization**: At funding round close, compute and cache final balances
4. **Cache Invalidation**: Cache cleared on allocation/contribution events

### Implementation
```rust
fn get_balance(member_id: &Principal, round_id: &str) -> Nat {
    // Check epoch cache first
    if let Some(cached) = get_cached_balance(member_id, round_id) {
        return cached;
    }

    // Compute from allocations and contributions
    let allocations: Nat = sum_allocations(member_id, round_id);
    let contributions: Nat = sum_contributions(member_id, round_id);

    allocations - contributions
}
```

---

## Open Questions

### OQ-1: ICP Ledger Integration
**Question**: Should the Treasury Layer integrate with the ICP Ledger for real token movements, or track virtual balances only?

**Options**:
1. Virtual only (simpler, no real money)
2. ICP Ledger integration (real tokens, more complex)
3. Hybrid (virtual for voting, ledger for payouts)

**Status**: PENDING - Requires decision based on tokenomics design

### OQ-2: Quadratic Funding Support
**Question**: Should the Allocation/Contribution model support quadratic funding formulas?

**Options**:
1. Linear only (simpler, direct proportional)
2. Quadratic (sqrt of individual, sum squared)
3. Configurable per round

**Status**: PENDING - Requires governance input

### OQ-3: Multi-Round Membership
**Question**: Should members have separate balances per funding round, or pooled across rounds?

**Options**:
1. Per-round (Cobudget model, isolated)
2. Pooled (any allocation usable anywhere)
3. Hierarchical (Space → Round inheritance)

**Status**: PENDING - Impacts Treasury Layer design
