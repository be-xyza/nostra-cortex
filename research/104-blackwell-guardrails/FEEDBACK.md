---
id: '104'
name: blackwell-guardrails
title: 'Feedback: Blackwell Guardrails'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-07'
updated: '2026-02-08'
---

# Feedback: Blackwell Guardrails

## Open Questions
- Which thresholds should be tuned first after observe telemetry?
- Should `require_simulation` escalate to automatic workflow pause in soft mode?
- Which quality signal should dominate policy in tie-break conditions?

## Calibration Protocol (Observe Window)
1. Duration: 14 days in `Observe` mode.
2. Sampling: Review every `governance` and `merge` assessment with `warn`, `require_review`, or `require_simulation`.
3. Logging format (per sample):
   - `assessment_id`, `workflow_id`, `mutation_id`, `decision_class`
   - `gate_outcome`, `reasons[]`, `robustness`, `voi`, `regret`
   - Operator adjudication: `true_positive | false_positive | false_negative`
   - Final real-world outcome after 24-72h (`no_change | corrected | reverted | escalated`)
4. Override quality rubric:
   - `high`: concrete risk/rollback reasoning + evidence reference
   - `medium`: plausible rationale but weak evidence link
   - `low`: generic justification with no traceability

## False-Positive / False-Negative Collection Format
- Store each adjudicated sample with:
  - `sample_id`, `captured_at`, `assessment_id`, `workflow_id`, `mutation_id`
  - `decision_class`, `gate_outcome`, `operator_decision`
  - `adjudication` (`TP|FP|FN|TN`)
  - `justification_quality` (`high|medium|low`)
  - `followup_outcome` (`no_change|corrected|reverted|escalated`)
  - `notes` (free text, <= 500 chars)
- Persist snapshots in `logs/testing/blackwell_calibration_samples_latest.json` (append-only per window).

## SLOs (Stage Enrichment)
- **Projection Freshness**: `system_blackwell_status_latest.json` updated within 20 minutes of run completion.
- **Sync Reliability**: governance->workflow sync success >= 99% over rolling 7 days.
- **Override Discipline**: low-quality overrides < 20% of all overrides over rolling 7 days.
- **Missed-Risk Guardrail**: false negatives for governance/merge <= 5% over rolling 14 days.

## First Tuning Sequence
1. Tune `min_evidence` and `min_alternatives` first (largest false-positive leverage, lowest blast radius).
2. Tune `min_robustness` second (directly changes simulation pressure).
3. Tune pressure caps (`max_confidence_drift`, `max_fork_pressure`, `max_correction_density`) third.
4. Keep `hard_gate` floor (0.55) fixed during first observe window unless false negatives exceed threshold.

## Escalation Trigger
- Escalate to steward review if:
  - false-negative rate for governance/merge exceeds 5%, or
  - low-quality overrides exceed 20% of overrides in any 7-day period.
  - projection freshness SLO breached for 3 consecutive runs.
