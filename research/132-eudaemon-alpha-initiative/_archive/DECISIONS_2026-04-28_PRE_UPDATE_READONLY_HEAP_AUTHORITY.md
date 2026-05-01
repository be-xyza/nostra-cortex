# Initiative 132 Decisions

## 2026-04-28 — Observe-Once Worker Proof Passed on VPS

**Decision**

Record Eudaemon Alpha observe-once worker validation as passed on the VPS at commit `a8af1afe312b521ab3d448b15716d9d6fd219312`.

The proof is captured in [`evidence/20260428T080222Z_vps_observe_once_worker_proof.md`](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/20260428T080222Z_vps_observe_once_worker_proof.md). The worker ran one explicit `NOSTRA_WORKER_OBSERVE_ONCE=1` pass as the `nostra` service user, read only `/api/system/whoami` through the loopback gateway, wrote a local observation artifact, reported production-auth posture as `authzDevMode=false`, `allowUnverifiedRoleHeader=false`, `agentIdentityEnforcement=true`, and exited with `exitStatus=pass`.

**Why**

- The observe-once authority packet needed live proof that the worker could operate under production identity without enabling polling or mutation.
- The proof confirms the worker can observe its runtime posture through the gateway while preserving the Cortex execution-infrastructure boundary.
- Capturing the result in governed records closes the Hermes/records-style sync gap for this runtime milestone.

**Consequences**

- Initiative 132 has now passed passive preflight, production-auth posture, host-mode authority, and bounded observe-once validation.
- Live polling, autonomous task selection, provider execution, heap/proposal/workflow emission, runtime mutation, repo mutation, graph mutation, and untrusted execution remain unauthorized.
- The next runtime-expansion packet should be narrower than execution: bounded read-only heap/context visibility or steward-reviewed local evidence projection.

## 2026-04-28 — Observe-Once Is the Next Runtime Expansion Gate

**Decision**

Adopt an Initiative 132 runtime-expansion authority packet for a proposed observational one-shot `cortex_worker` mode before enabling live polling or execution.

The packet is [`RUNTIME_EXPANSION_AUTHORITY_PACKET_OBSERVE_ONCE.md`](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/RUNTIME_EXPANSION_AUTHORITY_PACKET_OBSERVE_ONCE.md). It defines a narrow worker slice that may authenticate to the loopback gateway, read only explicitly configured self-observation endpoints, write one local observation artifact under deployment state/logs, and exit. It forbids provider calls, autonomous task selection, repo mutation, runtime mutation, graph mutation, heap/proposal/workflow emission, untrusted shell/code execution, and default-readable runtime topology inspection.

**Why**

- Passive worker preflight and production auth are now proven, so the next risk is not buildability; it is authority scope.
- A one-shot observation mode proves the worker can operate under production identity without granting it a recurring queue, provider lane, or mutation surface.
- Keeping the first output local preserves the distinction between runtime evidence and governed Initiative 132 records until the operator promotes evidence manually.

**Consequences**

- The next implementation PR should add an explicit opt-in flag such as `NOSTRA_WORKER_OBSERVE_ONCE=1`, preserving passive default behavior.
- Success of observe-once does not authorize live polling, task execution, provider execution, heap emission, proposal creation, or graph mutation.
- Any later expansion requires a separate packet describing allowed reads, writes, identity requirements, artifacts, rollback, and forbidden actions.

## 2026-04-28 — VPS Passive Runtime Production Auth Proof Passed

**Decision**

Record Eudaemon Alpha as validated for passive VPS runtime with production-auth posture at commit `2cfbf65dbe2093666de443366d33626b1c325090`.

This updates the earlier blocker state: host-mode VPS authority and production identity/auth posture are no longer blockers for passive runtime. The worker remains passive-preflight only, and live polling or execution authority still requires a separate governed runtime-expansion decision.

**Why**

- The VPS was promoted to the merged deploy/auth hardening commit.
- `cortex-gateway.service` and `cortex-worker.service` were active after promotion.
- `scripts/check_vps_runtime_authority.sh` passed on the host and now validates production auth flags.
- Live gateway checks showed unverified operator headers are rejected, a bound operator principal is accepted, and unknown agent IDs are rejected.
- The worker logs confirm passive mode remains in force rather than silently enabling runtime polling.

