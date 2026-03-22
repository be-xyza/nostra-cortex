# Eudaemon Alpha — Cross-Initiative Synthesis

> How Eudaemon Alpha integrates with the current Nostra/Cortex stack after the Phase 6 Hetzner resolution.

---

## The Big Picture

Eudaemon Alpha is not a standalone VPS experiment anymore. It is now treated as a governed Phase 6 deployment slice with:

- a **Hetzner VPS**
- a **Rust `cortex-gateway`** as the canonical API surface
- a **Python Eudaemon worker** as the current hosted agent loop
- a **future Rust-native migration path** for Phase 7+

The important synthesis is that Initiative 132 no longer treats the old `ZeroClaw + Hostinger + Docker` framing as current authority.

The newer Doubleword transcript also does not change that authority. Its value is narrower: it offers a useful pattern for recommendation-only cognitive audit loops if those loops are routed through the current heap, lifecycle, workflow, and publication surfaces.

## 1. Workspace and Context: Initiative 124

Eudaemon continues to use the standard Heap runtime rather than a bespoke agent workspace:

- emit blocks with `POST /api/cortex/studio/heap/emit`
- list recent blocks with `GET /api/cortex/studio/heap/blocks`
- package canonical context with `POST /api/cortex/studio/heap/blocks/context`

This keeps exploratory material in the same operator-visible surfaces the steward already uses.

## 2. Governance and Lifecycle: Initiative 126

Initiative 126 remains the hard runtime contract:

- Eudaemon Phase 6 runs at **Authority L1**
- every cycle emits `AgentExecutionRecord`
- production readiness now includes **identity enforcement**, not just activity emission

The Hetzner deployment is therefore not healthy until `agent:eudaemon-alpha-01` is accepted by the gateway with enforcement enabled.

## 3. Durable Execution Semantics: Initiative 047

The execution model is still Temporal-style workflow orchestration:

- deterministic workflow logic
- activities for network, tool, and LLM IO
- durable sleep/wait semantics

Hosting the worker on Hetzner does not relax that contract. It only changes where the transitional runtime is hosted.

## 4. Chronicle and Promotion: Initiative 080

The chronicle remains a DPub-bound surface, but the current Phase 6 cut is intentionally conservative:

- **local chronicle drafting** stays in the agent memory root
- **promotion to Heap/DPub** remains a later governed integration step

This avoids conflating deployment normalization with publication workflow expansion.

## 5. Space Governance: Initiative 130

The steward-facing deployment slice depends on real Space governance state:

- the target Space must exist
- the agent must be a `member`
- the intended `archetype` must be set
- capability surfaces must allow the steward to observe heap activity

This is why the governance bootstrap step is now explicit in the Hetzner runbook.

## 6. Sandbox Boundary: Initiatives 121 and 127

The Hetzner layout preserves the same separation:

- repo checkout under a single deployment root
- sandbox root for constrained code work
- Git-backed memory root for internal traces
- gateway workspace state under host-local `_spaces/`

The move from Hostinger to Hetzner changes the host, not the isolation contract.

## 6.5 Advisory Audit Evidence: Initiative 125

Initiative 125 is the guardrail that keeps transcript-inspired audit work from outranking hard evidence:

- deterministic SIQ checks remain the release and alignment authority
- batch cognition may enrich drift discovery, contradiction spotting, and prioritization
- audit outputs stay advisory until steward-reviewed and linked back into governed artifacts

## 7. Runtime Target: Initiative 122

Initiative 122 still defines the end-state: a Rust-native Cortex runtime. The new resolution is narrower:

- **Phase 6**: Python worker + Rust gateway on Hetzner
- **Phase 7+**: parity-backed migration into Rust-native Cortex execution

That keeps current delivery practical without confusing the prototype host with the long-term platform runtime.

## 8. Cognitive Audit Pipeline: Initiatives 133 and 134

The transcript's strongest reusable pattern is a split pipeline:

- Cortex executes extraction, batch submission, polling, and artifact routing
- external batch cognition performs large-scale parallel analysis
- Eudaemon re-enters for synthesis, contradiction review, and recommendation drafting
- Nostra retains publication and governance authority

Grounded to current repo state, this means:

- no direct graph mutation from batch output
- no replacement of workflow authority with provider-specific batch jobs
- no bypass of heap-first publication, proposal gating, or chronicle lineage
- meta-evaluation and scoring belong with Initiative 133
- durable orchestration and adapter boundaries belong with Initiative 134

## 8.5 Live Provider Boundary

Phase 6 still needs a primary live cognition path separate from any later batch audit overlay:

- the main steward-facing loop uses low-latency direct model invocation
- provider selection belongs to the Eudaemon-side adapter boundary, not to Nostra authority
- `api_key` and `codex_subscription` are acceptable live-lane categories
- latest ZeroClaw is relevant only as a possible sidecar/profile broker for Codex subscription auth
- ChatGPT Pro does not become generic worker billing; it matters only if the Codex subscription path is explicitly adopted
- Doubleword remains the stronger fit for the secondary batch audit lane because it is API-key based and batch-oriented

## 9. Model Constitution: Initiative 062

Prompting and disclosure remain governed locally:

- local prompt file remains the fallback
- Heap `prompt_override` remains the primary override path
- LLM responses must be treated as inputs to governed analysis, not as upstream authority

## Summary of Alignments

| Domain | Phase 6 Resolution | Governing Initiative |
|--------|--------------------|----------------------|
| Workspace | Heap blocks and context bundles | 124 |
| Authority | L1 plus production identity enforcement | 126 |
| Runtime model | Temporal-style workflow plus activity separation | 047 |
| Hosted runtime | Python worker on Hetzner, Rust gateway local to host | 132 / 122 transition |
| Memory | Git-backed host-local memory root | 121 |
| Ingestion | Sandbox-root constrained search and tooling | 127 |
| Chronicle | Local draft now, governed promotion later | 080 |
| Space rules | Membership, archetype, capability governance | 130 |

## Next Steps

1. Bring the Hetzner box up with the canonical env contract and `systemd` services.
2. Bootstrap actor and space registry state for `agent:eudaemon-alpha-01`.
3. Complete one end-to-end solicitation-to-proposal cycle with production auth settings.
4. Use that parity slice as the baseline for future Rust-native migration work.
5. Treat any future cognitive audit pipeline as a governed execution slice, not as a new authority plane.
6. Keep the live provider lane deployment-ready before investing in batch audit throughput or subscription-sidecar expansions.
