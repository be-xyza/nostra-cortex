# System Integrity + Quality (SIQ) Contract

This directory defines the SIQ contract for Cortex governance/execution integrity checks and graph-ready projections.

## Canonical Artifacts

- `/Users/xaoj/ICP/logs/siq/siq_coverage_latest.json`
- `/Users/xaoj/ICP/logs/siq/siq_dependency_closure_latest.json`
- `/Users/xaoj/ICP/logs/siq/siq_gate_summary_latest.json`
- `/Users/xaoj/ICP/logs/siq/graph_projection_latest.json`
- `/Users/xaoj/ICP/logs/siq/runs/<run_id>.json`

## Schemas

- `siq_governance_gate.schema.json`
- `siq_graph_projection.schema.json`
- `GRAPH_PROJECTION_MAPPING.md`

## Command Contract

- `bash scripts/run_siq_checks.sh --mode observe`
- `bash scripts/run_siq_checks.sh --mode softgate`
- `bash scripts/run_siq_checks.sh --mode hardgate`

## Failure Classes

- `missing_gate_evidence`
- `parity_contract_drift`
- `governance_contract_mismatch`
