---
id: ''
name: nostra-on-nostra
title: 'Research: Nostra on Nostra (The Self-Building Protocol)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Nostra on Nostra (The Self-Building Protocol)

## 1. Core Question
**"How can we use Nostra to build Nostra?"**

We aim to explore how the research, planning, and development process of the Nostra platform can be enhanced, streamlined, and accomplished *within* Nostra itself. This is the ultimate "dogfooding" exercise: treating the development of the platform as a primary use case.

## 2. The Vision
A decentralized, multi-agent, and participatory ecosystem where:
*   **Research** is not a static document but a dynamic graph of questions, evidence, and hypotheses.
*   **Planning** is a collaborative workflow involving users, developers, and AI agents.
*   **Execution** is streamlined through defined paths (workflows) that any contributor can pick up.

## 3. The "Use Case" / User Story
**Title**: "The Living Roadmap"

**Actors**:
*   **The User**: Provides feedback (passive) or requests features (active).
*   **The Architect (Agent/Human)**: synthesizes feedback into Research Initiatives.
*   **The Contributor (Dev/Agent)**: Selects a defined path to implement a solution.

**Scenario**:
1.  **Input**: A user conducting a survey on "Family Coordination" in Nostra notices a missing feature (e.g., "Private Voting"). They submit this feedback via an in-app poll.
2.  **Capture**: This feedback is captured as a `SentimentNode` in the Nostra Graph, linked to the `Feature:Polls` node.
3.  **Synthesis**: An "Architect Agent" notices a cluster of negative sentiment around `Feature:Polls` regarding privacy. It automatically generates a `ResearchInitiative`: "Private Voting Mechanisms".
4.  **Workflow Generation**: The system proposes a set of workflows for this initiative:
    *   *Path A*: UX Research (Interview users).
    *   *Path B*: Tech Spike (Investigate ZK voting).
5.  **Action**: A developer (human) sees the "Tech Spike" opportunity in their "Recommended Contributions" feed and accepts it.
6.  **Loop**: The output of the code is deployed, and the original feedback providers are notified to verify the fix.

## 4. Key Components Needed
To release this potential, we need:
1.  **Workflow Engine**: A system to define, track, and execute multi-step processes (Research -> Plan -> Dev).
2.  **Graph-Based Feedback**: Direct linkage between user inputs (polls, chats) and system artifacts (features, components).
3.  **Contribution Paths**: A marketplace of "Next Actions" tailored to different skills (Design, Dev, Research), utilizing the **Contribution Types** defined in [008-nostra-contribution-types](../008-nostra-contribution-types/PLAN.md) (specifically `Review`, `Proposal`, `Report`).
4.  **Artifacts Editor**: A tool to create and manage these research documents and assets directly in the Space (See `030-artifacts-editor`).

## 5. Agent Roles (BMAD Integration)
To enable the "Actors" defined above, we will adopt the "Agent-as-Code" patterns from the BMAD framework (See `research/017-ai-agent-role-patterns`).

| Use Case Actor | BMAD Role | Responsibility |
| :--- | :--- | :--- |
| **Architect** | **Analyst / PM** | Synthesize feedback (`SentimentNode`) into `ResearchInitiative` and `Requirements`. |
| **System** | **Architect** | Design technical schemas and validate `ContributionPaths`. |
| **Contributor** | **DEV / QA** | Execute `Tech Spike` or `Review` workflows. |
