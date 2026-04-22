# Knowledge Engine Local-First Runbook

> **Status**: Active
> **Audience**: Cortex operators and contributors validating local orchestration.
> **Last Updated**: 2026-02-08 (Sprint 5 modality evidence refresh)

## 1. Runtime Defaults

- Embedding provider selection: `NOSTRA_EMBEDDING_PROVIDER=auto|ollama|openai|mock`
- Local embedding default: `qwen3-embedding:0.6b` (`384` dimensions)
- Local generation default: `llama3.1:8b`
- Vector backend switch: `VECTOR_BACKEND=mock|elna` (or `NOSTRA_VECTOR_BACKEND`)
- ELNA canister endpoint: `NOSTRA_VECTOR_ENDPOINT=<principal>` (required for real ELNA mode)
- IC API endpoint: `IC_URL` (default `http://127.0.0.1:4943`)
- Agent identity override: `NOSTRA_AGENT_IDENTITY_PEM` (defaults to DFX identity discovery)
- Vector timeout: `NOSTRA_VECTOR_TIMEOUT_MS` (default `2500`)
- ELNA fail-open policy: `NOSTRA_ELNA_FAIL_OPEN=true|false` (default `true`)
- Shadow compare toggle: `NOSTRA_VECTOR_SHADOW_COMPARE=true|false`
- Canonical ELNA repo path: `/Users/xaoj/ICP/research/reference/topics/data-knowledge/elna-vector-db`
- Compatibility symlink: `/Users/xaoj/ICP/elna-vector-db`

## 2. Readiness Matrix (Source of Truth)

| Provider | Backend | Shadow | Expected behavior | Gate |
|---|---|---|---|---|
| `auto` | `mock` | `false` | Local-first route with mock-safe retrieval | Must pass |
| `ollama` | `mock` | `false` | Full local embedding + local retrieval storage | Must pass |
| `ollama` | `elna` | `true` | ELNA primary with shadow parity capture and fail-open fallback | Must pass before pilot |
| `openai` | `mock` | `false` | Comparative baseline when `OPENAI_API_KEY` present | Optional |
| `openai` | `elna` | `true` | Cloud embeddings with ELNA retrieval for comparison only | Optional |

Known degradations:
- If Ollama is unavailable and provider is `auto`, runtime routes to OpenAI when key exists, else to mock.
- If ELNA times out and `NOSTRA_ELNA_FAIL_OPEN=true`, retrieval remains available via lexical fallback.
- If `NOSTRA_ELNA_FAIL_OPEN=false`, ELNA timeout/failure returns explicit 5xx errors.
- If `NOSTRA_VECTOR_ENDPOINT` is unset/invalid, `VECTOR_BACKEND=elna` degrades to mock backend.
- If local gateway cannot resolve direct ingress for the configured ELNA principal, ELNA calls fail and
  runtime behavior depends on `NOSTRA_ELNA_FAIL_OPEN`:
  - `true`: lexical fail-open path remains available
  - `false`: explicit storage errors are returned (gate blocker)

## 3. Start Worker (Local-First)

```bash
cd nostra/worker
NOSTRA_EMBEDDING_PROVIDER=auto VECTOR_BACKEND=mock cargo run
```

## 4. Health and Readiness Checks

```bash
curl -s http://localhost:3003/health/model | jq
curl -s http://localhost:3003/activity/readiness | jq
curl -s http://localhost:3003/knowledge/shadow/report | jq
```

Expected:
- `/health/model` returns embedding probe success or explicit degradation reason.
- `/knowledge/shadow/report` returns parity metrics when shadow compare is enabled.
- `/health/model.vector.embedding_probe_ok == true` for hard gate.

## 5. Benchmark Harness (Quality + Latency)

Run the benchmark matrix (local mock + local ELNA + optional cloud):

```bash
./scripts/knowledge-closeout-benchmark.sh
```

Generate close-out gate verdict from captured artifacts:

```bash
./scripts/knowledge-closeout-gates.sh
```

Validate semantic search UI cutover evidence matrix:

```bash
./scripts/check_semantic_search_ui_surfaces.sh
```

Validate semantic search interaction evidence matrix:

```bash
./scripts/check_semantic_search_ui_interactions.sh
```

Validate semantic search browser evidence matrix (Playwright):

