---
id: '106'
name: cortex-decision-surfaces
title: Requirements
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Requirements

## Functional
1. Surface all decision-critical metadata: `workflow_id`, `mutation_id`, `gate_status`, `gate_level`, replay references, and required actions.
2. Provide projection endpoints for execution profile, attribution domains, governance scope, replay contract, and latest decision gates.
3. Enforce decision-action payload quality for risky gates (`risk_statement`, `rollback_path`, `evidence_refs`).
4. Persist decision lineage artifacts at gateway with deterministic action IDs.

## Non-Functional
1. Additive-only API changes.
2. Deterministic behavior for identical inputs.
3. Inbox-first operator discoverability.
4. Adapter neutrality and layer boundaries preserved.
