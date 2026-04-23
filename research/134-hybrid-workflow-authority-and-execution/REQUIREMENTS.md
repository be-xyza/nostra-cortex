# Requirements

1. Canonical source-of-truth IR must be an internal workflow definition artifact, not raw Serverless Workflow JSON.
2. All derived projections must be deterministic and reproducible from the canonical definition.
3. Human checkpoints and evaluation gates must emit typed A2UI projection payloads rather than embedding renderer-specific schemas in the canonical definition.
4. Workflow motifs must be bounded and validate structural constraints before they can be staged or compiled.
5. Governance envelopes must include decision gate, replay, lineage, and degraded-source metadata.
6. Execution adapters must preserve replay evidence and discarded branch outputs for comparative workflows.
7. The dedicated canister path must not be treated as canonical executor until workspace/build/runtime validation is demonstrated.
