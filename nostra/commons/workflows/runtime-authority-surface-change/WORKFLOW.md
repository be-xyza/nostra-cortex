---
id: runtime-authority-surface-change
title: Runtime Authority Surface Change
owner: Systems Steward
updated_at: 2026-04-09
---

# Runtime Authority Surface Change

## Purpose
Govern changes to operator-only execution infrastructure surfaces such as provider inventory, runtime hosts, auth bindings, execution bindings, discovery diagnostics, and runtime status.

## Triggers
- Changes to `/api/system/*` execution-infrastructure surfaces
- Changes to Hetzner/VPS runtime authority docs, manifests, or systemd templates
- Changes to runtime authority schema consumers

## Inputs
- Runtime authority schema and repo-contract runbook docs
- Updated deploy/unit/config state for the affected surface

## Lanes
- `repo-contract`: documentation, schema, or deploy-template changes inside repo authority.
- `operator-surface`: code or route changes affecting operator-only runtime surfaces.
- `deploy-followup`: changes that require explicit promotion or VPS-side action.

## Analysis Focus
- Whether the affected surface remains operator-only and redacted correctly.
- Whether deploy/runbook/systemd authority artifacts stay aligned.
- Whether execution eligibility rules remain server-side.

## Steps
1. Classify the affected operator-only surface.
2. Confirm the change preserves operator-only visibility and server-side eligibility checks.
3. Run the repo-contract authority verification path.
4. Record any deploy/runbook/systemd fallout that must accompany the code change.

## Outputs
- Runtime authority verification evidence
- Clear operator follow-up notes when deploy surfaces changed

## Observability
- Capture which runtime authority surface changed and what evidence was produced.
- Record manifest, unit, or runbook drift if present.
- Note repeated mismatches between repo docs and deploy artifacts.

## Required Checks
- `bash scripts/check_vps_runtime_authority.sh --repo-contract`
