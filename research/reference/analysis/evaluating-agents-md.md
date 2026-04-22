---
id: evaluating-agents-md
name: evaluating-agents-md
title: Evaluating AGENTS.md Paper Analysis
type: reference_analysis
project: cortex
status: draft
portfolio_role: satellite
execution_plane: cortex
authority_mode: recommendation_only
reference_topics: [agent-systems]
reference_assets:
  - "research/reference/topics/agent-systems/evaluating-agents-md"
evidence_strength: moderate
handoff_target:
  - "Systems Steward"
authors:
  - "Codex"
tags: [agents, context, context-files, benchmarks]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Agent Systems"
created: "2026-03-29"
updated: "2026-03-29"
---

# Evaluating AGENTS.md Paper Analysis

## Placement
- Destination: `research/reference/topics/agent-systems/evaluating-agents-md`

## Intent
Evaluate the impact of repository-level context files on agent task performance, specifically for refining our own Constitutional Framework and `AGENTS.md` practices.

## Possible Links To Nostra Platform and Cortex Runtime
- Strongly influences our core `AGENTS.md` and Constitutional Framework rulesets.
- Instructs us to enforce minimal requirements in human-written context files to avoid increasing inference costs and making tasks harder.

## Pattern Extraction
- **Context File Optimization**: The paper proves that unnecessary requirements in `AGENTS.md` reduce task success rates and increase cost.
- **Behavioral Changes**: Agents tend to respect context instructions but explore broader file surfaces, driving up token usage. Thus, concise directive boundaries are critical.

## Execution Matrix & Scorecard
- **ecosystem_fit**: 5
- **adapter_value**: 3
- **component_value**: 3
- **pattern_value**: 5
- **ux_value**: 2
- **future_optionality**: 4
- **topic_fit**: 5
- **Total**: 27/35
- **Primary Steward**: Research Steward
- **Authority Mode**: recommendation_only
