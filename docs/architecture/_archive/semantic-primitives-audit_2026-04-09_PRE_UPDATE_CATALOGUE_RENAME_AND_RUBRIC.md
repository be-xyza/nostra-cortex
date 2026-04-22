# Semantic Primitives Audit

Date: 2026-04-09
Status: Draft
Authority Mode: recommendation_only

## Purpose

Catalog current high-signal terms by semantic status so the repo can critique
drift before new primitives become canonical by accident.

## Canonical Terms

| Term | Layer | Rationale |
|---|---|---|
| `space` | Nostra | Stable user-facing container term with explicit non-workspace rule. |
| `workspace` | Cortex developer layer | Structural code/operator term only; must not surface to users. |
| `workbench` | Cortex | Canonical execution shell concept. |
| `steward` | Nostra | Canonical caretaker role for governance, continuity, and escalation. |
| `labs` | Cortex | Canonical scope marker for experimental surfaces; not a maturity label. |
| `contribution graph` | Nostra | Canonical graph naming; `initiative graph` is retired. |
| `notes`, `tasks`, `plans`, `initiatives` | Mixed | Work primitive split already governed by decision log. |

## Experimental or Local Terms

| Term | Current Status | Notes |
|---|---|---|
| `catalogue` | Experimental | Preferred local name for the current layout comparison surface. |
| `layout family` | Experimental | Cortex-local comparison vocabulary only; not a shared contract. |
| `lane_board`, `spatial_bsp`, `force_graph`, `time_indexed` | Experimental | Local topology vocabulary for current layout experiments. |

## Ambiguous or Reserved Terms

| Term | Concern | Planned Direction |
|---|---|---|
| `gallery` | Strong user expectation of browsing curated artifacts or media. Current layout use risks future collision. | Reserve for a future user-facing browsing or collection concept. Treat current layout use as temporary naming debt. |

## Deprecated Terms

| Term | Replacement | Notes |
|---|---|---|
| `mayor` | `steward`, `operator`, or a layer-true replacement | Deprecated in role semantics doctrine. |
| `initiative-graph` | `contribution graph` | Retired by graph naming decision. |

## Current Layout Surface Note

The current `/gallery` route in `cortex-web` remains live in this phase for
compatibility, but it should be interpreted as an experimental `catalogue`
surface rather than a canonical `gallery` concept.

## Next Review Targets

1. User-facing navigation and shell labels in `cortex-web`
2. Work primitive terminology in docs and runtime descriptions
3. Remaining historical civic-role terminology in canonical docs
