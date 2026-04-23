---
id: '063'
name: testing-methodology
title: 'Feedback: Standard Testing Methodology (063)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback: Standard Testing Methodology (063)

## 2026-01-24: Agentic Flakiness
- **Source**: AI Agent (Task Execution)
- **Question/Concern**: How do we prevent flakiness in agent-driven UI tests?
- **Resolution**: Establish "Semantic Assertions" and "Drift Budgets" rather than strict string matching.
- **Decision**: → DEC-001 (Dimension 3).

## 2026-01-20: Mocking vs Simulation
- **Source**: User
- **Question/Concern**: Should we run a full IC replica for all tests?
- **Resolution**: No, prefer PocketIC for speed and determinism; only use full replica for E2E.
- **Decision**: → DEC-002.
