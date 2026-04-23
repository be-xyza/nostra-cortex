---
id: '061'
name: zk-proof-integration
title: 'Feedback: ZK Proof Integration (061)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback: ZK Proof Integration (061)

## 2026-01-24: Verification Performance
- **Source**: AI Agent (Logic Audit)
- **Question/Concern**: Is verification fast enough for real-time UI gating?
- **Resolution**: Implementation of "Async Gating" in `AttestedLab` and `ZkLab`; UI shows spinner while verification completes (approx 200ms).
- **Decision**: → Phase 1 UX pattern.

## 2026-01-22: zkVM Selection
- **Source**: User
- **Question/Concern**: Should we run RISC0/SP1 directly in Cortex?
- **Resolution**: No, too heavy for browser WASM. Delegation to zCloak is the preferred architecture.
- **Decision**: → DEC-001
