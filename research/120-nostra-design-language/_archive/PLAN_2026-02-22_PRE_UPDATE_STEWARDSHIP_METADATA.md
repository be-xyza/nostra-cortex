---
status: "active"
owner: "Architect"
tags: ["ndl", "a2ui", "governance", "ui", "verified-projection"]
created: "2026-02-19"
---

# Plan: Nostra Design Language (NDL)

## 1. Objective
Establish the **Nostra Design Language (NDL)** as a "Constitutional Interface Layer" for the Nostra Platform. This initiative defines the visual grammar, temporal motion semantics, and anti-spoofing protocols required to render governance artifacts securely.

## 2. Rationale
While Initiative `074-cortex-ui-substrate` handles the mechanical UI rendering (Dioxus, A2UI), it does not define **what** is rendered or **how** truth is visually guaranteed.
A malicious AI agent could spoof a "Ratified" badge via an A2UI payload if the renderer blindly trusts the input. NDL solves this by defining an immutable visual grammar and a "Verified Projection Architecture" that forces the runtime to query the graph directly for constitutional components, completely decoupling UI declaration from truth state.

## 3. Scope
**In Scope:**
*   NDL Specification Document (Philosophy, Grammar, AI Contracts).
*   NDL JSON Schema for machine-enforceable rules.
*   NDL Component Registry mapping (Tier 1 vs Tier 2 vs Tier 3).
*   Verified Projection Architecture design (A2UI `*Ref` components).
*   Cortex Runtime `NdlComponentResolver` strategy.

**Out of Scope:**
*   Building the `cortex-desktop` Dioxus UI Substrate (handled by `074`).
*   Building standard, non-constitutional components like Buttons or TextFields (handled by `045-component-library-labs`).

## 4. Key Deliverables
- [x] `DESIGN_LANGUAGE.md` (Authoritative Specification)
- [x] `NDL_JSON_SCHEMA_v0.1.json` (Machine-Validation Schema)
- [x] `NDL_COMPONENT_REGISTRY_v0.1.md` (Tier Classification)
- [x] `A2UI_NDL_POINTERS_0_1.json` (Extension Catalog for Verified Projection)
- [x] Integrate Surface Boundary Protocol into Research 014, 018, 008

## 5. Timeline & Dependencies
*   **Dependencies**: Requires `074-cortex-ui-substrate` (for Dioxus rendering) and `103-agent-client-protocol-alignment` (for A2UI spec updates).
*   **Status**: Phase 1 (Scaffolding), Phase 2 (Verified Projection Architecture), and Phase 3 (Surface Boundary Doctrine) are complete. NDL is now architecturally aligned with Cortex Runtime.
