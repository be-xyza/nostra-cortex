# Contribution Graph Naming Contract

## Purpose
Define a single canonical naming model for graph semantics across Nostra/Cortex so code, routes, artifacts, and contracts remain unambiguous.

## Canonical Terms
- Graph container: `ContributionGraph`
- Graph API route segment: `contribution-graph`
- Graph artifact filename: `contribution_graph.json`
- Graph schema id: `nostra.contribution_graph.v1`
- Node id payload field: `contributionId`

## Allowed Initiative Usage
- `initiative` is only allowed as a `ContributionKind` node subtype value.
- `initiative` may appear in historical research content and archived artifacts.

## Forbidden Graph-Level Terms
- `initiative-graph`
- `initiative_graph`
- `InitiativeGraph`
- `InitiativeNode`
- `InitiativeEdge`
- `initiativeId`
- `initiative_id`

## Contract Surfaces
- Rust extraction/domain/gateway types must use `Contribution*` graph naming.
- TypeScript contracts and API clients must use `Contribution*` graph naming.
- Gateway endpoints must use `/contribution-graph/*` paths.
- Workbench graph route id must use `/system/contribution-graph`.
- Primary navigation authority must remain `/api/cortex/layout/spec` + `/api/spaces/:space_id/navigation-plan`; graph endpoints are metadata surfaces, not nav discovery authority.

## Legacy Aliases (Retired)
- Active code must not expose or depend on `/api/kg/spaces/:space_id/initiative-graph/*`.
- If legacy aliases are discovered in a host surface, treat as drift and migrate to `/api/kg/spaces/:space_id/contribution-graph/*`.
- Historical aliases may remain only in archived research or narrow metadata-compatibility bridges; they must not drive route discovery, nav ordering, or shell placement.

## Enforcement
- Run `bash scripts/check_contribution_graph_naming_contract.sh`.
- Run gateway parity and protocol coverage checks after endpoint changes.
