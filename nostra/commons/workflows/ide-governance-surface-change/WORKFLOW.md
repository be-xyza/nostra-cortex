---
id: ide-governance-surface-change
title: IDE Governance Surface Change
owner: Systems Steward
updated_at: 2026-04-09
---

# IDE Governance Surface Change

## Purpose
Govern changes to preflight, dynamic-source policy, skill/workflow registries, and other IDE developer-agent control surfaces.

## Triggers
- Edits to `AGENTS.md`
- Edits to `shared/standards/*contract*`
- Edits to skill/workflow registries or sync/check scripts

## Inputs
- Updated governance contracts and agent instructions
- Skill and workflow registry state

## Lanes
- `preflight-change`: preflight contract or gate sequencing changed.
- `registry-change`: skill/workflow registry or sync behavior changed.
- `policy-change`: dynamic-source, skill policy, or other governance contract changed.

## Analysis Focus
- Whether the IDE control plane remains repo-local, deterministic, and boundary-first.
- Whether the change affects trigger behavior, install parity, or governed evidence expectations.
- Whether a policy/check change is migration-only or should become blocking.

## Steps
1. Identify whether the change affects preflight, dynamic-source, skill policy, or workflow declaration behavior.
2. Run the corresponding governance checks.
3. Confirm the change preserves repo-local determinism and boundary-first behavior.
4. Record any new required closeout evidence or operator expectations.

## Outputs
- Governance-surface classification
- Updated validation evidence for the touched control plane

## Observability
- Capture which control-plane surface changed and which checks were rerun.
- Record whether the change affects trigger quality, install parity, or evidence requirements.
- Note repeated control-plane regressions as candidates for narrower workflows or stronger gates.

## Required Checks
- `bash scripts/check_agent_preflight_contract.sh`
- `bash scripts/check_dynamic_config_contract.sh`
- `bash scripts/check_skill_policy.sh`
- `bash scripts/check_workflow_declarations.sh`
