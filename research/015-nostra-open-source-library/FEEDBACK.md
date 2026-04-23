---
id: '015'
name: nostra-open-source-library
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

**Context**: Collection of open questions, user feedback, and resolved inquiries regarding the Open Source Library.

## Open Questions

### Q1: Metric of "Fit"
- **Question**: How do we mathematically score the "fit" between an abstract Idea and a concrete Library?
- **Context**: Need to rank search results in the Feasibility Check.
- **Possibilities**: Vector Similarity (Embedding distance) vs Function Signature Matching.

### Q2: Verification Trust
- **Question**: How does the system verify that a library *actually* does what the README says?
- **Ideas**: Auto-generated test harnesses? Reputation staking?

## Resolved Items

### Q3: Evolution & Updates (Resolved via 127)
- **Question**: How does the library handle version updates?
- **Context**: Re-analyzing on every commit is expensive.
- **Resolution**: Resolved by **Initiative 127**. The library updates dynamically when `127`'s temporal sync daemon detects changes in the upstream repo, triggers a Semantic Diff evaluator, and a maintainer approves the resulting `Proposal`.
