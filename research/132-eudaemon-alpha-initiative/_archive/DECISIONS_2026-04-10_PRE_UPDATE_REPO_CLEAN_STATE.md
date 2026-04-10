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

## 2026-03-19 — Native Live Cognition Precedes Subscription or Batch Extensions

**Decision**

Phase 6 cognition defaults to a native live provider lane first, with explicit provider boundaries for `api_key` and `codex_subscription` paths. Advisory batch cognition remains secondary.

**Why**

- The current worker loop, benchmark path, and steward-feedback flow are all low-latency and request/response oriented.
- A live lane keeps Phase 6 deployment aligned to the real heap, lifecycle, and workflow surfaces that already exist in the repo.
- Codex subscription access may be useful, but only as an explicit sidecar/profile adapter path rather than an implicit "ChatGPT Pro credits" shortcut.

**Consequences**

- ChatGPT Pro is not treated as a generic API-credit source for Eudaemon.
- Any Codex subscription integration must remain an auth/provider adapter, not a workflow or governance authority surface.
- Doubleword-style batch inference stays behind typed audit manifests and source-linked synthesis work.

## 2026-03-27 — VPS Deploy Authority Moves to Operator-Local SSH Promotion

**Decision**

Phase 6 production deployment authority moves out of GitHub Actions and into an operator-local SSH promotion flow rooted in [`scripts/promote_eudaemon_alpha_vps.sh`](/Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh), with [`ops/hetzner/deploy.sh`](/Users/xaoj/ICP/ops/hetzner/deploy.sh) and [`scripts/check_vps_runtime_authority.sh`](/Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh) enforcing the host-local runtime contract.

**Why**

- Production runtime mutation is a sensitive Cortex execution action and should remain steward-owned rather than hidden behind GitHub secrets.
- The VPS agent needs one legible authority chain for both analysis and deploy verification: repo mirror plus runtime manifest.
- Commit-to-runtime provenance is clearer when the promoted commit, on-host checkout, rendered units, and manifest all converge on the same host-local files.

**Consequences**

- GitHub `main` no longer deploys directly to the VPS; it reports promotability only after required gates pass.
- Operators promote or roll back by redeploying an explicit commit through the same governed local command.
- `/srv/nostra/eudaemon-alpha/repo` and `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json` become the primary host analysis authority surfaces.
- `cortex-web` remains explicitly `not_deployed` on the VPS until a later governed change updates the contract.

## 2026-03-31 — Phase 7 Uses Parity Slices, Typed Boundary Hardening, and Executor-Specific Sandboxing

**Decision**

Initiative 132 adopts a selective subset of the validated external agent-runtime patterns: Phase 7 proceeds as a parity-backed Rust hardening program, not as a wholesale replatform. Strict DTO hardening moves forward now on exposed Gateway and A2UI boundaries, while OS-level sandboxing is required for executor slices that run untrusted code or broader autonomous contribution paths rather than being treated as an immediate whole-runtime replacement for the current Hetzner `systemd` model.

**Why**

- The current repo already validates the long-term Rust-native direction through Initiative 122 and the governed execution boundaries in Initiatives 126 and 134.
- The current deployment authority model is still explicitly operator-local SSH plus `systemd`, and that contract is already governed and verified on the repo side.
- The strongest present engineering pressure is not the absence of Rust crates in general, but the breadth of the current `cortex-eudaemon` application slice and its gateway/service surfaces.
- External sandboxing patterns are useful reference points, but the current local threat model only justifies executor-specific OS isolation once untrusted code execution becomes a real capability.

**Consequences**

- Initiative 132 must not describe Phase 7 as replacing the current Phase 6 Hetzner runtime model before Rust parity is demonstrated.
- Strict typed DTO and contract-sync work becomes the immediate hardening priority on networked boundaries.
- Extraction work should follow existing workspace seams in `cortex-domain`, `cortex-runtime`, `cortex-ic-adapter`, and `cortex-eudaemon` rather than inventing a new umbrella core crate.
- The first recommended extraction seams are the provider runtime surface, the ACP / terminal execution-control surface, and the workbench UX / heap projection surface.
- OS-level sandboxing is promoted to a requirement for executor-specific untrusted code paths and later institutional execution stages, not to a blanket requirement for the entire gateway/runtime stack today.

