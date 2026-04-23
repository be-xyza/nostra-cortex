---
id: "000"
name: "contribution-graph"
title: "DPub Contribution Graph Bootstrap"
type: "plan"
project: "nostra-cortex"
status: active
portfolio_role: anchor
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Agents & Execution"
created: "2026-02-18"
updated: "2026-02-18"
---

# 000 — DPub Contribution Graph Bootstrap

## Objective

Bootstrap a DPub-native initiative corpus that imports and validates research
plans/specs/decisions, maps relationships as a deterministic graph, and runs
path/simulation analysis using `cortex-domain` primitives.

## Deliverables

- Canonical CLI: `nostra-contribution-cli` (`validate`, `ingest`, `query`, `path`, `simulate`, `publish-edition`)
- Corpus artifacts under `research/000-contribution-graph/`
- Edition snapshots under `editions/v0.1.0/`
- Read-only Cortex Desktop initiative explorer route consuming generated artifacts

## Governance Rules

- Hard-fail metadata consistency before publish.
- Research corpus remains source-of-truth.
- Desktop explorer is read-only and does not maintain an independent graph model.
