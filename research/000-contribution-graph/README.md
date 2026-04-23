---
id: '000'
name: contribution-graph
title: "000 \u2014 Contribution Graph DPub Bootstrap"
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-17'
updated: '2026-02-18'
---

# 000 — Contribution Graph DPub Bootstrap

This corpus is the canonical DPub-native bootstrap for portfolio mapping, validation,
path assessment, and simulation on top of `cortex-domain` primitives.

Steward runbook: `STEWARD_ACTION_GUIDE_V0_2_0.md`

## Artifacts

- `dpub.json`
- `contribution_graph.json`
- `path_assessment.json`
- `doctor_report.json`
- `editions/<version>/edition_manifest.json`
- `editions/<version>/snapshot.json`
- `schemas/*.json`
- `scenarios/*.yaml`
- `simulations/*.json`

## Command Contract

```bash
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- validate --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- ingest --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- path --goal stable-cortex-domain --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- explain-path --goal accelerate-118 --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- doctor --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- simulate \
  --scenario /Users/xaoj/ICP/research/000-contribution-graph/scenarios/accelerate_118.yaml \
  --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- publish-edition --version v0.2.0 --root /Users/xaoj/ICP

cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml \
  --bin nostra-contribution-cli -- diff-edition --from v0.1.0 --to v0.2.0 --root /Users/xaoj/ICP
```