## 2026-03-31 — Provider Runtime Discovery Moves Behind a Shared Provider-Runtime Boundary

**Decision**

The provider-admin discovery and runtime-status paths now depend on a shared provider-runtime discovery module rather than keeping the discovery parser, client envelope resolver, and live-discovery aggregation logic inside `gateway/server.rs`.

**Why**

- The provider-admin routes were still reaching into the gateway for discovery-oriented provider-runtime helpers, which kept the gateway too fat for the intended Batch 1 boundary.
- The shared discovery module can own the provider-runtime client configuration, supported-model parsing, and live-discovery assembly without changing route wiring or operator authorization behavior.
- The gateway still retains the operator-only host/discovery surfaces and the generic provider record shaping helpers that are not yet part of this extraction seam.

**Consequences**

- `cortex-eudaemon` now has a narrower provider-runtime import surface in the gateway/provider-admin path.
- The provider-runtime discovery parser is shared by the gateway tests and the discovery module, reducing drift between the runtime and the gateway validation path.
- The gateway still owns several provider-record presentation helpers, so this is a batch-boundary extraction rather than a full provider-runtime crate split.
- The next extraction step can focus on the remaining provider-runtime presentation/binding helpers only if the current Batch 1 seam stays green.

## 2026-03-31 — Batch 0A Contract Hardening Holds Before Batch 1 Extraction

**Decision**

Batch 0A contract hardening is accepted as implementation progress, but Initiative 132 remains held in Batch 0A rather than advancing into Batch 1 extraction.

**Why**

- The touched Batch 0A contract slices now have stronger evidence:
  - heap emission now normalizes the public `space_id` field while keeping `workspace_id` as an explicit compatibility alias where needed
  - provider-runtime SSE handling now uses typed envelopes for deltas and tool-call extraction instead of depending only on ad hoc raw-value matching
  - lifecycle payload expectations are covered by explicit `camelCase` serialization tests
  - ACP params now reject silent snake_case alias drift rather than accepting ignored unknown fields
- The broader parity doctrine is still unmet because `gateway_parity` remains red on pre-existing fixture inventory drift that predates the Batch 0A edits.
- Initiative 132's own progression rule says extraction should not advance while parity is failing, even if the newly touched surfaces themselves are green.

**Consequences**

- Provider-runtime extraction should not start yet, even though its boundary is clearer than before Batch 0A.
- The next required step is to repair the `gateway_parity` fixture inventory debt and rerun that suite cleanly.
- ACP remains a valid future extraction seam, but the current work stays in contract hardening and parity recovery rather than broad execution-control movement.
- Workbench and heap projection compatibility around `space_id`, `workspace_id`, and `workspaceId` remains a known follow-up concern to narrow before extraction begins.

## 2026-04-01 — Batch 1 Narrows Provider Runtime Discovery and Auth-Binding State Behind Shared Boundaries

**Decision**

Initiative 132 accepts the current Batch 1 provider-runtime extraction slice as stage-valid progress. Remote SSH runtime-host discovery now lives in `provider_runtime::discovery`, and provider-admin auth-binding resolution/state helpers now live in `gateway::provider_admin`, leaving `gateway/server.rs` with thin operator-route wrappers instead of duplicated provider-runtime helper logic.

**Why**

- The most obvious remaining provider-runtime bloat in the gateway was the remote host probe pipeline and the provider-admin auth-binding fallback helpers.
- Those behaviors are runtime- and provider-admin-specific, not generic gateway responsibilities.
- The extracted seams already had governed evidence available: provider-route behavior tests, execution-binding rejection tests, and `gateway_parity`.
- Keeping ACP and workbench extraction deferred preserves the phase-ordering rule that Batch 2 should not be smuggled into Batch 1 simply because the provider-runtime seam stayed green.

**Consequences**

- The provider-runtime surface now has a smaller import and behavior footprint inside `gateway/server.rs`.
- Provider discovery merge behavior, host-scoped provider identity synthesis, and probe-key precedence are now tested closer to the modules that own them.
- Batch 1 can be considered materially advanced for the current stage, but Batch 2 ACP / terminal extraction remains deferred until it is started as its own governed batch.
- The current stage closeout should reference [`BATCH1_DECISION_GATE.md`](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/BATCH1_DECISION_GATE.md) as the authoritative evidence record for this extraction slice.