**Consequences**

- Initiative 132 may proceed from build/preflight and host-auth proof into scoped runtime-expansion design.
- Any execution expansion must explicitly decide what the worker may read, emit, queue, or mutate.
- Provider execution, autonomous task selection, repo/runtime mutation, production graph mutation, and untrusted shell/code execution remain forbidden until separately governed.
- The next safe technical step is to define a minimal passive-to-observational worker slice rather than enabling broad execution.

## 2026-04-27 — Worker Build Is Passive-Preflight Only and Hermes Records Need Explicit Sync

**Decision**

Record PR #69 as a build/preflight restoration for `nostra/worker`, not as live Eudaemon Alpha runtime readiness. The worker may build, load configuration, manage HPKE key material, and run in one-shot passive preflight mode, but runtime polling and production execution claims remain blocked until `agent:eudaemon-alpha-01` is proven under production identity enforcement and host-mode VPS authority validation passes.

Also add an explicit Hermes record-sync gate for Initiative 132: local Hermes artifacts under `/Users/xaoj/hermes` and profile state under `~/.hermes` do not automatically update governed research records. Any new Hermes profile, schema, runbook, task-packet class, developer handoff lane, stabilization change, or role-boundary correction must be either promoted into Initiative 132 evidence/DECISIONS/PLAN updates or explicitly marked local-only with no authority claim.

**Why**

- PR #69 fixes the `cortex_worker` build blocker, but the implementation intentionally disables live polling to avoid overstating runtime authority.
- The recent `hermescortexdev` work exposed a sync gap: local Hermes profiles and handoff artifacts can change faster than the governed research initiative records.
- Initiative 132 treats root `ICP` research records as authority; local Hermes state is useful operator material, not durable governance unless promoted.

**Consequences**

- Eudaemon Alpha is no longer blocked at the local worker build/preflight layer once PR #69 merges, but it is still blocked at the live runtime authority layer.
- `hermes132` remains the advisory synthesis profile; `hermescortexdev` remains patch-prep only and must not mutate repo/runtime state.
- A Hermes output may inform Codex/operator work only when its local-only or promoted-evidence status is clear.
- Future Hermes-related sessions should include a closeout question: does this artifact need Initiative 132 evidence, a DECISIONS/PLAN update, a heap/proposal projection, or no promotion?

## 2026-04-25 — Hermes Green Means Local Advisory Review and Lineage Only

**Decision**

Promote the Hermes stage to `green` only for bounded local advisory review and lineage.

This green status covers the validated local envelope: bounded Hermes synthesis, source-packet/audit-unit discipline, operator-mediated Cortex heap visibility, local heap projection, and steward feedback lineage. It does not authorize production identity readiness, production authorization, ICP evidence promotion beyond the recorded evidence note, provider execution, subagents, skill activation, memory authority, unattended execution, or Hermes acting as a workflow authority.

**Why**

- The Hermes lane sweep completed more than the required five clean bounded passes.
- Local stabilization sub-signals are green.
- The local heap emission pilot validated contract shape, heap ingestion, read-back, and steward feedback lineage.
- Cortex Web review surfaces were hardened enough for the bounded local advisory path, including explicit review outcome semantics, decision receipts, and space-aware artifact navigation.
- The operator explicitly authorized promotion with the scope: `bounded local advisory review and lineage only`.

**Consequences**

- `/Users/xaoj/hermes/stabilization/initiative-132-hermes-stage-stabilization.v1.json` now records overall `status: "green"` with a bounded `statusScope`.
- The promotion evidence lives at `/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/20260425T054737Z_hermes_green_local_advisory_review_and_lineage.md`.
- Production-auth heap emission, production role proof, ICP evidence promotion beyond this note, and any execution expansion remain separate governed gates.

## 2026-04-23 — Hermes Uses Source Packets and Existing Cortex Visibility Surfaces

