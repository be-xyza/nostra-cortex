---
id: '108'
name: cortex-decision-plane-security-legibility
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
1. Gateway must evaluate decision actions against governance with actor principal + space context.
2. Gateway must support signed enforcement modes: `off`, `warn`, `required_p0_p1`, `required_all`.
3. Decision replay lineage must be queryable by mutation and listable per space.
4. Cortex decision surfaces must expose source/policy/lineage/auth metadata in operator-visible panes.
5. Risky actions must require structured quality fields: risk statement, rollback path, and evidence references.

## Non-Functional
1. Determinism: identical input history and policy context produce identical lineage digest.
2. Durability: actor-role bindings survive canister upgrades/restarts.
3. Compatibility: additive API evolution only; existing endpoints remain valid.
4. Transparency: degraded modes must emit explicit degraded reasons.
