---
id: '107'
name: cortex-decision-plane-hardening
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
1. Gateway projection endpoints must query canisters first and annotate fallback/cached modes.
2. Mutation-gate projection must synthesize epistemic assessment, governance scope, replay contract, and test-gate status into one deterministic envelope.
3. Decision actions (`ack`, `escalate`) must enforce role and policy gates and persist deterministic lineage IDs.
4. Multi-space projection routing must be supported for decision-plane surfaces.
5. Replay contracts must expose optional lineage and policy snapshot references.

## Non-Functional
1. Additive-only interface evolution.
2. Deterministic `verb:id` decision action behavior.
3. Stable-memory durability for attribution weight policy storage.
4. No panic-based legacy decode path for workflow-engine stable structures.
