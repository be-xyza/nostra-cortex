# Nodepad Reference Mapping: Contribution Semantics and Renderer Hints

**Date**: 2026-04-08
**Status**: Experimental reference mapping only
**Reference Initiative**: `Nodepad Reference Intake`

## Status Snapshot

| Layer | Status | Notes |
| --- | --- | --- |
| Implemented now | Local `cortex-web` experiments | `SpatialHeapGrid` and the layout catalogue consume block `type` and `accent` as renderer inputs only. |
| Experimental proposal | Reference comparison | Nodepad's 14 note types can inform research and UX samples without becoming canonical Nostra ontology. |
| Deferred contract candidate | Governed semantic hint registry | Only consider promotion after multiple live consumers need the same vocabulary and a steward approves the contract. |

## Boundary Rule

- Nostra owns the existence and governance of `Contribution`.
- Cortex may use local renderer hints while experimenting with presentation and layout.
- This file does **not** define canonical platform enums, canonical `metadata.semantic_type`, or shared runtime contracts.
- Layout-family taxonomy and semantic hint taxonomy are separate concerns. This file covers semantic hints only.

## Objective

Compare Nodepad's note taxonomy to current Nostra/Cortex concepts without asserting that the upstream labels are already first-class platform types. The goal is to preserve a useful reference vocabulary for local experiments while avoiding false claims about implemented contracts.

## Reference Mapping

| Nodepad Type | Experimental Interpretation | Accent Hint For Local Web Experiments | Contract Status |
| --- | --- | --- | --- |
| `entity` | Named thing, actor, or concept node | `var(--ui-accent-blue)` | Experimental only |
| `claim` | Asserted statement or proposition | `var(--ui-accent-purple)` | Experimental only |
| `question` | Open inquiry or unresolved prompt | `var(--ui-accent-orange)` | Experimental only |
| `task` | Action item or follow-up work | `var(--ui-accent-green)` | Experimental only |
| `idea` | Generative concept or option | `var(--ui-accent-yellow)` | Experimental only |
| `reference` | External source or citation | `var(--ui-accent-slate)` | Experimental only |
| `quote` | Attributed excerpt | `var(--ui-accent-slate)` | Experimental only |
| `definition` | Explanatory or glossary-style note | `var(--ui-accent-cyan)` | Experimental only |
| `opinion` | Subjective stance or interpretation | `var(--ui-accent-pink)` | Experimental only |
| `reflection` | Retrospective or introspective note | `var(--ui-accent-indigo)` | Experimental only |
| `narrative` | Longer-form prose or story thread | `var(--ui-accent-amber)` | Experimental only |
| `comparison` | Contrast or evaluation between options | `var(--ui-accent-teal)` | Experimental only |
| `general` | Uncategorized fallback | `var(--ui-background)` | Experimental only |
| `thesis` | Synthesized position or promoted takeaway | `var(--ui-accent-gold)` | Experimental only |

## What Is Explicitly Not Claimed

- No `Type::Entity`, `Type::Claim`, or similar enum has been introduced by this document.
- No canonical `Contribution` payload field is established here.
- No A2UI, `ViewSpec`, or `RenderSurface` contract is extended by this document.
- No runtime synthesis workflow is activated by this document.

## Promotion Rule

Only promote any part of this mapping into a governed contract when all of the following are true:

1. At least two live consumers need the same semantic hint vocabulary.
2. The field placement and validation rules are explicitly defined.
3. The boundary remains intact: Nostra owns platform semantics; Cortex owns presentation/runtime behavior.
4. The steward records the promotion in `DECISIONS.md`.
