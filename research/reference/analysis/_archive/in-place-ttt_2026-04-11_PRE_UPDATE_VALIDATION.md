# In-Place Test-Time Training Analysis

## Placement
`research/reference/knowledge/agent-systems/2026_he_in_place_ttt`

## Intent
Analyze the "In-Place Test-Time Training" paper to determine if dynamic adaptation of LLM fast weights fits the Cortex Eudaemon runtime. The paper presents a framework to use MLP final projection matrices as fast weights, dynamically updated at inference to remember contextual information.

## Possible Links To Nostra Platform and Cortex Runtime
- **Cortex Eudaemon Adaptation**: The runtime agent operates over long-context state (user graphs and histories). TTT could provide a way to adapt the base LLM on-the-fly to specific graph schemas without fine-tuning.
- **Workflow Memory**: Could act as a substitute to infinite-context parsing by natively ingesting execution contexts in chunks.

## Initiative Links
- None specific yet, but relates broadly to evaluating future agent harness loops.

## Pattern Extraction
- **In-Place Modification**: Repurposing existing MLP components prevents needing completely net-new pre-training architectures.
- **LM-Aligned Objective**: Uses future token information to align TTT reconstruction with next-token prediction, unlike older TTT papers that just use self-reconstruction.
- **Chunk-wise Updates**: Efficient strategy processing chunks of context for hardware utilization.

## Adoption Decision
Retain as knowledge reference. The architectural approach to fast weights could inform future eudaemon runtime evolution, particularly for caching long-lived user graph context.

## Known Risks
The technique requires control over model inference weights and gradient descent at inference time, which complicates Hetzner VPS and IC-based sandboxing.

## Suggested Next Experiments
Map the chunk-wise logic to our current `cortex-eudaemon` workflow node schema to estimate the latency overhead on standard runtime hosts. Evaluate how fast weights integrate with `agent-rs` state.
