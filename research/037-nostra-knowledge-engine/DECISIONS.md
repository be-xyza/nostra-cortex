---
id: '037'
name: nostra-knowledge-engine
title: 'Decisions: Knowledge Engine Strategy'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions: Knowledge Engine Strategy

## 001. Separate Initiative vs Merge
**Context**: We needed to decide whether to bundle this work into `021-kip-integration`, `034-nostra-labs`, or start a new logic.
**Decision**: Start **037-nostra-knowledge-engine**.
**Reasoning**:
- `021` is about the *Protocol* (Spec).
- `034` is about the *Container* (Platform).
- `037` is about the *Tool* (Implementation).
- The "Conversion Layer" is a distinct, complex logic domain that deserves its own lifecycle and documentation.

## 002. Client-Side Processing
**Decision**: The "Engine" will run in the browser (Wasm).
**Reasoning**:
- Privacy: User's raw CSVs shouldn't leave their machine until they choose to save the resulting Knowledge Graph.
- Speed: Instant feedback loop for mapping columns.

## 003. Worker-First Knowledge API Contract (Local-First)
**Date**: 2026-02-07
**Decision**: Standardize knowledge engine ingress/retrieval on worker HTTP contracts:
- `POST /knowledge/index` (idempotency + CEI metadata)
- `POST /knowledge/search` (retrieval mode + filters + diagnostics)
- `POST /knowledge/ask` (grounded response with citations)
- `GET /health/model` (embedding/vector readiness)

**Reasoning**:
- Keeps local-first orchestration fast for scaffolding and iteration.
- Decouples frontend integration from canister-level embedding logic.
- Enables explicit readiness and provenance gating before broader rollout.

## 004. Release-Hardening Close-Out Criteria
**Date**: 2026-02-07
**Decision**: Close-out readiness is defined by evidence gates, not feature volume. Required gates:
- Build/test gate passes (worker + frontend + canister check path).
- Provenance gate enforced (`require_provenance=true` rejects non-grounded chunks).
- Stability gate verified (ELNA shadow parity evidence + rollback drill).
- Benchmark gate documented (quality/latency deltas captured in artifact reports).

**Reasoning**:
- Protects pilot rollout quality while preserving local-first momentum.
- Keeps the initiative aligned with constitutional requirements on lineage and memory integrity.
- Separates release hardening from long-horizon enrichment so scope stays bounded and auditable.

## 005. Agent Identity Resolution + ELNA Direct-Ingress Gate
**Date**: 2026-02-07
**Decision**:
- Resolve worker/benchmark IC agent identity from local DFX PEM by default (`NOSTRA_AGENT_IDENTITY_PEM` override, anonymous fallback).
- Treat ELNA local gateway direct-ingress failure (`canister id not resolved`) as a hard close-out blocker for ELNA-backed evidence.

**Reasoning**:
- Local pilot execution must not assume anonymous access for admin-gated canisters.
- Fail-open behavior is useful for availability, but cannot be used as evidence of ELNA readiness.
- Gate integrity requires proving true ELNA insert/search paths, not lexical fallback outcomes.

## 006. Benchmark Evidence Hardening + Machine-Readable Gate Verdict
**Date**: 2026-02-07
**Decision**:
- Upgrade retrieval benchmark corpus from lexical-friendly prompts to semantic-hard prompts.
- Extend benchmark report payload to include `shadow` metrics.
- Add strict ELNA matrix case (`NOSTRA_ELNA_FAIL_OPEN=false`) ahead of fail-open diagnostics.
- Add deterministic gate-summary artifact generation (`scripts/knowledge-closeout-gates.sh`) as release evidence source.

**Reasoning**:
- The previous benchmark corpus saturated lexical retrieval, masking meaningful quality deltas.
- Close-out requires explicit distinction between strict ELNA success and fail-open fallback behavior.
- A machine-readable gate summary reduces ambiguity and makes readiness reproducible across operators.

## 007. ELNA Index Refresh Requirement
**Date**: 2026-02-08
**Decision**:
- Rebuild ELNA ANN index after successful insert batches in `VectorService::index_documents`.
- Keep fail-open/fail-closed behavior unchanged; index refresh is required in both modes whenever vector writes succeed.

