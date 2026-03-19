# GSMS Bench Mapping Contract

Date: 2026-02-16
Dependencies: Initiative 091 (Nostra Bench), Initiative 118 (GSMS Activation)

## Purpose

Define the deterministic mapping from GSMS `SimulationResult` to Nostra Bench categories so Phase-0 GSMS outputs are bench-compatible without schema drift.

## Contract

`SimulationResult.bench_metrics` must always include:

1. `violation_counts` (required, populated in GSMS-0)
2. `capture_probability` (reserved, default `0.0` in GSMS-0)
3. `deadlock_rate` (reserved, default `0.0` in GSMS-0)
4. `minority_suppression_index` (reserved, default `0.0` in GSMS-0)
5. `churn_index` (reserved, default `0.0` in GSMS-0)
6. `governance_throughput` (reserved, default `0.0` in GSMS-0)

## Category Mapping

1. Analyst:
   - Source: `violation_counts`, `violation_summary.risk_score`
2. Planner:
   - Source: `governance_throughput`, `deadlock_rate`
3. Navigator:
   - Source: `capture_probability`, `minority_suppression_index`
4. Builder:
   - Not directly scored by GSMS-0 deterministic governance scenarios.

## GSMS-0 Rule

Until LLM augmentation phase starts, only `violation_counts` is treated as non-placeholder telemetry. All other bench fields remain deterministic placeholders (`0.0`) and must not gate release decisions.
