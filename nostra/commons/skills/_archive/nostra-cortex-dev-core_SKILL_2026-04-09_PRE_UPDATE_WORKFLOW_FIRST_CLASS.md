---
name: nostra-cortex-dev-core
description: Mandatory preflight and governance-aligned coding workflow for Nostra and Cortex development, including FM-009 dynamic source checks before and after edits.
---

# Nostra Cortex Dev Core

Use this skill for any implementation work touching Nostra, Cortex, shared standards, scripts, or research governance surfaces.

## Required Workflow

1. Run preflight before implementation:
   - `bash scripts/check_agent_preflight_contract.sh`
   - `bash scripts/check_dynamic_config_contract.sh`
   - `bash scripts/check_skill_policy.sh` (required when skill governance surfaces are in scope)
2. Validate relevant context before edits:
   - target initiative `research/*/PLAN.md`
   - `docs/architecture/standards.md`
   - `docs/architecture/nostra-cortex-boundary.md`
3. Apply boundary-first development:
   - Nostra defines what exists.
   - Cortex defines how execution runs.
4. Prevent FM-009 regressions:
   - no hardcoded canister IDs
   - no hardcoded workspace roots
   - no governance bypass literals
5. Run post-edit verification:
   - `bash scripts/check_dynamic_config_contract.sh`
   - `bash scripts/check_skill_policy.sh` (required when skill governance files changed)

## Required Response Evidence

Every closeout must include:
1. `Preflight Evidence`
2. `Dynamic Source Evidence`
3. `Skill Policy Evidence` (when skill governance surfaces changed)

### Dynamic Source Evidence format
- governed value
- actual source used (env/config/governance contract)
- file path(s) changed

## FM-009 Escalation

If `FM-009` triggers:
1. classify subtype
2. update dynamic contracts/patterns/tests
3. apply exception only with owner + expiry
4. log DECISIONS entry for recurrence/escalation
