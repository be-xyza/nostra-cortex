---
id: '017'
name: ai-agent-role-patterns
title: 'Research: BMAD Agent Roles & Patterns'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: BMAD Agent Roles & Patterns

## 1. Executive Summary
The BMAD (Breakthrough Method of Agile AI Driven Development) framework provides a structured "Agent-as-Code" approach to software development. It moves beyond simple "vibe coding" by employing specialized AI agents with distinct roles, persistent memory, and sequential workflows.

This research analyzes BMAD's core patterns to inform **Nostra's Agent Architecture** (`014`) and the **Self-Building Ecosystem** (`012`).

## 2. Core Concepts

### A. The "Agent-as-Code" Paradigm
Agents are not just prompts; they are defined entities with:
- **Persona**: Role, Identity, Voice, Principles.
- **Capabilities**: Mapped to interactive menu commands.
- **Context**: Persistent memory (often in a "sidecar" folder) handling project history.

### B. Two Agent Types
1.  **Simple Agents**: Single-file, focused, session-only memory. (e.g., A commit message generator).
2.  **Expert Agents**: "Sidecar" equipped. They maintain persistent context across sessions, handle multi-step workflows, and coordinate with other agents.

### C. The Workflow Engine: "Progressive Disclosure"
Workflows are structured sequences of markdown files (`step-01`, `step-02`...) that:
- **Focus**: The LLM sees only one step at a time.
- **Continuity**: Progress is tracked, allowing "Continuable Workflows" across sessions.
- **Tri-Modal Pattern**:
    - **Create**: Build from scratch.
    - **Validate**: Check against standards (QA).
    - **Edit**: Modify while maintaining compliance.

## 3. Role Taxonomy (The "Squad")

BMAD defines a standard team of agents mapped to development phases:

| Phase | Agent | Role | Responsibilities |
| :--- | :--- | :--- | :--- |
| **Analysis** | **Analyst** (Mary) | Research | Market research, Product Briefs, Ideation. |
| **Planning** | **PM** (John) | Requirements | PRDs, Epics, Stories, Success Metrics. |
| **Planning** | **UX** (Sally) | Design | User Journeys, Wireframes, UX Specs. |
| **Solution** | **Architect** (Winston) | Tech Design | System Arch, ADRs, Technical Standards. |
| **Exec** | **SM** (Bob) | Process | Sprint Planning, Story Prep, Retrospectives. |
| **Exec** | **DEV** (Amelia) | Coding | Implementation, Code Review, Testing. |
| **Quality** | **TEA** (Murat) | QA | Test Strategy, ATDD, Automation. |

## 4. Integration Analysis

### Applicability to "Nostra on Nostra" (`012`)
The BMAD roles map almost 1:1 with the actors needed for the "Nostra Self-Building" loop:
- **User Feedback** -> **Analyst** (Synthesizes into a Brief).
- **Brief** -> **PM** (Converts to Requirements).
- **Requirements** -> **Architect** (Designs the Nostra Graph schema changes).
- **Design** -> **Workflow Engine** (Generates the "Contribution Path").

### Applicability to "AI Agents on ICP" (`014`)
BMAD provides the *behavioral* layer for the infrastructure defined in `014`.
- **Infrastructure**: `Motoko Maps` (Memory), `AsyncExternalOp` (Execution).
- **Behavior**: BMAD Agents (Process).
- **Gap**: BMAD relies on local files for memory. **Nostra's innovation** will be to move this "Sidecar Memory" *on-chain* to the Knowledge Graph.

## 5. Technology Patterns to Port

1.  **The "Sidecar" Pattern**:
    - *Current*: Local folder with markdown files.
    - *Target*: A `Space` or `Project` container in Nostra holding `ContextNodes`.

2.  **Tri-Modal Workflows**:
    - Adopt the **Create/Validate/Edit** pattern for all Nostra Contribution Types (`008`).
    - *Example*: A `Proposal` can be *Created* by an agent, *Validated* by a Governance Agent, and *Edited* by the community.

3.  **Agent-Specific Context**:
    - Give every Agent specific "Instructions" and "Principles" stored as `ConceptNodes` in the graph, which they query before acting.

4.  **Schema Enforcement**:
    -   The **Architect** agent must enforce [040-nostra-schema-standards](../040-nostra-schema-standards/RESEARCH.md).
    -   Usage: Check proposed types against the "Standard Index" before approving.

5.  **Recursive Delegation (The "Boss" Pattern)**:
    -   *Source*: Agent Zero (`call_subordinate.py`).
    -   *Target*: Standardization of a "Manager" role that does *no work* other than breaking down tasks and spawning "Worker" agents (Subordinates).

6.  **Extension Hooks**:
    -   *Source*: Agent Zero (`agent.py` hooks).
    -   *Target*: A standard "Interceptor" layer for every agent role that runs *before* and *after* every LLM call for safety and logging.

## 6. Recommendations

1.  **Adopt the Taxonomy**: Use standard names (Analyst, PM, Architect) for Nostra's system agents to reduce cognitive load.
2.  **Port Workflows**: Convert BMAD's "BMM" (Business Method Module) workflows into Nostra **Process Templates** (`013`).
3.  **Hybrid Memory**: Use local files (BMAD style) for dev-loop speed, but "sync" critical memory to on-chain Motoko Maps (LDC Labs KIP).
