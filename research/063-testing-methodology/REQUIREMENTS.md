---
id: '063'
name: testing-methodology
title: 'Requirements: Standard Testing Methodology (063)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Standard Testing Methodology (063)

## Testing Stack (Vertical)
| Level | Tool | Requirement |
|-------|------|-------------|
| **L1: Unit** | `cargo test` / `moc --test` | Mandatory for all pure logic |
| **L2: Integration**| PocketIC / Temporal Mocks | Mandatory for inter-component flows |
| **L3: Simulation** | Labs Runner / Arena | Required for Agent logic & UX |
| **L4: E2E** | Playwright | Required for full protocol validation |

## Temporal Requirements
- [ ] Replay safety must be verified for all workflows.
- [ ] System must support virtual "Time-Travel" for testing schedules.

## Agentic Requirements
- [ ] All agent changes must pass an Arena Run.
- [ ] Semantic drift must be tracked against a "Golden Set".
