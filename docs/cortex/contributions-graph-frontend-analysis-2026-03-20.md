# Contributions Graph Frontend Analysis

Date: 2026-03-20
Scope: `cortex-web` contribution graph handling for steward workflows

## Current Handling

The frontend currently uses three graph patterns with very different jobs:

1. [AmbientGraphBackground.tsx](/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/heap/AmbientGraphBackground.tsx)
   - Full-space contribution graph.
   - Visual atmosphere only.
   - No interaction by design.

2. [ForceGraph.tsx](/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/ForceGraph.tsx)
   - Generic D3 force layout over raw contribution graph nodes and edges.
   - Click-to-select, but low semantic guidance.
   - Better for exploratory browsing than for stewardship.

3. [ContributionsWorkbenchHost.tsx](/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/contributions/ContributionsWorkbenchHost.tsx)
   - Steward cockpit for governed action.
   - Previously depended on an embedded backend summary surface.
   - Now renders graph history and contribution focus directly in the host.

## Simplicity Assessment

The old cockpit summary path was too indirect:

- the steward route depended on a nested backend A2UI summary
- when that backend surface lagged, the operator saw placeholder copy inside a route that was otherwise already complete
- graph context was presented as tables and blast-radius lists, but not as a focused interaction model

That was misaligned with the actual steward task. Operators rarely need a full force-directed graph first. They need:

- a current focus
- direct relations around that focus
- a short path to change focus
- clear distinction between graph-run history and live agent execution

## Principles Alignment

The better boundary for this surface is:

- Nostra defines the relation types and contribution identity.
- Cortex renders a steward-appropriate projection of those governed relations.

That means the contribution cockpit should prefer:

- deterministic one-hop projections over animated full-graph exploration
- relation semantics over physics simulation
- direct actionability over visual spectacle

The new [ContributionFocusMap.tsx](/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/contributions/ContributionFocusMap.tsx) moves in that direction:

- one focused contribution at the center
- relation lanes grouped by governed edge meaning
- each related contribution is directly focusable
- no canvas dependency
- keyboard- and screen-reader-friendly controls

## Enrichment Opportunities

### High Value

1. Add relation provenance in the focus map.
   - Show whether an edge came from graph seed, steward action, simulation, or edition diff.

2. Connect graph runs to contribution focus when possible.
   - If a graph run artifact includes touched contributions, prefill the focus map instead of stopping at a run ID.

3. Add steward explanations beside each relation lane.
   - Example: “Invalidates” should explain what would need re-review if this contribution changes.

4. Add compact path overlays.
   - When a goal is present, show “why this node matters to the selected goal” rather than just adjacency.

### Medium Value

5. Add relation filtering in the cockpit.
   - Toggle between structural relations, reference relations, and invalidation relations.

6. Add small run provenance badges to graph history rows.
   - Show whether a run was validate-only, simulation, doctor, or publish oriented at a glance.

7. Add local cache and stale-state labeling.
   - Distinguish “no relations exist” from “graph artifact missing” from “graph data seeded but not yet refreshed.”

### Lower Value

8. Revisit full force-graph exploration only as a secondary drill-down.
   - Keep it out of the primary steward loop.
   - Use it for exploration or investigations, not for default contribution operations.

## Recommendation

Keep the cockpit on the focused-map path and treat full-graph force layouts as secondary exploration tools. The steward route should optimize for governed decision-making, not general graph browsing.