**Decision**

Adopt `HermesSourcePacketV1` as a local planning/runtime input for bounded Hermes passes when deterministic excerpts or validated facts are preferable to direct file inspection. Also adopt the existing Cortex Web heap, A2UI approval, steward-gate, viewspec proposal, workflow-draft proposal, and agent-contribution approval surfaces as the intended future visibility and approval route for Hermes outputs.

Hermes does not get a bespoke UI control plane in this stage. It remains a local observer whose outputs may later be projected into those existing Cortex surfaces after separate steward review.

**Why**

- The first runbook validation showed that direct Hermes file inspection can drift into shell/code-style behavior even when the pass only needs bounded facts.
- The repo already contains the user-visibility and approval primitives we want: heap solicitations, steward feedback, A2UI approval tracking, steward-gate validation/apply, and proposal review routes.
- Reusing existing Cortex visibility surfaces keeps Hermes aligned with current architecture instead of inventing another authority lane.

**Consequences**

- Source packets become the preferred bounded excerpt/fact layer when they are sufficient for a pass.
- Source packets are local inputs only; if they deserve durable lineage, their content should be promoted into governed evidence, heap artifacts, or Nostra-native contributions through normal review.
- Hermes visibility work should target projection into heap/proposal/approval surfaces, not direct execution, approval bypass, or a new standalone runtime UI.

## 2026-04-23 — Hermes Stage Stabilization Must Stay Explicit

**Decision**

Track Hermes readiness for Initiative 132 through explicit green/yellow/red stabilization signals rather than intuition. The stage remains active only while Hermes continues to provide bounded architectural value with acceptable operator overhead and no authority confusion.

**Why**

- The current envelope is useful, but it is still a staged advisory system rather than a mature platform primitive.
- We need legible criteria for when to tighten Hermes, when to expand its role, and when to retire it after its core jobs are absorbed elsewhere.

**Consequences**

- `/Users/xaoj/hermes/stabilization/` may contain local stage-status artifacts describing update, upgrade, and deprecate thresholds.
- A clean bounded pass count, guardrail reliability, input discipline, user visibility, and net architectural value are now first-class stage signals.
- Hermes should be deprecated for this stage if Cortex-native primitives absorb the same observation/synthesis job more directly or if the operator burden remains higher than the value returned.

## 2026-04-23 — Hermes Runbooks Automate Ritual, Not Agency

**Decision**

Add a local `HermesAuditRunbookV1` planning contract under `/Users/xaoj/hermes` to standardize the operator-mediated ritual around Hermes advisory passes: preflight validation, one bounded Hermes pass, postflight validation, optional evidence drafting, and manual promotion.

This runbook may generate or reuse a bounded prompt template, but it does not enable unattended execution, schedules, webhooks, subagents, MCP connectors, provider jobs, browser automation, code execution, skill installation, repository mutation, runtime mutation, or execution adapters.

**Why**

- The first activation and capability-discovery passes proved that the envelope is useful but still too manual for repeatable use.
- Automating the ritual reduces operator error while preserving the current governance boundary.
- Initiative 132 needs repeatable advisory passes before any future Initiative 134 adapter work is justified.

**Consequences**

- Hermes runbooks are local planning artifacts, not public Cortex APIs or workflow authority.
- The operator remains responsible for preflight, postflight, evidence promotion, commits, and pushes.
- Any future unattended scheduler, background worker, or runtime adapter remains separate Initiative 134 work.

## 2026-04-23 — Hermes Capability Discovery Envelope Is Planning-Only

**Decision**

Add a second local Hermes planning envelope beside the strict activation lane: the Hermes Capability & Discovery Envelope. This envelope may classify Hermes features, name observer lanes, design capability-discovery source manifests, and draft skill-improvement proposals, but it remains read-only, one-pass, source-linked, local, and recommendation-only.

The useful content from the original Hermes analysis is adopted as observer pipeline design: system cartography, component/dependency mapping, boundary integrity review, pattern extraction, adapter translation, test synthesis, skill architecture, capability inventory, event/lifecycle logging, memory/continuity posture, batch design/aggregation, and final synthesis. It is not adopted as an action-loop plan, runtime-authority plan, or permission to activate external execution behavior.

