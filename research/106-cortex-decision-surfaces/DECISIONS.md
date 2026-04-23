---
id: '106'
name: cortex-decision-surfaces
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

## D1 — Contract-First Incremental Delivery
Adopted. Workflow/governance canister APIs were extended first, then projected via gateway APIs, then surfaced in Cortex Desktop and frontend inbox.

## D2 — Deterministic Gateway Lineage IDs
Adopted. `decision_ack` / `decision_escalate` actions are persisted with deterministic SHA-based IDs to preserve replay/audit consistency.

## D3 — Risky-Gate Quality Enforcement at Gateway
Adopted. Risky decision actions are rejected unless risk, rollback, and evidence references are supplied.

## D4 — Inbox Label Expansion
Adopted. Added explicit inbox labels: Execution, Attribution, Governance, Replay, and Release Gate.
