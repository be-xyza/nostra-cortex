---
id: '108'
name: cortex-decision-plane-security-legibility
title: Decisions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions

## D-108-001 — Signed Intent Rollout
- Decision: Adopt staged signed-intent enforcement with mode enum.
- Rationale: Supports safe migration from permissive development mode to strict production enforcement.

## D-108-002 — Canonical Actor Role Authority
- Decision: Governance canister actor-role bindings are canonical; env mappings are explicit labs fallback only.
- Rationale: Reduces hidden authority paths and aligns role decisions with auditable policy state.

## D-108-003 — Replay Lineage Digest
- Decision: Expose deterministic decision digest and lineage references on replay artifacts.
- Rationale: Enables reproducible audit/replay without requiring gateway-local cache as truth.
