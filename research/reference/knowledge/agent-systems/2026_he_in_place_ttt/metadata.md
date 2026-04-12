---
schema_version: "2.0"
artifact_id: "in-place-ttt-2026"
artifact_type: "paper"
title: "In-Place Test-Time Training"
authors:
  - "Guhao Feng"
  - "Shengjie Luo"
  - "Kai Hua"
  - "Ge Zhang"
  - "Wenhao Huang"
  - "Di He"
  - "Tianle Cai"
year: "2026"
publisher: "arXiv"
upstream_url: "https://arxiv.org/html/2604.06169v1"
source_files:
  - path: "research/reference/knowledge/agent-systems/2026_he_in_place_ttt/paper.md"
    sha256: "e66965915e81c979f023ba8746cf66883cab68d84b6e91149815b48e32a43764"
    mime_type: "text/plain"
topic: "agent-systems"
tags: ["LLM", "TTT", "inference", "adaptation", "long-context"]
status: "reviewed"
nostra_cortex_scope: "Cortex runtime host-side adaptive inference experiments; not a Nostra authority artifact"
initiative_refs:
  - "118-cortex-runtime-extraction"
  - "132-eudaemon-alpha-initiative"
primary_steward: "Research Steward"
authority_mode: "recommendation_only"
escalation_path: "steward_review -> owner_decision"
lineage_record: "Primary-source manually validated on 2026-04-11 against arXiv HTML v1 and the OpenReview ICLR 2026 entry; unsupported assumptions from the initial intake draft were reduced to recommendation-only inferences. Validator-backed compliance is not claimed in this checkout. See research/reference/analysis/reference-intake-protocol-drift-2026-04-11.md."
review_date: "2026-04-11"
confidence_score: 4
source_reliability: 4
validation_proof:
  method: "Primary-source manually validated against arXiv HTML v1 and the OpenReview ICLR 2026 entry."
  evidence_refs:
    - "https://arxiv.org/html/2604.06169v1"
    - "https://openreview.net/forum?id=dTWfCLSoyl"
  validated_findings:
    - "The method repurposes the final projection matrix of MLP blocks as adaptable fast weights."
    - "The paper introduces a next-token-prediction-aligned objective and gives theorem-backed motivation for that target."
    - "The implementation uses chunk-wise updates compatible with context parallelism and resets fast weights at document boundaries."
    - "Reported evaluations cover long-context LLM benchmarks and from-scratch pretraining comparisons, including Qwen3-4B-Base, LLaMA-3.1-8B, and Qwen3-14B-Base."
  rejected_or_weakened_assumptions:
    - "The paper does not demonstrate direct compatibility with Nostra graph schemas, workflow memory, or agent-rs state."
    - "The paper does not justify treating In-Place TTT as a core Cortex memory architecture decision; that remains an exploratory runtime inference."
adoption_decision: "Retain as a reviewed exploratory reference for Cortex host-side adaptive inference patterns."
known_risks: "Requires mutable inference-time weights and accelerator-oriented execution; the paper does not validate Internet Computer canisters, CPU-only serving, or graph-native agent memory deployments."
suggested_next_experiments: "Benchmark a host-side Cortex adapter that applies chunk-wise fast-weight updates, then compare latency, memory overhead, and quality against retrieval/context-window baselines before any broader adoption decision."
standards_alignment:
  modularity:
    score: 4
    applicability: "core"
    rationale: "The method is presented as a drop-in modification to standard MLP blocks rather than a wholly separate architecture."
  composability:
    score: 3
    applicability: "core"
    rationale: "It composes with existing LLM backbones and RoPE extension techniques, but still assumes control over model internals and training/inference plumbing."
  confidence_integrity:
    score: 3
    applicability: "core"
    rationale: "The paper offers theory plus experiments, but this intake validates paper claims only and does not reproduce results locally."
  portability:
    score: 2
    applicability: "supporting"
    rationale: "The published evidence is oriented toward GPU training and inference; portability to Cortex runtime hosts or IC-adjacent environments is unproven."
  durability:
    score: 2
    applicability: "supporting"
    rationale: "The fast-weight state is intentionally transient and reset at document boundaries, which limits direct reuse as durable memory."
  accessibility:
    score: 4
    applicability: "supporting"
    rationale: "The paper is available in accessible arXiv HTML and an OpenReview entry, which supports traceable review."
  overall_weighted_score: 3.0
---

# In-Place Test-Time Training
Primary-source manually validated against `paper.md` and the arXiv/OpenReview entries.
