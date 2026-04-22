# Nostra Test Catalog Contract v1

This directory defines the canonical filesystem contract for test cataloging used by local IDE agents, CI, and Cortex Desktop.

## Canonical Artifacts

- `/Users/xaoj/ICP/logs/testing/test_catalog_latest.json`
- `/Users/xaoj/ICP/logs/testing/runs/<run_id>.json`
- `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json`

## Schemas

- `test_catalog.schema.json`
- `test_run.schema.json`
- `test_gate_summary.schema.json`

## Artifact Semantics

1. Filesystem artifacts are canonical in v1.
2. Desktop inbox/state projections are derived views and never source-of-truth.
3. Runs are append-only records under `logs/testing/runs/`.
4. Gate summary is a derived snapshot generated from catalog + run artifacts.

## Generation Commands

```bash
bash scripts/generate_test_catalog.sh
bash scripts/write_test_run_artifact.sh --environment local_ide --agent-id codex-local --scenario pass
bash scripts/write_test_run_artifact.sh --environment ci --agent-id github-actions --scenario pass
bash scripts/generate_test_gate_summary.sh --mode advisory
bash scripts/check_test_catalog_consistency.sh --mode advisory
```

## Run Artifact Helper

- `scripts/write_test_run_artifact.sh` creates a contract-compliant run artifact under `logs/testing/runs/`.
- Use `--scenario blocker_fail` to generate a deterministic negative rehearsal artifact for blocking-mode checks.

## Gate Modes

- `advisory`: emits warnings, does not fail on invalid or missing artifacts.
- `blocking`: fails when artifacts are missing/invalid or release blockers fail.

## Status Vocabulary

- `pass`
- `fail`
- `warn`
- `pending`
