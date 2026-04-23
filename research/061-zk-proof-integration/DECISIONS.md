---
id: '061'
name: zk-proof-integration
title: 'Decisions: ZK Proof Integration (061)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: ZK Proof Integration (061)

## DEC-001: ZK Provider Choice
- **Decision**: Adopt zCloak Network as the primary ZK Coprocessor for identity and membership.
- **Rationale**: Best alignment with ICP stack; offloads expensive verification to a dedicated, high-performance canister layer.
- **Status**: Integrated (`ZkService`).

## DEC-002: Verification Logic Placement
- **Decision**: Frontend performs the initial verification call to zCloak, results are then passed to Cortex canisters.
- **Rationale**: Reduces inter-canister call complexity for initial UI gating, while still maintaining high security via signed attestations.
- **Status**: Implemented.

## DEC-003: Proof Tiering (The Ladder)
- **Decision**: Use a 4-level "Proof Ladder" (Signature, Hash, Attestation, ZK).
- **Rationale**: Allows for cost-optimization by matching verification complexity to the value of the action.
- **Status**: Schema defined and implemented.
