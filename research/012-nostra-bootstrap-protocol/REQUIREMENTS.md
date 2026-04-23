---
id: '012'
name: nostra-bootstrap-protocol
title: 'Requirements: Personal OS Bootstrap'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Personal OS Bootstrap

> **Status**: DRAFT
> **Source**: [RESEARCH.md](./RESEARCH.md)
> **Context**: Individual User - "Capture -> Route -> Store" Workflow

## 1. Functional Requirements

### 1.1 Capture Interface (The "Door")
- **FR-01: Unified Input**:
    - A single, persistent input field (chat/voice) available across all views.
    - Treats user prompts as "API requests" not creative writing.
- **FR-02: Multi-Modal Support**:
    - Text input (primary).
    - Voice input (future phase).
    - File/Image attachment (future phase).

### 1.2 Router (Intent Classification Engine)
- **FR-03: Intent Detection**:
    - Classify user input into actionable "Next Actions" (e.g., `Create Project`, `Add Note`, `Search`).
    - Must extract specific intent, not generic chat.
- **FR-04: Clarifying Questions**:
    - If confidence is below threshold, system MUST ask clarifying questions.
    - Questions should be targeted to improve classification confidence.
- **FR-05: Workflow Binding**:
    - Route classified intent to the appropriate `013` Workflow Definition.

### 1.3 Confidence Scoring
- **FR-06: Confidence Threshold**:
    - Every classification MUST have a confidence score (0.0 - 1.0).
    - Minimum threshold for auto-routing: `0.75` (configurable).
- **FR-07: Confidence Factors**:
    - **Correctness**: Is the user input well-formed?
    - **Relevance**: Does the input match a known workflow pattern?

### 1.4 Memory Store (Library Integration)
- **FR-08: Library Schemas**:
    - Support for domain-specific entity types:
        - People, Projects, Ideas, Notes
        - Documents, Receipts, Invoices
        - Business Filings, Legal Docs
        - Meeting Minutes, SOPs, Financial Records
- **FR-09: Inbox/Log**:
    - Central Audit Trail Index.
    - All captured inputs logged with timestamp, confidence score, and routing decision.

### 1.5 Proactive Surfacing (Cron Service)
- **FR-10: Periodic Summaries**:
    - Open Tasks / Proposals.
    - Blockers / Challenges.
    - Insights / Discoveries.
- **FR-11: Frequency Configuration**:
    - User-configurable (Daily, Weekly, On-Demand).

### 1.6 Feedback Handle
- **FR-12: Correction Mechanism**:
    - User can mark a routing decision as "incorrect".
    - System logs correction for reinforcement learning.

## 2. Technical Requirements

### 2.1 Infrastructure
- **TR-01: Workflow Engine Integration**:
    - Personal OS consumes `013-nostra-workflow-engine` for execution.
- **TR-02: Knowledge Engine Integration**:
    - Memory Store consumes `037-nostra-knowledge-engine`.
- **TR-03: Schema Manager Integration**:
    - Library schemas defined in `026-nostra-schema-manager`.

### 2.2 Data Model
- **TR-04: Inbox Entry**:
    ```
    {
      id: Text,
      timestamp: Nat,
      rawInput: Text,
      classifiedIntent: ?Text,
      confidenceScore: Float,
      routedToWorkflow: ?Text,
      status: { #pending | #routed | #failed | #corrected }
    }
    ```

## 3. User Experience Requirements

### 3.1 The "Capture Door" UI
- **UX-01**: Always-visible input field (bottom of screen or floating action).
- **UX-02**: Typing indicator / processing animation during classification.
- **UX-03**: Clear feedback on routing decision ("Creating a new Project...").

### 3.2 Dashboard
- **UX-04**: "Design for Restart" - Show most relevant/actionable info, not just backlog.
- **UX-05**: Proactive surfacing widget for summaries.
