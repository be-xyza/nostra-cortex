# Initiative Kickoff Approval

Date: 2026-03-26
Owner: Systems Steward

## Purpose

Initiative kickoff approval is a bounded, approval-first heap primitive for starting well-scoped initiative work without bypassing stewardship.

It is appropriate when an initiative already has:
- governed initiative identity in `research/*/PLAN.md`
- active status and steward metadata
- a bounded kickoff packet with required tasks, references, bottleneck signals, and fallback routes

It is not a generic "start initiative" button.

## Authority Model

- `PLAN.md` frontmatter remains the canonical authority for initiative identity, title, status, and stewardship.
- Adjacent `KICKOFF.toml` metadata is the canonical source for kickoff-specific routing data.
- Cortex may project that metadata into heap approval solicitations and routed tasks, but it does not replace initiative governance.

## Runtime Contract

1. A privileged user requests kickoff approval.
2. Cortex emits an `agent_solicitation` with `approval_kind = initiative_kickoff`.
3. A steward records feedback.
4. If approved, Cortex emits a follow-up kickoff `task`.
5. The kickoff task still routes through the task router before execution.

## Eligibility Rules

An initiative is kickoff-launchable only when all of the following are true:
- `PLAN.md` exists
- initiative status is `active`
- `stewardship.primary_steward` is present
- `KICKOFF.toml` exists and `enabled = true`
- kickoff metadata defines:
  - label and description
  - agent role
  - required capabilities
  - reference paths
  - required tasks

If any of those conditions fail, the initiative must not appear in the kickoff launcher.

## Boundaries

- Kickoff approval does not authorize structural mutations.
- Kickoff approval does not replace steward-gated APIs.
- Kickoff approval does not turn initiative plans into executable workflow definitions.
- Kickoff approval is for bounded kickoff packets, not open-ended research exploration.
