# In-Place Test-Time Training Analysis

## Placement
`research/reference/knowledge/agent-systems/2026_he_in_place_ttt`

## Intent
Validate the existing intake for the "In-Place Test-Time Training" paper against primary sources and retain only source-backed claims. The paper proposes inference-time fast-weight updates that repurpose the final projection matrix of standard MLP blocks, use a next-token-prediction-aligned objective, and apply chunk-wise/context-parallel updates for long-context language modeling experiments.

## Possible Links To Nostra Platform and Cortex Runtime
- **Cortex runtime inference adapters**: Relevant only as a host-side pattern source for adaptive inference experiments in Cortex. The paper evaluates long-context language modeling, not Nostra graph authority, workflow semantics, or agent memory products.
- **Eudaemon long-context experiments**: It may inform evaluation ideas for Initiative 132 where long-running agent loops need better long-context handling. This is an inference from the paper, not a claim made by the authors.
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
