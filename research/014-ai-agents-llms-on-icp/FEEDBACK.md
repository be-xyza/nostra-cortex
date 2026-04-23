---
id: '014'
name: ai-agents-llms-on-icp
title: Feedback & Open Questions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback & Open Questions

## Open Questions

### Q1: TEE Maturity
- **Question**: Is the Anda Framework / IC-TEE mature enough for production use?
- **Context**: If not, we must rely on standard off-chain python scripts for the short term.
- **Status**: Investigating.

### Q2: Vector Database Hosting
- **Question**: Should we host a Vector DB on-chain (e.g., `vectordb` canister) or off-chain (Pinecone/Weaviate)?
- **Trade-off**: On-chain is "Trustless" but expensive/slow. Off-chain is fast but centralized.
