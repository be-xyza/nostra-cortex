# Steward Action Guide - DPub Bootstrap Corpus (v0.2.0)
Date: 2026-02-18
Scope: `/Users/xaoj/ICP/research/[0-9][0-9][0-9]-*` initiatives only

## 1) Current State (Confirmed)

- Canonical graph artifact exists: `/Users/xaoj/ICP/research/000-contribution-graph/contribution_graph.json`
- Current baseline:
  - `nodes=120`
  - `edges=86`
  - `critical=0`
  - `violation=10`
  - `warning=139`
  - `unresolved_refs=0`
  - `graph_root_hash=ebacedb6b0a3cb2cc255e380dd21c76e7b1813da8d2c07d55f88edd1d2a2db2e`
- Doctor report passes: `/Users/xaoj/ICP/research/000-contribution-graph/doctor_report.json` (`pass=true`)
- Path bundle exists for:
  - `stable-cortex-domain`
  - `accelerate-118`
- v0.2.0 edition published:
  - `/Users/xaoj/ICP/research/000-contribution-graph/editions/v0.2.0/edition_manifest.json`
  - `/Users/xaoj/ICP/research/000-contribution-graph/editions/v0.2.0/snapshot.json`

## 2) Desktop State (Confirmed)

- Initiative Explorer is enriched and wired to generated artifacts in:
  - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/components/views/initiative_explorer_view.rs`
- Implemented UX surfaces:
  - Graph metrics + extraction confidence cards
  - Recommended Path panel
  - Why-This-Path evidence panel (source refs + line numbers)
  - Goal Path Matrix
  - Simulation Outputs panel
- Local restart validated on 2026-02-18:
  - Launcher rebuilt and started app
  - Readiness endpoint returned:
    - `{"ready":true,"gateway_port":3000,"dfx_port_healthy":true,"notes":[]}`

## 3) Immediate Steward Actions (Start Now)

1. Lock execution authority
   - Confirm this remains an **118 execution slice**, not a new standalone initiative.
   - Keep corpus authority in `/Users/xaoj/ICP/research`.
2. Run canonical gate
   - Execute: `/Users/xaoj/ICP/scripts/check_contribution_graph_bootstrap.sh`
   - Require pass before steward approvals for major plan sequencing.
3. Burn down remaining violations (10 -> target <=5 this sprint)
   - Use `/Users/xaoj/ICP/research/000-contribution-graph/path_assessment.json` rule violations as backlog source.
   - Prioritize governance-sensitive and cross-layer blockers first.
4. Enforce hard-fail metadata at review
   - Reject plan merges with status drift/frontmatter drift.
   - Keep unresolved references at `0`.
5. Publish weekly editions
   - Run `publish-edition` and `diff-edition`.
   - Keep lineage and path-quality trend visible to stewards.

## 4) Execution Process (Steward Runbook)

### Daily (Operator)

1. `ingest`
2. `validate`
3. `doctor`
4. `path --goal stable-cortex-domain`
5. `path --goal accelerate-118`
6. `simulate` using approved scenario templates
7. If gates pass, publish candidate edition

### Weekly (Steward Review)

1. Review top path recommendation and score breakdown.
2. Approve remediation tasks for highest-risk blocking nodes.
3. Confirm no superseded initiatives are driving recommended paths.
4. Review desktop parity:
   - Graph hash
   - Node/edge counts
   - Recommended path identity
5. Publish steward packet for governance records.

## 5) No-Blocker Confirmation

There are no architectural blockers to continue the corpus project.

Execution watchpoints:
- Metadata consistency discipline must remain strict.
- Violation/warning volume should keep trending down each edition.
- Desktop must remain read-only against CLI-produced artifacts.

Resolved runtime blocker:
- Patched launcher temp-file bug in `/Users/xaoj/ICP/cortex/apps/cortex-desktop/run_cortex.command` so rebuild no longer fails on `mktemp`.

## 6) Recommended Enrichments (Next 7-14 Days)

1. Add path score decomposition panel in desktop (critical/violation/warning/cross-layer/superseded breakdown).
2. Add blast-radius quick actions:
   - "What breaks if X changes?"
   - "What does 118 invalidate?"
3. Add edition time slider (`v0.1.0 -> latest`) with risk trend chart.
4. Add confidence heatmap filter (`explicit`, `inferred`, `steward-confirmed`).
5. Add one-command steward packet export:
   - Markdown bundle with recommendation, evidence links, blockers, remediation tasks.

## 7) Completion Criteria for Corpus Project

Mark corpus project execution complete when all are true:

1. `critical=0` sustained across 2 consecutive weekly editions.
2. `violation<=5` and decreasing trend.
3. `unresolved_refs=0` sustained.
4. Deterministic graph hash stable on no-change reruns.
5. Desktop parity checks pass against CLI artifacts.
6. Steward packet export and review cadence is active.

## 8) Command Reference

```bash
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- ingest --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- validate --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- doctor --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- explain-path --goal stable-cortex-domain --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- path --goal accelerate-118 --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- publish-edition --version v0.2.0 --root /Users/xaoj/ICP
cargo run --manifest-path /Users/xaoj/ICP/nostra/extraction/Cargo.toml --bin nostra-contribution-cli -- diff-edition --from v0.1.0 --to v0.2.0 --root /Users/xaoj/ICP
bash /Users/xaoj/ICP/scripts/check_contribution_graph_bootstrap.sh
```
