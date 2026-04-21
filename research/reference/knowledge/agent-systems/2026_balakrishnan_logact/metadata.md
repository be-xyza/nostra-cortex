---
schema_version: "2.0"
artifact_id: "paper-2026-balakrishnan-logact"
artifact_type: "paper"
title: "LogAct: Enabling Agentic Reliability via Shared Logs"
authors:
  - "Mahesh Balakrishnan"
  - "Ashwin Bharambe"
  - "Davide Testuggine"
  - "David Geraghty"
  - "David Mao"
  - "Vidhya Venkat"
  - "Ilya Mironov"
  - "Rithesh Baradi"
  - "Gayathri Aiyer"
  - "Victoria Dudin"
year: 2026
publisher: "arXiv"
upstream_url: "https://arxiv.org/html/2604.07988v1"
source_files:
  - path: "source.html"
    sha256: "5f31ea20afe3b7d09c87f6ba5564890c77989ff02343dcb8433eb85d3b0baffa"
    mime_type: "text/html"
topic: "agent-systems"
tags:
  - "agent-reliability"
  - "shared-logs"
  - "fault-tolerance"
  - "agent-safety"
  - "workflow-runtime"
status: "reviewed"
nostra_cortex_scope:
  - "cortex-runtime"
  - "workflow-engine"
  - "agent-approval"
  - "event-log"
  - "replay-and-recovery"
initiative_refs:
  - "118"
  - "123"
  - "125"
primary_steward: "Systems Steward"
authority_mode: "recommendation_only"
escalation_path: "Systems Steward review is required before introducing shared-log protocol changes, new execution authorities, or policy-enforcing agent voters into production."
lineage_record: "Intake created on 2026-04-13 from a direct research request to assess LogAct against Nostra/Cortex runtime and governance patterns."
review_date: "2026-04-13"
confidence_score: 0.84
source_reliability: "primary_source_arxiv_html"
validation_proof:
  method: "manual_primary_source_review"
  evidence_refs:
    - "https://arxiv.org/html/2604.07988v1"
    - "research/reference/knowledge/agent-systems/2026_balakrishnan_logact/source.html"
    - "research/reference/analysis/paper-2026-balakrishnan-logact.md"
  notes: "Validated against the arXiv HTML source and cross-checked against local Nostra/Cortex architecture and runtime files."
standards_alignment:
  overall_weighted_score: 4.2
  dimensions:
    modularity:
      score: 5
      applicability: "core"
      rationale: "LogAct cleanly separates Driver, Voter, Decider, and Executor roles around typed log contracts."
    composability:
      score: 5
      applicability: "core"
      rationale: "Typed entries, hot-swappable voters, and mailbox/policy events compose well with evented runtime surfaces."
    confidence_integrity:
      score: 5
      applicability: "core"
      rationale: "The design makes intended actions visible before execution and adds auditable approval checkpoints."
    portability:
      score: 3
      applicability: "supporting"
      rationale: "The abstraction is portable, but practical adoption depends on a durable shared-log substrate and sandbox/isolation model."
    durability:
      score: 5
      applicability: "core"
      rationale: "Durable append-before-act logging is the central mechanism for recovery, replay, and audit."
    accessibility:
      score: 2
      applicability: "supporting"
      rationale: "The paper is systems-oriented and production-focused; direct end-user accessibility patterns are limited."
topic_alignment:
  dimensions:
    runtime_auditability:
      score: 5
      rationale: "The AgentBus is explicitly positioned as a durable audit trail for intentions, results, mail, votes, and policy."
    execution_safety:
      score: 5
      rationale: "Pre-execution voting plus decider quorum gives a concrete safety hook before environment mutation."
    recovery_semantics:
      score: 4
      rationale: "Semantic recovery is compelling, but still probabilistic because LLM introspection remains fallible."
classification_extensions:
  agent_runtime_pattern: "shared-log-deconstructed-state-machine"
  adoption_stage: "prototype-recommended"
adoption_decision:
  verdict: "adopt_selectively"
  rationale: "Strong fit for Cortex runtime audit, approval, and recovery design. Recommended as a reference architecture for typed execution logs, pre-commit policy gates, and supervisor-style introspection rather than a wholesale runtime rewrite."
known_risks:
  - "Semantic recovery and LLM-based voting remain heuristic rather than formally guaranteed."
  - "A shared-log architecture raises implementation complexity and operational cost if applied too broadly."
  - "Policy encoded in logs can create governance confusion unless privilege boundaries and steward authority stay explicit."
  - "Cross-agent concurrency control is left largely future work in the paper and would need a Nostra-native authority model."
suggested_next_experiments:
  - "Prototype a typed Cortex execution log for one agent run surface, with intent, vote, decision, result, and mailbox entries."
  - "Add a rule-based pre-execution approval gate for high-risk mutations, then measure utility and false positives."
  - "Test supervisor-style introspection over existing run/event records to reduce duplicate work across agent swarms."
  - "Map replay/recovery contracts to semantic health checks for long-running workflow tasks."
---

## Summary

LogAct proposes treating an agent as a deconstructed state machine over a durable shared log. The log records intentions before execution, allows pluggable voters to approve or reject those intentions, and captures results, mailbox messages, and policy changes so failures can be recovered and runs can be audited.

## Why This Matters For Nostra Cortex

The paper aligns closely with current Nostra/Cortex goals around durable execution, auditability, approval-gated mutations, and replayable runtime events. It is especially relevant to Cortex runtime work where execution traces already exist but are not yet elevated into a shared, typed, pre-execution control plane.

## Key Takeaways

1. Shared logs become more valuable when they carry intent before side effects, not just events after the fact.
2. Reliability improves when unsafe or high-risk actions pass through independent approval components before execution.
3. Typed mailbox and policy entries suggest a clean way to unify human intervention, cross-agent coordination, and runtime policy changes.
4. Semantic introspection is most promising as an operator-assist or supervisor layer, not as sole recovery authority.
