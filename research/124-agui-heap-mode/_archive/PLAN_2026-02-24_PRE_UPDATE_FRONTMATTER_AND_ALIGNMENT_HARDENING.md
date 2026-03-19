# Initiative 124 Plan Lock: Desktop Source, Web Parity Rollout

Date: 2026-02-24  
Initiative: `research/124-agui-heap-mode`

## Scope Lock

1. Source implementation: `cortex/apps/cortex-desktop` and `cortex/libraries/cortex-domain`.
2. Web parity: `cortex/apps/cortex-web` as post-desktop contract consumer.
3. Out of scope: unrelated workspace conflict cleanup and mirrored dual-host implementation tracks.
4. Non-goal: importing Heaper runtime components.

## Contract Surface

1. `POST /api/cortex/studio/heap/emit`
2. `GET /api/cortex/studio/heap/blocks`
3. `POST /api/cortex/studio/heap/blocks/:artifact_id/pin`
4. `POST /api/cortex/studio/heap/blocks/:artifact_id/delete`

## Phase Status (2026-02-24)

1. Phase 0 Guardrails/Baseline: completed.
2. Phase 1 Canonical Heap Ingress: completed.
3. Phase 2 Deterministic Mapping/Canonicalization: completed.
4. Phase 3 Projection + Query Surface: completed.
5. Phase 4 Desktop Heap Board Wiring: completed.
6. Phase 5 Agent Emission Integration: completed.
7. Phase 6 Desktop Hardening + Contract Freeze Candidate: completed.
8. Phase 7 Web Parity Rollout: in progress (feature-flagged consumer + parity request-contract tests landed).
9. Phase 8 Research/Governance Closeout: in progress (this update).

## Default Policies

1. Mention policy default: mirror mentions to relations unless explicitly disabled in payload hints.
2. File key policy default: persist canonical `hash:file_size`, accept `hash` on ingress.
3. CRDT authority: Initiative 113 primitives remain canonical; heap projection is denormalized query materialization.
