# Semantic Navigation Slots (Governed UX Contract)

## Purpose
Ensure new Workbench capabilities land in the correct sidebar location without frontend hardcoding or manual ordering, while preserving Initiative 123 principles:

- Navigation is **contract-driven** (governed UX contracts), not route scanning.
- Ordering is **deterministic** for the same contract state.
- Structural updates are **steward-gated** with approval metadata.
- Discovery is **Labs-only** until promoted.

## Contract Field
`ShellLayoutSpec.navigationGraph.entries[]` supports:

- `navSlot?: string`

This is authored in the persisted layout contract and projected via:

- `GET /api/cortex/layout/spec`
- `GET /api/spaces/:space_id/navigation-plan`

## Slot Vocabulary
Valid values:

- `primary_attention` (Inbox, Notifications)
- `primary_workspace` (Spaces, Heap)
- `primary_execute` (Workflows)
- `secondary_build` (Studio, Artifacts, VFS)
- `secondary_ops` (System, Metrics, SIQ, Testing, Logs)
- `secondary_agents` (Agents, Discovery, Memory, Simulation)
- `secondary_admin` (Settings)
- `labs` (Labs-only navigation)
- `hidden` (never shown in sidebar; still routable)

If `navSlot` is missing, the compiler infers a slot deterministically from route and metadata.

## Ranking Rules (Deterministic)
`/api/spaces/:space_id/navigation-plan` assigns `rank` using:

1. Slot weight (`navSlot`)
2. Existing surfacing/frequency heuristics
3. Stable tie-breakers (category, label, routeId, capabilityId)

Slot weights (higher ranks earlier):

- `primary_attention`: 900
- `primary_workspace`: 850
- `primary_execute`: 800
- `secondary_ops`: 600
- `secondary_build`: 550
- `secondary_agents`: 500
- `secondary_admin`: 450
- `labs`: 200
- `hidden`: 0 (filtered)

## Governed Discovery (Labs lane)
Capabilities present in `view_capabilities` but absent from `layoutSpec.navigationGraph.entries` are treated as **unpromoted**.

- `GET /api/cortex/layout/discovery` returns unpromoted routes with a recommended `navSlot` and rationale.
- The Workbench `/labs` surface renders this discovery table for operator/steward review.

## Promotion Workflow (Steward-gated)
1. Review unpromoted capabilities via `/labs` → “Navigation Discovery”.
2. Promote by updating the persisted shell contract via:
   - `POST /api/cortex/layout/spec`
3. Contract updates require:
   - **steward role**
   - `navigation_contract.approved_by` and `navigation_contract.rationale` (non-empty)
   - valid `navSlot` values
4. Every accepted contract update is audited to decision surface logs.
