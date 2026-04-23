# NCP Integration Patterns & Analysis

This document captures the analysis of the "NOSTRA - Cortex Nostra Code Management" specification and proposes the three most optimal integration patterns for the ICP ecosystem.

---

## 1. The "Contribution Proposal" Envelope (Meaning over Syntax)

### Analysis
Git is effective at tracking *what* changed (the diff), but fundamentally incapable of tracking *why*, the exact reasoning path, the simulated systemic impact, or the alignment with abstract platform goals. The PDF proposes the **Contribution Proposal (CP)** as the primary unit of change.

### Integration Pattern
**Wrap Git Diffs in YAML/JSON CP Envelopes.**
- Instead of treating a Pull Request as the ultimate record of truth, Cortex should treat a Git commit simply as a payload pointer (`git_diff_ref`) inside a larger Nostra object.
- **Implementation:** When an agent (or human) proposes a change, the system generates a CP object containing:
  - `intent` (category & description)
  - `reasoning` (`nostragraph://` URI to the structured reasoning path)
  - `validation_bundle` (evidence, tests passed, regression risk)
  - `ontology_impact` (which systemic concepts this touches)
- **Short-Term Step:** Map these CP YAML structures into Forgejo/GitHub PR descriptions or commit metadata, parsing them via webhooks until a native Nostra UI replaces the forge.

---

## 2. The Multi-Tiered Merge Decision Engine

### Analysis
The document explicitly states that "Stewards approve direction, not implementation." A binary PR merge button forces humans into mechanical review, bottlenecking agent autonomy.

### Integration Pattern
**Replace "Merge" with an Algorithmic Decision Matrix.**
- Merge execution should no longer be a direct user action. Instead, it becomes an orchestration output.
- **Implementation:** Construct a Decision Engine that evaluates incoming CPs based on:
  - **Governance Tier (T0-T4):** Is this a formatting change (auto-merge) or an architectural shift (multi-steward approval)?
  - **Agent Consensus:** Have other runtime agents critiqued, simulated, and validated the CP?
  - **Risk Coefficient:** Calculated from the validation bundle.
- **Short-Term Step:** Implement a pre-merge action (via Temporal workflow or MCP service) that calculates a `consensus_score` and `risk_assessment`. The engine then outputs state decisions (`MERGE`, `DEFER`, `REQUIRE_REVIEW`, `SIMULATE_LONGER`) rather than waiting for a manual click.

---

## 3. Human Stewardship vs. Agent Autonomy Boundaries

### Analysis
The PDF outlines a transition from human-bottlenecked deployments to structured agent autonomy, utilizing "Agent Identity & Trust Scores." Humans remain the constitutional authority, but agents earn autonomy through historical accuracy.

### Integration Pattern
**Decouple Authority from Code Generation.**
- The interface to evolution must present humans with "distilled intelligence" rather than raw diffs.
- **Implementation:** A Steward Dashboard where humans define the *intent space* and review the *impact/risk*, while agent swarms negotiate the actual diff syntax in the background. As agents successfully land T0/T1 changes without regressions, their Trust Score increments, allowing them to propose T2 execution independently.
- **Short-Term Step:** Build the execution-layer metric tracking (`consensus_score` and `trust_score` per Agent ID) into the existing Cortex system logs. Begin routing T0 (formatting, pure non-functional refactors) to auto-merge based strictly on agent consensus, proving the pipeline mechanics.
