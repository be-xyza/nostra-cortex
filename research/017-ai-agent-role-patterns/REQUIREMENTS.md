---
id: '017'
name: ai-agent-role-patterns
title: 'Requirements: BMAD Pattern Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: BMAD Pattern Integration

## Core Requirements

### 1. Agent Identity & Persona
- **REQ-001**: System MUST support defined Agent Roles (Analyst, PM, Architect, DEV, QA).
- **REQ-002**: Each Agent MUST have a persistable "Persona" including Name, Role, Voice, and Principles.
- **REQ-003**: Agent definitions MUST be portable (e.g., Markdown/YAML format) but statically identifiable on-chain.

### 2. Context & Memory ("The Sidecar")
- **REQ-004**: Agents MUST have access to persistent, session-independent memory.
- **REQ-005**: In Nostra, this memory MUST be stored as nodes in the Knowledge Graph (`ContextNode`), not just local files.
- **REQ-006**: Agents MUST be able to query their specific memory context before taking action.

### 3. Workflow Patterns
- **REQ-007**: Workflows MUST follow the "Progressive Disclosure" pattern (step-by-step execution).
- **REQ-008**: Critical workflows MUST adopt the "Tri-Modal" structure (Create -> Validate -> Edit).
- **REQ-009**: Workflows MUST be "Continuable," allowing human or agent hand-offs between steps.

### 4. Integration with Nostra Types (`008`)
- **REQ-010**: Agent outputs MUST map to standard Contribution Types:
    - Analyst Output -> `Report` / `ResearchInitiative`
    - PM Output -> `Requirements`
    - Architect Output -> `Design` / `Plan`
