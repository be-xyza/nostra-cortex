# Knowledge Engine Close-Out Report

> **Scope**: Release hardening (local-first)
> **Window**: 2026-02-09 to 2026-02-21
> **Last Updated**: 2026-02-07

## 1. Evidence Artifacts

- Worker benchmark binary: `/Users/xaoj/ICP/nostra/worker/src/bin/benchmark_retrieval.rs`
- Benchmark matrix script: `/Users/xaoj/ICP/scripts/knowledge-closeout-benchmark.sh`
- Gate summary script: `/Users/xaoj/ICP/scripts/knowledge-closeout-gates.sh`
- Shadow/rollback drill script: `/Users/xaoj/ICP/scripts/knowledge-shadow-rollback-drill.sh`
- Latest benchmark report: `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_latest.json`
- Latest gate summary: `/Users/xaoj/ICP/logs/knowledge/knowledge_closeout_gate_summary_latest.json`
- Latest rollback drill: `/Users/xaoj/ICP/logs/knowledge/rollback_drill_latest.json`
- Captured matrix artifacts:
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_local_mock.json`
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_local_elna.json`
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_local_elna_strict.json` (only present when strict ELNA succeeds)
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_mock_mock.json`
  - `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_mock_elna.json`
- Local runbook: `/Users/xaoj/ICP/docs/cortex/knowledge-engine-local-runbook.md`

## 2. Build and Test Status

- Worker compile: `pass`
- Frontend compile: `pass`
- Worker lib tests (HRM long test skipped): `pass`
- `dfx build --check`: `pass` when frontend assets are prebuilt to `target/dx/...`

## 3. Release Gates

| Gate | Target | Status | Notes |
|---|---|---|---|
| Latency | `P95 < 900ms` for `top_k<=20` | pass | Gate summary shows active hybrid `P95=121ms` |
| Quality uplift | `Recall@10` + `nDCG@10` >= 20% vs baseline | fail | Gate summary shows `uplift_recall_pct=0`, `uplift_ndcg_pct=-2.38` |
| Provenance | `require_provenance=true` always grounded or explicit 422 | pass | Enforced by worker API checks + tests |
| Stability | Shadow parity threshold + rollback drill | fail | Rollback drill search continuity passed, but strict ELNA indexing failed (`index_status_code=500`) and shadow volume is below threshold |
| Build/test | Worker+Frontend+dfx checks pass | pass | Verified locally in close-out session |

## 4. Implemented Hardening Changes

- Vector runtime hardening:
  - ELNA timeout control (`NOSTRA_VECTOR_TIMEOUT_MS`)
  - ELNA fail-open toggle (`NOSTRA_ELNA_FAIL_OPEN`)
  - fail-open lexical fallback for degraded ELNA paths
- Diagnostics enrichment:
  - Retrieval diagnostics now include `backend` and `embedding_model`
  - rank reason guaranteed non-empty
- Provenance/metadata hardening:
  - CEI metadata validation on indexed docs
  - `require_provenance=true` path rejects non-grounded chunks
- Retrieval consistency:
  - legacy search module now delegates to `VectorService`
- Agent identity hardening:
  - added DFX PEM-based agent identity resolution with anonymous fallback
  - worker and benchmark now use shared identity resolution path
- Benchmark evidence hardening:
  - semantic-hard 50-query corpus replaced saturated lexical corpus
  - benchmark report now includes `shadow` metrics payload
  - strict ELNA benchmark case added (`NOSTRA_ELNA_FAIL_OPEN=false`)
  - gate-summary artifact added for machine-readable pass/fail verdicts
  - shadow/rollback drill script added with JSON artifact output

## 5. Added Test Coverage

- API tests:
  - diagnostics payload includes backend/model fields
  - ask path enforces provenance requirement
- Vector service tests:
  - CEI metadata validation and serialization
  - dimension mismatch rejection
  - ELNA degraded behavior (fail-open and fail-closed)
  - diagnostics completeness
- Provider routing tests:
  - auto/local/openai/mock routing outcomes

## 6. Remaining Close-Out Actions

1. Restore a reachable ELNA canister on the active local replica (`http://127.0.0.1:8000`) and confirm direct ingress.
2. Re-run strict case to generate `/Users/xaoj/ICP/logs/knowledge/retrieval_benchmark_local_elna_strict.json`.
3. Capture shadow parity evidence with `>=100` searches under strict ELNA success.
4. Execute rollback drill (`VECTOR_BACKEND=elna -> mock`) under live worker API and save operator log.
5. Re-run `/Users/xaoj/ICP/scripts/knowledge-closeout-gates.sh` and close verdict to `ready`.

## 7. Constraints Observed in This Session

- Real local-model benchmarks were executed with host-level access (Ollama reachable).
- ELNA principal endpoint used in this run: `uzt4z-lp777-77774-qaabq-cai`.
- Strict ELNA calls currently fail on direct ingress (`/api/v3/canister/<id>/call`) on the active replica, so fail-open paths can mask backend unavailability unless strict mode is enforced.
- `dfx build --check` completes, but RustSec advisory-db lock warning appears due read-only home cargo path in sandbox.

## 8. Verdict

Current verdict: **not-ready (external blocker)**.

Condition to move to **ready**:
- strict ELNA benchmark success + shadow volume threshold + quality uplift threshold, then re-run gate summary and close all remaining gates.
