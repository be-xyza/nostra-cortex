# SIQ Graph Projection Mapping

This mapping defines how SIQ artifacts project into contribution-graph and related Cortex graph views.

## Entity Mapping

- `Contribution`
  - Source: `logs/siq/siq_coverage_latest.json.contributions[]`
  - Stable ID: `contribution:<contribution_id>`
- `Rule`
  - Source: SIQ rule registry
  - Stable ID: `rule:<rule_id>`
- `GateRun`
  - Source: `logs/siq/runs/<run_id>.json`
  - Stable ID: `run:<run_id>`
- `Violation`
  - Source: SIQ run `failures[]`
  - Stable ID: `violation:<index>:<rule_id>:<class>`
- `Evidence`
  - Source: SIQ coverage evidence + failure evidence references
  - Stable ID: `evidence:<workspace_relative_path>`
- `Waiver`
  - Source: `logs/siq/waivers_latest.json`
  - Stable ID: `waiver:<waiver_id>`

## Edge Mapping

- `contribution_has_rule`
- `rule_has_run`
- `run_emits_violation`
- `violation_backed_by_evidence`
- `contribution_has_waiver`

## Gateway Surface Alignment

- Contribution graph baseline routes:
  - `/api/kg/spaces/:space_id/contribution-graph/overview`
  - `/api/kg/spaces/:space_id/contribution-graph/graph`
  - `/api/kg/spaces/:space_id/contribution-graph/runs`
- SIQ projections remain filesystem-canonical in this phase and are read-only inputs for graph ingestion.

## Host Policy

No SIQ-driven governance mutation APIs are introduced in this phase.
