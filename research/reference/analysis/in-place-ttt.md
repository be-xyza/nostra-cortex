# In-Place Test-Time Training Analysis

## Placement
`research/reference/knowledge/agent-systems/2026_he_in_place_ttt`

## Intent
Primary-source manually validate the existing intake for the "In-Place Test-Time Training" paper and retain only source-backed claims. The paper proposes inference-time fast-weight updates that repurpose the final projection matrix of standard MLP blocks, use a next-token-prediction-aligned objective, and apply chunk-wise/context-parallel updates for long-context language modeling experiments.

## Possible Links To Nostra Platform and Cortex Runtime
- **Repository inference - Cortex runtime inference adapters**: Relevant only as a host-side pattern source for adaptive inference experiments in Cortex. The paper evaluates long-context language modeling, not Nostra graph authority, workflow semantics, or agent memory products.
- **Repository inference - Eudaemon long-context experiments**: It may inform evaluation ideas for Initiative 132 where long-running agent loops need better long-context handling. This is an inference from the paper, not a claim made by the authors.
- **Boundary reminder**: Any adoption would live in Cortex execution infrastructure or host-side model adapters, never in Nostra authority surfaces.

## Initiative Links
- `118-cortex-runtime-extraction`: relevant if host-side adaptive inference is explored behind runtime adapters and purity boundaries.
- `132-eudaemon-alpha-initiative`: relevant for recommendation-only experiments around long-context agent execution quality.

## Pattern Extraction
- **In-place fast weights**: The method reuses the final projection matrix of MLP blocks as adaptable fast weights instead of adding a wholly new memory architecture.
- **LM-aligned objective**: The paper replaces generic reconstruction with a next-token-prediction-aligned objective and provides theorem-backed motivation for why that target is better aligned with language modeling.
- **Chunk-wise/context-parallel execution**: The update rule is structured so chunks can be processed in parallel while preserving causal semantics and resetting fast weights at document boundaries.
- **Validated scope**: The paper reports gains on long-context benchmarks and pretraining comparisons for LLMs; it does not validate graph-schema adaptation, retrieval replacement, or `agent-rs` integration.

## Adoption Decision
Retain as a reviewed exploratory reference for Cortex host-side adaptive inference patterns. Do not treat it as a core memory architecture decision or as direct evidence for Nostra/Cortex graph-native agent memory.

## Known Risks
The paper's benefits depend on mutable inference-time weights, continual/continued training workflows, and accelerator-oriented execution. The reported experiments do not validate Internet Computer canister deployment, CPU-only serving, or direct compatibility with current Cortex runtime hosts.

## Suggested Next Experiments
Benchmark a host-side Cortex adapter prototype that applies chunk-wise fast-weight updates outside canister authority, then compare latency, memory overhead, and quality against retrieval/context-window baselines before any broader adoption decision.

## Claim Audit

| Bucket | Claim | Primary-source anchor | Disposition |
|---|---|---|---|
| validated | In-Place TTT reuses the final projection matrix of MLP blocks as fast weights. | [arXiv abstract](https://arxiv.org/html/2604.06169v1#abstract1), [OpenReview abstract](https://openreview.net/forum?id=dTWfCLSoyl) | retained |
| validated | The paper introduces an LM-aligned/NTP-aligned objective with theorem-backed motivation. | [arXiv section 3.2](https://arxiv.org/html/2604.06169v1#S3.SS2), [arXiv section 3.3](https://arxiv.org/html/2604.06169v1#S3.SS3) | retained |
| validated | The implementation uses chunk-wise updates compatible with context parallelism and resets fast weights at document boundaries. | [arXiv section 3.4](https://arxiv.org/html/2604.06169v1#S3.SS4), [arXiv appendix 8](https://arxiv.org/html/2604.06169v1#S8) | retained |
| validated | The evaluation covers long-context benchmarks and from-scratch pretraining comparisons, including Qwen3-4B-Base, LLaMA-3.1-8B, and Qwen3-14B-Base. | [arXiv section 4.1](https://arxiv.org/html/2604.06169v1#S4.SS1), [arXiv section 4.2](https://arxiv.org/html/2604.06169v1#S4.SS2) | retained |
| inference | The paper may inform Cortex host-side adaptive inference experiments and Initiative 132 long-context evaluation work. | Repository inference from the paper's long-context adaptation scope; not an author claim. | retained as recommendation-only relevance |
| removed | The paper validates graph-schema adaptation, workflow-memory replacement, or `agent-rs` integration. | No support found in arXiv HTML v1 or OpenReview. | removed |
| removed | The intake is validator-backed or fully protocol-compliant in this checkout. | No support found because the README-referenced validator/topic-registry assets are absent locally. | removed; downgraded to manual validation |
