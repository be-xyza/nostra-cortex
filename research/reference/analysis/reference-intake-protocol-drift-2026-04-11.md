# Reference Intake Protocol Drift - 2026-04-11

## Summary

The `In-Place TTT` intake was primary-source manually validated. During remediation, the active reference-governance docs were narrowed to match the real local contract in this checkout instead of implying missing validator/topic-registry assets were available.

## Missing Local Assets Observed During Remediation

The following assets remain absent locally as of 2026-04-11:

- `research/reference/knowledge/PAPER_TEMPLATE.md`
- `research/reference/analysis/ANALYSIS_TEMPLATE.md`
- `docs/reference/knowledge_taxonomy.toml`
- `docs/reference/topics.md`
- `scripts/check_reference_metadata_v2.py`
- `scripts/check_reference_taxonomy_integrity.py`

## Impact

- Intakes can still be retained through manual primary-source validation.
- Active docs now state the honest local mode: `primary-source manually validated`.
- Validator-backed compliance must still not be claimed unless the missing assets are restored and actually executed.

## Required Follow-Up

Choose one governance remediation path and complete it explicitly:

1. Restore the missing validator, taxonomy, topic-registry, and template assets so README claims are executable again.
2. Keep the narrowed manual-validation contract and remove any remaining active-doc references to missing assets if new ones appear.

Until one of those happens, the correct closeout language for knowledge intake in this checkout is `primary-source manually validated`.