```bash
./scripts/check_semantic_search_ui_playwright.sh
```

Validate worker/frontend contract evidence matrix:

```bash
./scripts/check_semantic_search_worker_contracts.sh
```

Run shadow + rollback drill artifact generation:

```bash
./scripts/knowledge-shadow-rollback-drill.sh
```

Generate real executed run evidence under the testing log standard (blocking mode):

```bash
./scripts/knowledge-phase-next-run.sh --mode blocking
```

Run full close-out orchestration with resilient replica reuse:

```bash
./scripts/knowledge-closeout-run-local.sh --reuse-replica true
```

Outputs:
- `logs/knowledge/retrieval_benchmark_latest.json`
- `logs/knowledge/retrieval_benchmark_<timestamp>.json`
- `logs/knowledge/retrieval_benchmark_local_mock.json`
- `logs/knowledge/retrieval_benchmark_local_elna_strict.json` (only when strict ELNA succeeds)
- `logs/knowledge/retrieval_benchmark_local_elna_strict_metadata.json` (strict ELNA metadata-filter matrix case)
- `logs/knowledge/retrieval_benchmark_local_elna.json`
- `logs/knowledge/retrieval_benchmark_cloud_openai_mockbackend.json` (optional)
- `logs/knowledge/knowledge_closeout_gate_summary_latest.json`
- `logs/knowledge/rollback_drill_latest.json`
- `logs/knowledge/ui_contract_matrix_latest.json`
- `logs/knowledge/ui_surface_matrix_latest.json`
- `logs/knowledge/ui_interaction_matrix_latest.json`
- `logs/knowledge/ui_playwright_matrix_latest.json`
- `logs/knowledge/ui_playwright_report_latest.json`
- `logs/testing/test_catalog_latest.json`
- `logs/testing/runs/<run_id>.json`
- `logs/testing/test_gate_summary_latest.json`

Latest validated blocking run:
- `logs/testing/runs/local_ide_phase_next_20260208T154854Z.json`

Search mode controls:
- Main navigation: `data-a2ui-id="main-search-mode"`
- Ideation panel: `data-a2ui-id="ideation-search-mode"`
- Projects panel: `data-a2ui-id="projects-search-mode"`
- Values: `hybrid` (semantic) and `lexical` (keyword)

Modality controls:
- Main navigation: `data-a2ui-id="main-search-modality"`
- Ideation panel: `data-a2ui-id="ideation-search-modality"`
- Projects panel: `data-a2ui-id="projects-search-modality"`
- Values: `all`, `text`, `image`, `audio`, `video`

Note:
- In the testing-catalog gate stream, rollback drill execution is currently `informational` because some restricted environments cannot open local listener ports. Stability release evidence remains enforced in `logs/knowledge/knowledge_closeout_gate_summary_latest.json`.

Note:
- If Ollama is unreachable in constrained/sandboxed environments, the script falls back to `mock_*` benchmark cases and records that fallback in stdout.
- For ELNA evidence runs, export `NOSTRA_VECTOR_ENDPOINT=<elna_principal>` and keep `IC_URL` pointed at the same local replica.
- The matrix includes a strict ELNA case (`NOSTRA_ELNA_FAIL_OPEN=false`) before fail-open diagnostics; pilot readiness requires the strict case to pass.
- The matrix includes a strict ELNA metadata-filter case (`NOSTRA_BENCHMARK_METADATA_FILTERS=true`) to verify metadata-filtered retrieval quality in strict mode.

Metrics in report:
- `recall_at_10`
- `ndcg_at_10`
- `p50_latency_ms`
- `p95_latency_ms`
- delta metrics (`hybrid - lexical`)
- `provider`
- `backend`
- `embedding_model`
- `total_queries` (close-out target: `>= 50`)

## 6. Deterministic Validation Commands

```bash
# Worker compile + tests
cd nostra/worker
cargo check
cargo test --lib -- --skip skills::hrm_scheduler::tests::test_hrm_demo_execution

# Frontend compile
cd ../frontend
cargo check
CARGO_TARGET_DIR=target dx build --platform web

# Canister build check
cd ..
TERM=xterm-256color COLORTERM=truecolor dfx build --check
```

## 7. Replica Port Collision Handling

The close-out runner now checks local port health before replica startup:

