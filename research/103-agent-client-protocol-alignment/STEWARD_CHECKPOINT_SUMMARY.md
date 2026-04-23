---
id: "103-agent-client-protocol-alignment-steward-checkpoint-summary"
name: "acp-steward-checkpoint-summary"
title: "ACP Steward Checkpoint Summary"
type: "summary"
project: "nostra"
status: approved
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-07"
---

# ACP Steward Checkpoint Summary

## Decision Context
Initiative `103-agent-client-protocol-alignment` has completed pilot hardening and operationalization for ACP in `cortex-desktop`, including explicit pilot gating, durability controls, and test-backed protocol lifecycle behavior.

## What Is Complete
1. ACP pilot feature gate implemented via `CORTEX_ACP_PILOT`.
2. ACP JSON-RPC lifecycle implemented (`initialize`, `session/*`, `terminal/*`).
3. `_meta` namespace/trace validation profile enforced.
4. Permission ledger + session replay durability implemented.
5. Event projection and hybrid durability path implemented:
   - local JSONL persistence
   - remote log-registry emit with retries + idempotency header
   - local outbox fallback + startup flush hook
6. Terminal policy hardening implemented:
   - wait timeout clamping
   - runtime auto-kill
   - deterministic stop reason contract
7. Local staging operational drill executed:
   - outage -> fallback queue -> recovery drain
   - ACP metrics endpoint verification (`/api/metrics/acp`)

## Verification Snapshot (2026-02-07)
1. ACP module suite:
   - `RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_`
   - Result: `23 passed, 0 failed`
2. Gateway integration lifecycle:
   - `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored`
   - Result: `1 passed, 0 failed`
3. Staging operationalization drill:
   - `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_staging_operationalization -- --ignored`
   - Result: `1 passed, 0 failed`

## Key Evidence Artifacts
- Gate report: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_GATE_REPORT.md`
- Staging evidence: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md`
- Adoption recommendation: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/ADOPTION_RECOMMENDATION.md`
- Pilot runbook: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_RUNBOOK.md`
- Decisions log: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/DECISIONS.md`

## Residual Risks
1. 14-day staging SLO evidence is still required before `pilot -> adopt` promotion.
2. Log-registry endpoint auth model remains environment-scoped.
3. Cross-product auth boundary redesign remains explicitly out of scope.

## Steward Decision State
1. Current state: approved `pilot` continuation.
2. Next required action: complete 14-day SLO evidence window and record final promotion decision (`adopt` or `watch`) in `DECISIONS.md`.
