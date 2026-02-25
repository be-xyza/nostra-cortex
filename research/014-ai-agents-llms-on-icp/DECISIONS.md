---
id: '014'
name: ai-agents-llms-on-icp
title: 'Decisions Log: AI Agents & LLMs'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions Log: AI Agents & LLMs

**Context**: Strategic technical decisions for AI integration on ICP.

## DEC-001: Hybrid Compute Architecture
*   **Context**: Canisters are too computationally constrained for LLM inference.
*   **Decision**: Adopt **3-Tier Architecture**:
    1.  **Brain** (Canister): Orchestration, State, World Model.
    2.  **Memory** (Hybrid): Symbolic Graph (On-chain) + Vector DB (Off-chain/Hybrid).
    3.  **Compute** (Off-chain): External "AI Workers" or TEEs for heavy lifting.
*   **Status**: DECIDED

## DEC-002: Agent Memory Standard (KIP)
*   **Context**: Need a standard way for agents to read/write shared knowledge.
*   **Decision**: Adopt **LDC Labs KIP (Knowledge-memory Interaction Protocol)**.
*   **Rationale**:
    *   Optimized for Neuro-Symbolic AI (Graphs).
    *   Built-in monetization/governance ("KnowledgeFi").
    *   Native ICP standard.
*   **Status**: DECIDED
*   **See Also**: [STUDY_KIP_VS_ALTERNATIVES.md](./STUDY_KIP_VS_ALTERNATIVES.md)

## DEC-003: AI Worker Pattern
*   **Context**: How to bridge interaction between Canisters and LLMs.
*   **Decision**: Use "Polling based AI Workers". Canisters queue jobs; external workers poll, execute, and callback.
*   **Status**: DECIDED

## DEC-004: Client Agent Protocol Authentication
*   **Context**: CLI tools and non-browser clients mapping to "Off-chain Compute" or "Memory" layers need to authenticate with Nostra to fetch data and submit Proposals/Reflections.
*   **Decision**: Standardize a "Client Agent Protocol" using SIWE or API keys specifically designed for headless AI agents.
*   **Status**: DECIDED
*   **See Also**: [016-nostra-skills-sync-service-use-case](../016-nostra-skills-sync-service-use-case/RESEARCH.md)
