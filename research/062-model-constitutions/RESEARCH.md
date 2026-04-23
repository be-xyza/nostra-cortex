---
id: '062'
name: model-constitutions
title: 'Research Initiative 062: Model Constitutions'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research Initiative 062: Model Constitutions

> **Status**: [Draft]
> **Priority**: High
> **Author**: Agent Antigravity / User
> **Context**: Analysis of "Model Constitutions" (e.g. Anthropic's Claude Constitution) and their impact on Nostra/Cortex behavior, governance, and safety.

## 1. Context & Problem Statement

Model-level constitutions (training-time normative scaffolds, RLHF criteria, inference guardrails) implicitly shape agent behavior. These "hidden co-authors" operate upstream of Nostra's explicit governance, creating potential conflicts with Nostra's core values of pluralism, explicit authority, and imagination.

Question: **How should Nostra architecture account for, utilize, or mitigate these model-level constitutions?**

## 2. Analysis: The Impact of Model Constitutions

### What they are
They are not governance in the Nostra sense. They are:
1.  **Pretraining bias**: What reasoning is reinforced.
2.  **RLHF criteria**: What answers are rewarded.
3.  **Inference guardrails**: Hard refusals and reframing.

### Impact on Nostra
1.  **Epistemic Framing**: Models may avoid strong claims or moral caution overrides exploration, conflicting with Nostra's **Knowledge Integrity Doctrine** (plural interpretations).
2.  **Imagination vs. Safety**: Models may be conservative/sanitized, conflicting with **Nostra Labs** ("The imagination door must remain open when instructed").
3.  **Authority Signaling**: Models may sound authoritative when enforcing hidden values, conflicting with **Space Sovereignty** and **Steward Authority**.

### Risks vs. Benefits
*   **Benefits**: Harm minimization, ethical consistency, de-escalation. Good for operational/high-trust spaces.
*   **Risks**: Normative drift (redefining "acceptable"), False Neutrality, Imagination Suppression, Authority Confusion.

### Key Distinction
| Claude Constitution | Nostra Constitutions |
|:---|:---|
| Implicit | Explicit |
| Training-time | Runtime + Governance |
| Model-centric | Space-centric |
| Safety-first | Evolution-first (with safety) |
| Unforkable | Forkable |
| Single Worldview | Plural Worldviews |

## 3. Nostra's Architectural Advantages

Nostra is well-positioned to exploit these rather than be dominated by them:
1.  **Space Sovereignty**: Conservative models for sensitive spaces; exploratory models for labs. No global behavior mode.
2.  **Explicit Agent Charters**: Agents can declare "This recommendation reflects model safety constraints, not Nostra policy."
3.  **Forking**: One model's refusal is a data point, not a dead end. Reasoning can be forked.
4.  **Knowledge Integrity**: Summaries labeled as interpretations; dissent preserved.
5.  **Governance & Escalation**: Refusals trigger escalation; humans override with rationale.

## 4. Design Recommendations & Implementation Strategy

### A. Explicit Acknowledgment (Agent Metadata)
Agents must declare their underlying model and its constraint profile.
*   **Action**: Update Agent Metadata schema (in `046`) to include `model_constitution_profile`.

```typescript
// Proposed addition to Agent Interface in 046-nostra-system-standards
interface Agent {
  // ... existing fields

  constitution_profile: {
    model_family: string;       // e.g. "Claude 3.5 Sonnet", "GPT-4o"
    safety_posture: "Conservative" | "Balanced" | "Exploratory";
    known_biases: string[];     // e.g. ["refuses_medical_advice", "strong_copyright_compliance"]
    upstream_policy_url?: string; // e.g. "https://www.anthropic.com/constitution"
  };
}
```

### B. Constitution-Aware Agents & Disclosure Pattern
Enable agents to articulate the delta between model norms and space norms.

**The `AgentDisclosurePattern`**:
When an agent refuses or significantly constrains an answer due to upstream policy, it MUST use this pattern:
1.  **Acknowledge**: "I am restricted by my base model's safety guidelines regarding [TOPIC]."
2.  **Differentiate**: "This is a model constraint, not necessarily a Nostra Space policy."
3.  **Offer Fork (If possible)**: "A less restricted model (e.g., local open-source) might be able to explore this."

> **Example**: "I cannot generate code that exploits this vulnerability due to safety guidelines. Use a local security-audit agent for authorized penetration testing."

### C. Refusal as Knowledge
Capture model refusals as signals.

**New Contribution Type: `ModelBiasAnnotation`**
Instead of just logging error text, we create a structured annotation on the prompt/response pair.

```typescript
interface ModelBiasAnnotation extends Contribution {
  type: "ModelBiasAnnotation";
  target_event: string;         // The Interaction ID
  bias_category: "Refusal" | "TonePolicing" | "PrematureClosure";
  severity: 1 | 2 | 3;
  model_id: string;
  rationale: string;            // Why the model refused
}
```

### D. Proposed Charter Amendments (034)
We recommend amending `NOSTRA_AGENT_BEHAVIOR_AUTHORITY_CHARTER.md` to explicitly address model constitutions.

**Proposed Addition: Section 17. Model Transparency**
> "Agents must distinguish between their System Instructions (Nostra logic) and their Base Model Constraints (Upstream alignment). When these conflict, the Agent must disclose the source of the constraint. An agent shall not present a model refusal as a moral truth, but as a technical constraint."

## 5. Integration Plan
*   **System Standards (046)**: Update `Agent` interface with `constitution_profile`.
*   **Cortex Desktop (057)**:
    *   Add "Constitution Widget" to Agent Runner to show `safety_posture`.
    *   Visualize `ModelBiasAnnotation` events in the Error Graph.
*   **Labs (034)**: Update `NOSTRA_AGENT_BEHAVIOR_AUTHORITY_CHARTER.md` with Section 17.


## 6. Next Steps
1.  Draft `AgentDisclosurePattern`.
2.  Formalize `ModelBiasAnnotation`.
3.  Update Agent configurations in `nostra/agents`.