**Reasoning**:
- ELNA append operations do not automatically refresh searchable ANN state.
- Strict-mode indexing could succeed while retrieval returned low/empty overlap, causing false shadow parity failures.
- Rebuilding once per indexing call restores deterministic retrieval behavior for close-out evidence.

## 008. Shadow Parity Normalization
**Date**: 2026-02-08
**Decision**:
- Compute shadow parity by comparing dense top-k against lexical top-k over the same candidate pool.
- Assign lexical score `0.0` to candidates with no lexical match before ranking.

**Reasoning**:
- Previous parity calculation used a sparse lexical set, which could undercount overlap despite valid dense retrieval.
- Candidate-pool normalization makes parity meaningful, repeatable, and aligned with readiness gate intent.

## 009. Close-Out Readiness Verdict
**Date**: 2026-02-08
**Decision**:
- Mark local-first knowledge-engine close-out gates as `ready` for pilot continuation.
- Evidence source: `/Users/xaoj/ICP/logs/knowledge/knowledge_closeout_gate_summary_latest.json` (generated `2026-02-08T15:54:42Z`).

**Reasoning**:
- All required gates are passing in the latest run:
  - `latency_pass = true`
  - `quality_pass = true`
  - `shadow_pass = true`
  - `strict_elna_pass = true`
- Rollback continuity remains validated in `/Users/xaoj/ICP/logs/knowledge/rollback_drill_latest.json`.

## 010. Canonical ELNA Repository Location Policy
**Date**: 2026-02-08
**Decision**:
- Treat `/Users/xaoj/ICP/research/reference/topics/data-knowledge/elna-vector-db` as the canonical ELNA source.
- Keep `/Users/xaoj/ICP/elna-vector-db` as a compatibility symlink only; do not treat it as authoritative source-of-truth.

**Reasoning**:
- Aligns ELNA with reference intake governance and catalog indexing.
- Prevents path drift between research docs and runtime scripts.
- Preserves local compatibility for existing commands while converging on one canonical location.

## 011. Real Executed Test-Run Logging Standard
**Date**: 2026-02-08
**Decision**:
- Require phase execution evidence to include:
  - `logs/testing/test_catalog_latest.json`
  - `logs/testing/runs/<run_id>.json` (real command execution, non-synthetic)
  - `logs/testing/test_gate_summary_latest.json`
- Keep `scripts/write_test_run_artifact.sh` for rehearsal/synthetic contract checks only.

**Reasoning**:
- Close-out and continuation gates must be auditable against actual execution, not simulated status matrices.
- Standardized run artifacts make gate outcomes reproducible and machine-checkable.

## 012. Next-Phase Continuation Gates
**Date**: 2026-02-08
**Decision**:
- Continue post-closeout work only when both gate streams remain green:
  - Knowledge gate summary: `/Users/xaoj/ICP/logs/knowledge/knowledge_closeout_gate_summary_latest.json` = `ready`
  - Testing gate summary (blocking mode): `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json` = `ready`
- Treat metadata-filter regression on `perspective_scope`/`produced_by_agent`/`source_version_id` as release-blocking for this phase.

**Reasoning**:
- Prevents moving forward on enrichment while operational confidence regresses.
- Keeps retrieval quality, provenance, and test governance coupled as one readiness contract.

## 013. Replica Reuse Policy for Local Close-Out Runner
**Date**: 2026-02-08
**Decision**:
- `scripts/knowledge-closeout-run-local.sh` defaults to `--reuse-replica=true`.
- If IC replica port (`4943`) is already bound and `dfx ping` succeeds, runner reuses the existing healthy replica.
- If port is bound but health check fails, runner exits with explicit manual remediation steps instead of force-killing listeners.

**Reasoning**:
- Resolves repeated local-blocker failures caused by parallel/stale replica startup attempts.
- Prevents destructive process termination during active developer sessions.
- Keeps close-out run behavior deterministic while preserving operator control.

## 014. Additive Metadata Filter Contract Sync
**Date**: 2026-02-08
**Decision**:
- Extend additive metadata filter fields across contracts:
  - `perspective_scope`
  - `produced_by_agent`
  - `source_version_id`
