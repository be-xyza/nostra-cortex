---
id: hera
name: hera
title: HERA Paper Analysis
type: reference_analysis
project: cortex
status: draft
portfolio_role: satellite
execution_plane: cortex
authority_mode: recommendation_only
reference_topics: [agent-systems]
reference_assets:
  - "research/reference/knowledge/agent-systems/2026_Li_HERA"
evidence_strength: strong
handoff_target:
  - "Systems Steward"
authors:
  - "Antigravity"
tags: [agents, rag, multi-agent, orchestration, evolution]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Agent Systems"
created: "2026-04-05"
updated: "2026-04-05"
---

# HERA Paper Analysis

## Placement
- Destination: `research/reference/knowledge/agent-systems/2026_Li_HERA`

## Intent
Analyze the HERA (Hierarchical Evolving RAG Agents) framework for its applicability to Nostra's knowledge engine (`037-nostra-knowledge-engine`), RAG pipeline (`051-rag-ingestion-pipeline`), and Cortex's agent orchestration (`133-eval-driven-orchestration`).

## Possible Links To Nostra Platform and Cortex Runtime
- **Adaptive Orchestration**: HERA's optimization of query-specific agent topologies via reward-guided sampling directly informs the future hybrid orchestration workflows where routing logic isn't static but evolves based on success rates (`133-eval-driven-orchestration`).
- **Role-Aware Prompts**: The framework's ability to refine agent behaviors via credit assignment maps well to Cortex's structured agent boundaries and our goals towards self-refining evaluation agents (`126-agent-harness-architecture`).
- **Advanced RAG**: Reinforces patterns for the `051-rag-ingestion-pipeline`, moving beyond simple semantic search to multi-hop, multi-agent reasoning over the knowledge graph.

## Initiative Links
- `133-eval-driven-orchestration`
- `126-agent-harness-architecture`
- `051-rag-ingestion-pipeline`
- `037-nostra-knowledge-engine`

## Pattern Extraction
- **Global Topology Evolution**: Searching and accumulating experience for optimal multi-agent DAGs per query.
- **Local Prompt Evolution**: Refining role-specific prompts across both operational and behavioral axes based on credit assignment.
- **Self-organization**: Using sparse exploration to create compact, high-utility agent networks rather than monolithic LLM calls.

## Execution Matrix & Scorecard
- **ecosystem_fit**: 4
- **adapter_value**: 3
- **component_value**: 4
- **pattern_value**: 5
- **ux_value**: 2
- **future_optionality**: 5
- **topic_fit**: 5
- **Total**: 28/35
- **Primary Steward**: Research Steward
- **Authority Mode**: recommendation_only

## Adoption Decision
Recommendation: adopt patterns selectively for sophisticated orchestrators in later phases.

Reasons:
- The distinction between global orchestration evolution and local role refinement provides a robust architecture for Cortex's execution plane, separating routing logic from agent persona logic.

Critical critique:
- Like HyEvo, HERA targets auto-generation and optimization of workflows. In Nostra/Cortex, our workflows are predominantly deterministic contracts in the short term. However, this is highly relevant as a pattern for agentic "labs" experiments where we seek autonomous topology optimization. (Similarly analyzed in AlphaEvolve).

## Known Risks
- High complexity in implementing reward models for credit assignment across agents.
- May overcomplicate simple RAG pipelines which might just need better vector embeddings rather than complete multi-agent topologies (a risk seen similarly in TrustGraph integration).

## Suggested Next Experiments
- Define a generic "credit assignment" interface in our Agent execution loops to start tracking success signals from end-user feedback or deterministic evaluators, laying the groundwork for prompt evolution.
- Prototype a subset: multi-agent routing between distinct retrieval functions (e.g. Graph search vs Document vector search) with a static heuristic before moving to full evolutionary orchestration.
