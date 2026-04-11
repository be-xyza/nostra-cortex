---
id: dual-host-parity-validation
title: Dual Host Parity Validation
owner: Systems Steward
updated_at: 2026-04-09
---

# Dual Host Parity Validation

## Purpose
Verify that `cortex-web` and desktop/eudaemon remain host-neutral consumers of shared runtime contracts.

## Triggers
- Changes to `cortex-web`, Workbench routes, shared A2UI projections, or readiness/build/status endpoints
- Changes to host parity specs or dual-host verification scripts

## Inputs
- The canonical host-parity spec and latest runtime/web contract changes
- Relevant replay, parity, or smoke artifacts

## Lanes
- `shared-contract`: a runtime contract changed and both hosts must follow.
- `host-drift`: one host diverged from the shared contract.
- `ui-followup`: parity holds at the runtime layer but host rendering/interaction still needs adjustment.

## Analysis Focus
- Shared contract ownership versus host-local divergence.
- Whether the affected surface is API, A2UI projection, readiness/status, or route behavior.
- Whether the change requires host updates or a contract rollback.

## Steps
1. Identify the shared runtime surface affected by the change.
2. Confirm web and desktop consume the same server-backed contracts.
3. Run host parity verification and capture any divergence.
4. Classify whether the change is a shared contract update or host-specific drift.

## Outputs
- Host parity verification result
- Explicit statement of whether any host fork was introduced

## Observability
- Capture the parity script result and the affected shared surface.
- Record whether divergence was in API shape, render behavior, or host-specific handling.
- Track recurring host forks as a structural regression signal.

## Self-Improvement
- If the same host parity mismatch repeats, create a narrower workflow or targeted parity script.
- If parity checks are too coarse to localize failures, add better fixture or route-level reporting.

## Required Checks
- `bash scripts/check_cortex_dual_host_parity.sh`