- Keep API compatibility by making all new fields optional.
- Extend CEI metadata model with optional lineage/agent fields (`source_version_id`, `model_id`, `perspective_scope`, `produced_by_agent`, `confidence`, `purpose`) in frontend-worker contract types.

**Reasoning**:
- Aligns worker retrieval capabilities with frontend request builders and canister declarations.
- Removes contract drift that can silently drop filter intent at boundaries.
- Preserves backward compatibility for existing clients.

## 015. Real Blocking Run Evidence Recorded for Sprint 2
**Date**: 2026-02-08
**Decision**:
- Treat `logs/testing/runs/local_ide_phase_next_20260208T025647Z.json` as the initial blocking-mode continuation evidence run.
- Refresh latest closeout evidence with `logs/testing/runs/local_ide_closeout_20260208T032029Z.json` after strict metadata-matrix execution.
- Require gate outputs to remain green after changes:
  - `logs/testing/test_gate_summary_latest.json` = `ready`
  - `logs/knowledge/knowledge_closeout_gate_summary_latest.json` = `ready`

**Reasoning**:
- Provides concrete machine-readable proof that release-blocker tests executed under the new test-log standard.
- Couples code change completion with refreshed operational readiness evidence.

## 016. Strict ELNA Metadata-Filter Matrix Requirement
**Date**: 2026-02-08
**Decision**:
- Add a strict ELNA benchmark matrix case with metadata filters enabled:
  - `NOSTRA_ELNA_FAIL_OPEN=false`
  - `NOSTRA_BENCHMARK_METADATA_FILTERS=true`