1. It prints:
   - `lsof -nP -iTCP:4943 -sTCP:LISTEN`
   - `dfx ping`
2. If a healthy local replica is already running and `--reuse-replica=true`, it reuses that replica.
3. If port `4943` is occupied but `dfx ping` fails, the script exits with manual remediation steps instead of killing processes.

Manual remediation:

```bash
lsof -nP -iTCP:4943 -sTCP:LISTEN
dfx stop
./scripts/knowledge-closeout-run-local.sh --reuse-replica true
```

## 8. Cutover Gate Checklist

Before enabling ELNA for pilot surfaces:

1. `cargo check` passes for worker and frontend.
2. Worker tests pass (`cargo test --lib -- --skip skills::hrm_scheduler::tests::test_hrm_demo_execution`).
3. `/health/model` shows embedding dimension and probe status as `ok`.
4. Shadow report has non-zero search volume and acceptable parity trend.
5. Shadow acceptance threshold: `average_parity >= 0.60` over at least `100` searches.
6. Rollback drill succeeds (`VECTOR_BACKEND=elna -> mock`) with uninterrupted `/knowledge/search`.
7. Benchmark evidence captures `>= 50` queries and records `P95` for both lexical and hybrid modes.
8. Pilot gate: `P95 < 900ms` for `top_k<=20`; quality uplift target `>=20%` over baseline.
9. Provenance gate: `require_provenance=true` returns only cited chunks with provenance or explicit `422`.
10. Knowledge Workbench validates: ingest -> search -> ask with provenance citations and diagnostics.
11. Blocking test gate validates:
   - `bash scripts/knowledge-phase-next-run.sh --mode blocking`
   - `test_gate_summary_latest.json.overall_verdict == "ready"`
12. UI surface gate validates:
   - `bash scripts/check_semantic_search_ui_surfaces.sh`
   - `knowledge_closeout_gate_summary_latest.json.gates.ui_surface_matrix_present == true`
13. UI interaction gate validates:
   - `bash scripts/check_semantic_search_ui_interactions.sh`
   - `knowledge_closeout_gate_summary_latest.json.gates.ui_interaction_matrix_present == true`
14. UI contract gate validates:
   - `bash scripts/check_semantic_search_worker_contracts.sh`
   - `knowledge_closeout_gate_summary_latest.json.gates.ui_contract_matrix_present == true`
15. UI browser gate validates:
   - `bash scripts/check_semantic_search_ui_playwright.sh`
   - `knowledge_closeout_gate_summary_latest.json.gates.ui_playwright_matrix_present == true`

If UI browser gate fails with a blank page:
1. Inspect artifacts:
   - `logs/knowledge/ui_playwright_matrix_latest.json`
   - `logs/knowledge/ui_playwright_report_latest.json`
   - `logs/knowledge/ui_playwright_server_latest.log`
2. Confirm whether failure is render/runtime versus network:
   - runtime blank/panic typically shows `tests_unexpected > 0` with no search UI controls found.
   - if stderr includes `The hook list is already borrowed`, treat as hook-order/runtime state regression.
3. Validate frontend runtime guardrails:
   - ensure `/Users/xaoj/ICP/nostra/frontend/src/services/vfs_service.rs` remains hook-free and uses global state initialization.
   - run `cargo check --manifest-path /Users/xaoj/ICP/nostra/frontend/Cargo.toml`.
4. Re-run just the browser gate after frontend fixes:
   - `bash scripts/check_semantic_search_ui_playwright.sh`
5. Then refresh blocking evidence:
   - `bash scripts/knowledge-phase-next-run.sh --mode blocking`
   - `bash scripts/knowledge-closeout-gates.sh`

## 9. Rollback Drill

If ELNA degrades:

```bash
export VECTOR_BACKEND=mock
```

Then re-run:

```bash
curl -s http://localhost:3003/health/model | jq
```

Confirm:
- backend returns `mock`
- `/knowledge/search` remains available
- `/knowledge/shadow/report` still reports historical parity values

## 10. Cutover Signoff Record

Record this in your close-out report:

- Date/time of cutover validation
- Operator name
- Benchmark artifact paths
- Shadow report snapshot
- Rollback drill log
- Verdict: `ready` or `not-ready` with explicit blocker list
