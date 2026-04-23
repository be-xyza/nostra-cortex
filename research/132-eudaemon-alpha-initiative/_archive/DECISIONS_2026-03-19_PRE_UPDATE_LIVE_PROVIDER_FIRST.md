# Initiative 132 Decisions

## 2026-03-18 — Phase 6 Hetzner Runtime Resolution

**Decision**

Phase 6 deploys Eudaemon Alpha as a Python worker on a Hetzner VPS with the Rust `cortex-gateway` running on the same host as the canonical local API surface.

**Why**

- The Python worker is the most testable and deployment-ready loop in the current repo.
- The Rust gateway already exposes the canonical heap and identity surfaces needed for live validation.
- This keeps the delivery slice operational without pretending the Rust-native runtime migration is already complete.

**Consequences**

- Hostinger and Docker are no longer the active deployment assumptions in Initiative 132 docs.
- Phase 6 production auth must disable dev-mode role bypasses and enable agent identity enforcement.
- Linux `systemd` assets, a Hetzner runbook, and a governance bootstrap step become required operational surfaces.
- Rust-native `cortex-eudaemon` remains the migration target for a later parity-backed phase, not the Phase 6 primary runtime.

## 2026-03-18 — Eudaemon Alpha Companion Repo Boundary

**Decision**

Initiative 132 remains authoritative in the root ICP repo, while the Python Eudaemon Alpha worker moves into a companion implementation repo attached back to the root repo as the `eudaemon-alpha` submodule.

**Why**

- The root repo should remain the governance and architecture source of truth.
- The Python worker is transitional implementation surface, not the long-term platform authority.
- A submodule preserves a pinned revision from the root repo while keeping the implementation boundary clean.

**Consequences**

- Root docs and Hetzner guidance must refer to `eudaemon-alpha/` as a submodule-owned path.
- Agent-owned service units and bootstrap tooling move under the companion repo.
- Root deployment flows must use `git clone --recurse-submodules`.

## 2026-03-19 — Doubleword Batch Cognition Is Advisory and Eudaemon Is the Synthesizer

**Decision**

Adopt the Doubleword transcript as a pattern source for a recommendation-only Cognitive Audit Pipeline, with Eudaemon Alpha acting as the architect and synthesis agent rather than the primary batch analyzer.

**Why**

- The transcript's extractor -> batch cognition -> scoring -> synthesis split aligns with the existing heap, lifecycle, workflow, and publication surfaces.
- The current repo already exposes the endpoints and runtime artifacts needed to publish advisory findings without granting direct mutation authority.
- Keeping Eudaemon in the design and synthesis role preserves Nostra/Cortex boundary discipline and avoids conflating external batch output with local governance truth.

**Consequences**

- Any batch-cognition backend must be treated as an execution adapter or activity behind Initiative 134, not as a workflow authority source.
- Deterministic SIQ gates under Initiative 125 remain authoritative for release and alignment checks.
- Audit outputs publish first as heap blocks, proposals, closeout follow-through, workflow drafts, or chronicle drafts.
- Core-graph bootstrap ideas from the transcript remain semantic discovery input only until a governed Nostra authority path is defined.