- Persist artifact as:
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_local_elna_strict_metadata.json`
- Surface metadata-matrix evidence in knowledge gate summary via:
  - `metrics.metadata_filter_queries`
  - `metrics.metadata_filtered_hybrid_*`
  - `gates.metadata_matrix_present`

**Reasoning**:
- Closes the remaining continuation gap in initiative `042` for strict-mode metadata-filter benchmarking.
- Ensures filter correctness is evidenced in both test and benchmark streams.
- Keeps strict ELNA quality evidence machine-readable for future gate automation.

## 017. Semantic Search UI Cutover Beyond Workbench
**Date**: 2026-02-08
**Decision**:
- Retire legacy main-nav placeholder retrieval path (`vector_search` + alert UX) in favor of live worker retrieval.
- Expand live semantic retrieval to product pages: Ideation and Projects (in addition to Knowledge Workbench).
- Standardize surface request payload construction through shared frontend service contracts.

**Reasoning**:
- Removes divergence between lab-only capabilities and real product surfaces.
- Makes provenance-visible retrieval available where users actually operate (navigation + contribution pages).
- Reduces request-shape drift by centralizing filter/ask payload builders.

## 018. UI Surface Evidence as Close-Out Gate Input
**Date**: 2026-02-08
**Decision**:
- Introduce `scripts/check_semantic_search_ui_surfaces.sh` to generate:
  - `/Users/xaoj/ICP/logs/knowledge/ui_surface_matrix_latest.json`
- Add `ui_surface_matrix_present` gate metric in `/Users/xaoj/ICP/scripts/knowledge-closeout-gates.sh`.
- Require latest blocking run artifacts to link the UI surface matrix for readiness continuity.

**Reasoning**:
- Gate readiness must prove not only backend quality but also frontend cutover completion.
- A machine-readable matrix prevents regressions back to alert/mock UI paths.
- Linking the matrix via run artifacts keeps evidence auditable across phases.

## 019. UI Contract Matrix as Blocking Continuation Evidence
**Date**: 2026-02-08
**Decision**:
- Introduce `scripts/check_semantic_search_worker_contracts.sh` to generate:
  - `/Users/xaoj/ICP/logs/knowledge/ui_contract_matrix_latest.json`
- Add this contract matrix to release-blocker test catalog entries and run artifacts.
- Extend close-out gate criteria with:
  - `gates.ui_contract_matrix_present = true`

**Reasoning**:
- UI rollout readiness also depends on contract continuity between worker requests, worker responses, and frontend rendering.
- Contract-level evidence catches regressions that surface-only checks cannot detect (for example, dropped `ui_surface` propagation).
- Treating this as blocking keeps operator diagnostics reliable across future incremental phases.

## 020. Full-App Browser Runtime Stabilization for Semantic Search
**Date**: 2026-02-08
**Decision**:
- Stabilize full-app browser execution by making frontend VFS state management hook-free in `/Users/xaoj/ICP/nostra/frontend/src/services/vfs_service.rs`.
- Keep `VfsService` `Copy`-compatible while backing storage with a global `OnceLock<Mutex<FileNode>>` to preserve existing closure/event call patterns.
- Retain panic instrumentation (`console_error_panic_hook`) for fast diagnosis of future runtime regressions.

**Reasoning**:
- Browser gate failures showed blank-page behavior caused by runtime panic (`The hook list is already borrowed`) from hook usage inside VFS initialization paths.
- A hook-free state backend removes this class of Dioxus hook-order violations while minimizing code churn on existing UI surface handlers.
- This approach preserves sprint velocity by fixing runtime stability without introducing breaking frontend API changes.

## 021. Blocking Evidence Refresh After Runtime Stabilization
**Date**: 2026-02-08
**Decision**:
- Record `logs/testing/runs/local_ide_phase_next_20260208T145226Z.json` as the latest blocking continuation run after runtime stabilization changes.
- Require the refreshed run to keep both gate streams green:
  - `logs/testing/test_gate_summary_latest.json` (`overall_verdict=ready`, `pass=29`, `fail=0`)
  - `logs/knowledge/knowledge_closeout_gate_summary_latest.json` (`verdict=ready`)
- Require browser matrix evidence in the same run:
  - `logs/knowledge/ui_playwright_matrix_latest.json` (`tests_expected=3`, `tests_unexpected=0`)

**Reasoning**:
- Runtime stabilization must be proven by a full blocking rerun, not assumed from compile success.
- Linking browser evidence to the same run artifact prevents drift between UI claims and gate-level execution proof.

## 022. Semantic Mode Toggle Across Product Surfaces
**Date**: 2026-02-08
**Decision**:
- Add user-facing retrieval mode controls (`hybrid`/semantic vs `lexical`/keyword) on:
  - main navigation search
  - Ideation related-knowledge panel
  - Projects related-knowledge panel
- Propagate selected mode through shared frontend request builders into worker payloads (`retrieval_mode`).
- Treat request-contract propagation as a blocking browser check in the semantic UI Playwright suite.
- Refresh blocking evidence after this rollout:
  - `logs/testing/runs/local_ide_phase_next_20260208T154854Z.json`
  - `logs/testing/test_gate_summary_latest.json` (`overall_verdict=ready`, `pass=29`, `fail=0`)
  - `logs/knowledge/knowledge_closeout_gate_summary_latest.json` (`verdict=ready`)

**Reasoning**:
- Operator and user workflows need explicit control over semantic versus keyword retrieval behavior for debugging and relevance tuning.
- The worker contract already supports retrieval mode selection, so UI controls can be added additively without API breaks.
- Browser-level verification prevents silent regressions where UI controls exist but payloads ignore selected mode.

## 023. Modality Controls + Evidence Enforcement Across Product Surfaces
**Date**: 2026-02-08
**Decision**:
- Add modality selector controls (`all|text|image|audio|video`) on:
  - main navigation search
  - Ideation related-knowledge panel
  - Projects related-knowledge panel
- Propagate modality selection through shared search payload builders into `filters.modalities`.
- Extend blocking evidence matrices to require modality coverage:
  - `logs/knowledge/ui_surface_matrix_latest.json`
  - `logs/knowledge/ui_interaction_matrix_latest.json`
  - `logs/knowledge/ui_contract_matrix_latest.json`
  - `logs/knowledge/ui_playwright_matrix_latest.json`
- Record latest blocking continuation run with modality checks:
  - `logs/testing/runs/local_ide_phase_next_20260208T154854Z.json`

**Reasoning**:
- Modal namespace routing in worker is only operationally useful when product surfaces can explicitly request modality-scoped retrieval.
- Gate-level evidence must prove both UI control presence and request-contract propagation to avoid silent regressions.
- Keeping modality assertions in the blocking stream preserves continuation readiness for local-first orchestration.