**Why**

- The first Hermes activation pass proved the strict advisory envelope, but it intentionally suppressed broader Hermes capabilities that should still be inventoried for realistic expectations.
- The original analysis contains useful lane and pipeline structure, especially adapter translation and pattern extraction, without requiring external action-loop activation.
- Initiative 133 already identifies a skill-architecture gap: `nostra-cortex-dev-core` should remain a lean governance gate, while platform knowledge should move into a progressive-disclosure companion proposal.

**Consequences**

- `/Users/xaoj/hermes` may contain local planning contracts for `HermesLaneCatalogV1`, `HermesCapabilityMatrixV1`, and `SkillImprovementProposalV1`.
- Capability discovery may classify Hermes skills, memory, session search, cron/webhooks, MCP, subagents, gateway delivery, terminal tooling, and batch-runner concepts, but it must not enable them for Initiative 132.
- The selected skill strategy is hybrid companion: keep `nostra-cortex-dev-core` as the governance/preflight skill and draft a `nostra-platform-knowledge` proposal with eval prompts before any registry change.
- Any future live scheduler, execution adapter, distributed execution layer, provider job control, or Cortex-native workflow integration remains separate Initiative 134 work.

## 2026-04-23 — Hermes Activation Uses Root ICP Authority and Workspace Guardrails

**Decision**

Activate Hermes only after promoting the approved Initiative 132 Hermes language into root `ICP`, and treat root `ICP` as the sole governed authority source for Hermes activation. Hermes activates from `/Users/xaoj/hermes` through a workspace-local `.hermes.md` guardrail file and operates only as a local advisory meta-observer.

Hermes may use normal advisory inference to reason over explicit `SourceManifestV1` and `AuditUnitV1` inputs, but each session must remain one bounded, deterministic, auditable pass that emits one session record, zero or more source-linked findings, and one synthesis artifact. `~/.hermes` config, SOUL, and profiles remain non-authoritative convenience state and must not redefine Initiative 132 governance boundaries.

**Why**

- Root `ICP` is the durable institutional authority surface; the request worktree is staging only.
- The strongest surviving value from the original Hermes analysis is the meta-observer role plus synthesis discipline, not the earlier external action-loop framing.
- Workspace-local context files are the most reliable place to enforce activation guardrails, whereas generic runtime profiles are designed for ergonomics rather than governance.

**Consequences**

- Hermes source bundles must resolve only promoted root `ICP` authority files.
- `/Users/xaoj/hermes/.hermes.md` becomes the runtime control surface for activation guardrails.
- Advisory inference is allowed, but batch-provider submission, polling, queue runners, execution adapters, repo mutation, and runtime mutation remain out of scope.
- Hermes outputs remain local and source-linked until promoted through governed Initiative 132 surfaces.

## 2026-04-23 — Hermes Advisory Batch Context Has No Provider Execution

**Decision**

Adopt the next local Hermes pass as advisory batch-design context only. Hermes may receive governed Initiative 132, Doubleword, architecture, Nostra/Cortex boundary, and provider batch-policy references as read-only source material, but it must not receive provider credentials, submit Doubleword or other batch-provider jobs, poll batch APIs, or mutate the repo or runtime.

The active VPS runtime authority for this stage is also clarified as `cortex-gateway` plus `cortex_worker` under the current Hetzner runbook and runtime authority manifest. Older Python `eudaemon-alpha/` companion references are historical only; Initiative 132 authority lives in the root ICP tree.

**Why**

- The strongest useful pattern from the batch strategy work is the manifest-driven split: extractor/source manifest -> typed audit units -> advisory cognition -> Eudaemon synthesis -> governed publication.
- Initiative 132 already requires live cognition first, with batch audit secondary and advisory rather than boot-critical.
- Live provider APIs and execution-adapter logic would introduce action behavior before the source-manifest and audit-unit boundaries are accepted.
- Current deployment authority has moved to the repo-local gateway/worker contract, so older companion-worker wording should not steer the next implementation pass.

**Consequences**

- `SourceManifestV1`, `AuditUnitV1`, and `HermesObserverSessionV1` are planning-level contracts only, not public runtime APIs.
- Any future live batch adapter belongs under Initiative 134 after the advisory design stage is accepted.
- Hermes outputs may be promoted only through heap blocks, proposals, closeout follow-through, workflow drafts, or chronicle drafts.
- Broader execution-surface work remains deferred until opened as its own governed stage.

## 2026-04-03 — Meta-Harness Intake Reality Sync

**Decision**

Adopt the Meta-Harness findings for Initiative 132 as recommendation-only planning inputs, while treating gateway parity as locally passing, the root repo as authoritative for this pass, older `eudaemon-alpha/` companion references as historical only, and prompt override as unverified until a live path is demonstrated.

**Why**

- The current checkout validates the root Cortex/Nostra surfaces directly; the former companion `eudaemon-alpha/` implementation repo is no longer an active authority boundary.
- Meta-Harness is useful as a harness-evaluation pattern, not as a license to bypass governance, workflow authority, or steward review.
- The planning docs need to distinguish current evidence from future migration assumptions so later work does not inherit stale certainty.

**Consequences**

- Initiative 132 planning must label prompt-override behavior as unverified until a live path exists.
- Any Meta-Harness-derived optimization remains advisory until steward-reviewed and cannot self-apply changes.
- Active docs that refer to the `eudaemon-alpha/` companion boundary should treat it as historical only.

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

Initiative 132 remains authoritative in the root ICP repo. The Python Eudaemon Alpha worker was previously planned as a companion implementation repo boundary, but that companion path is now deprecated.

**Why**

- The root repo should remain the governance and architecture source of truth.
- The Python worker is transitional implementation surface, not the long-term platform authority.
- Earlier deployment notes used a submodule pattern to describe the boundary, but that boundary is not validated in the current checkout.

**Consequences**

- Root docs and Hetzner guidance should treat `eudaemon-alpha/` references as historical only.
- Agent-owned service units and bootstrap tooling now resolve through the root ICP tree and active Hetzner authority manifests.
- Root deployment flows should not require the deprecated companion repo.

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
- Historical `/srv/nostra/eudaemon-alpha/...` host paths no longer define the active authority surface; the current authority chain is the root ICP checkout plus the governed runtime manifest and Hetzner runbook.
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
- Batch 1 can be considered materially advanced for the current stage, but broader execution-surface work remains deferred until it is opened as its own governed batch.
- The current stage closeout should reference [`BATCH1_DECISION_GATE.md`](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/BATCH1_DECISION_GATE.md) as the authoritative evidence record for this extraction slice.

## 2026-04-10 — Developer Worktree Hygiene and Evidence Promotion Stay Operator-Side

**Decision**

Initiative 132 explicitly treats clean request worktrees, durable checkpointing, and immutable evidence promotion as operator/developer governance controls around the system-definition layer. They do not become new heap, closeout-ledger, or workflow primitives.

**Why**

- The current repo has shown that important steward-facing updates can be lost or obscured when generated artifacts, mutable runtime outputs, and authored changes coexist in one dirty tree.
- Initiative 132 already separates exploratory runtime material, operational follow-through, and durable workflow execution. Extending those runtime primitives to cover Git hygiene would blur the boundary between system definition and runtime execution.
- Initiative 125 is the right place for continuous hygiene/integrity gates, while Initiatives 133 and 134 already define how evidence and durable execution should re-enter the governed runtime stack.

**Consequences**

- Request work should start in a clean `.worktrees/` branch by default, with the shared root worktree reserved for repo-wide stewardship operations.
- Mutable `logs/*` outputs, including `*_latest.*` projections, remain local operational artifacts rather than durable Git authority.
- Evidence that matters must be promoted into governed initiative surfaces as immutable artifacts instead of being preserved only as mutable runtime projections.
- Handoff and context switching require a durable checkpoint bundle or WIP commit so important updates are not stranded in a dirty worktree.
