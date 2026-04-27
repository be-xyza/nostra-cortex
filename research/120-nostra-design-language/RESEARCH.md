---
id: "120"
name: "nostra-design-language"
title: "Research: Nostra Design Language (NDL)"
type: "research"
project: "nostra"
status: completed
authors:
  - "Nostra Team"
tags: ["ndl", "ui", "governance", "a2ui", "verified-projection", "surface-boundaries"]
created: "2026-02-19"
updated: "2026-02-20"
meta_scope_updated: "2026-04-27"
related:
  - "074-cortex-ui-substrate"
  - "103-agent-client-protocol-alignment"
  - "014-ai-agents-llms-on-icp"
  - "018-nostra-library-registry"
  - "008-nostra-contribution-types"
---

# Research: Nostra Design Language (NDL)

**Date**: 2026-02-20
**Status**: ACTIVE
**Context**: Defines the visual grammar, temporal semantics, and anti-spoofing protocols required to render governance artifacts securely. This document synthesizes the core principles of NDL and specifically details the "Surface Boundary Doctrine" integration.

---

## 1. Executive Summary

The Nostra Design Language (NDL) is a "Constitutional Interface Layer". While overarching substrates like Dioxus and A2UI handle *how* things render, NDL dictates *what* is allowed to render and *what authority* it carries.

The core problem NDL solves is **UI Spoofing**. In an AI-agent-heavy, user-generated-UI world (A2UI), a malicious agent could simply draw a red `[Ratified]` badge on a fake proposal. NDL uses a Verified Projection Architecture to structurally decouple UI declarations from ground-truth constitutional state.

---

## 2. The Surface Boundary Doctrine

### 2.1 The Architectural Tension
A key dilemma in NDL's development was balancing rigid constitutional security against the creative freedom required for space-level applications, games, and rich interactive workflows.
- If NDL is too rigid, developers cannot build immersive apps.
- If NDL is too loose, malicious apps can spoof governance primitives.

### 2.2 Surface Classifications (The Solution)
The Surface Boundary Doctrine resolves this by categorizing every UI surface into one of three strict types:

1.  **Constitutional Surfaces**: The immutable source of truth. Rendered exclusively by the Cortex Runtime. Cannot be modified by applications. Examples: `Decision`, `Institution`, `Profile`.
2.  **Execution Surfaces**: Full creative freedom sandboxes. Applications, games, AI tools, and external workflows render here. They are structurally prohibited from rendering Tier 1 Constitutional Components.
3.  **Hybrid Surfaces**: Views that combine constitutional data with execution context, requiring stringent boundary demarcation.

### 2.3 Visual Containment Rule
To prevent "invisible" Execution Surfaces from mimicking Constitutional Surfaces, the Cortex Desktop runtime enforces a **Visual Containment Rule**.

If a surface declares itself as `execution`, the runtime physically wraps the entire view in a highly visible, non-hideable Dioxus header (`⚡ Execution Surface - Labs Mode Active / Sandbox`). This guarantees the user always knows when they have left the constitutional layer.

### 2.4 The Exchange I/O Doctrine
Execution surfaces cannot mutate constitutional truth by drawing it. Instead, they must interact with the graph via **Exchange I/O**.
App workflows compute state and logic internally, but to affect the Nostra graph, they must submit structured `Contribution` contracts back to the host, which then renders the resulting state via a Constitutional Surface.

---

## 3. Enforcement & Implementation Findings

The Surface Boundaries have been fully implemented and validated across the Nostra hardware/software stack.

### 3.1 NDL Schema Expansion (`NDL_JSON_SCHEMA_v0.1.json`)
The core JSON schema now requires a `surface_type` declaration. Utilizing an `allOf` condition, the schema fundamentally rejects any payload where `surface_type == "execution"` attempts to include authoritative Tier 1 components (like `Decision` or `Institution`).

### 3.2 Native Client Hardening (`ndl_validator.rs`)
The Cortex Desktop Rust client uses an `NdlValidator` that intercepts the raw JSON payload *before* it reaches the A2UI parser. The validator explicitly denies rendering if a spoofing attempt occurs, acting as the final physical protection layer.

### 3.3 A2UI Metadata Context (`renderer.rs`)
The A2UI renderer was updated so `surface_type` inherently flows down the React/Dioxus component tree via `A2UIMeta`. The root renderer intercepts this context and mounts the Visual Containment Frame automatically.

---

## 4. Cross-Initiative Alignment Matrix

To ensure the Surface Boundary Doctrine is legally binding across the entire Nostra ecosystem, the architectural rules were retroactively enforced across adjacent research specs:

| Initiative | Enforcement Action | Finding |
| :--- | :--- | :--- |
| **014 AI Agents** | Added `FR-4.4 NDL UI Integrity` | Agents emitting A2UI are now explicitly prohibited from hallucinating or spoofing ratified governance frames. |
| **018 Library Registry** | Updated `Section 5.3 A2UI Integration` | Library tools that generate interactive UI responses MUST classify as `surface_type = "execution"`. |
| **008 Contribution Types** | Hardened `FR-1 Decision Type` | The `Decision` artifact was bound explicitly as a Tier 1 component that exclusively renders within `constitutional` surfaces. |

---

## 5. Conclusion

The implementation of the Surface Boundary Doctrine successfully secures the Nostra runtime against AI-driven UI spoofing while maintaining 100% of the creative flexibility required for future DApps, games, and autonomous agents. The distinction between "Truth" (Constitutional) and "Play" (Execution) is now physically, visually, and speculatively enforced.

---

## 6. Space-Level Design Standards Meta-Cognition

### 6.1 Current Reality as Truth

Future Space design standards must begin by treating current NDL, A2UI, brand, accessibility, ViewSpec, Cortex Web, and Space capability contracts as locked design reality. Candidate Space profiles do not redefine those contracts. They describe allowable visual identity inside the boundary those contracts already define.

This matters because Space-level design is inherently tempting: richer visual identity can make Spaces feel alive, but the same freedom can blur authority, spoof governance, or create inaccessible decision paths. The right cognition is therefore evaluative before generative.

### 6.2 Evaluation Questions

Every Space design profile, imported design element, or template pack should be evaluated against these questions:

1. Does it preserve the NDL surface boundary and avoid Tier 1 spoofing?
2. Does it preserve A2UI theme-policy metadata and host-neutral rendering?
3. Does it preserve brand policy rather than replacing official brand authority?
4. Does it meet accessibility invariants for perceivability, operability, comprehensibility, and adaptability?
5. Does it fit the Space capability overlay and navigation context?
6. Does it leave generated UI artifacts under ViewSpec governance?
7. Does it provide enough lineage for steward review and later promotion?

### 6.3 Import and Template Posture

External design systems, token packs, templates, and component recipes may be useful as candidate materials. They should enter as `DesignElementImportV1` records, not as direct runtime dependencies or authority sources. Import analysis should classify:

- what primitive is being imported,
- which Space archetype it supports,
- which local contract it maps to,
- which claims must be stripped or rewritten,
- which accessibility and governance risks remain,
- whether the recommendation is adopt, adapt, reject, or steward review.

### 6.4 Hermes Role

Hermes is appropriate for meta-cognition here only if it receives a bounded `DesignAuditUnitV1` containing the profile or import candidate, the locked reality snapshot, and the applicable initiative/standard references. Hermes output remains advisory. It cannot approve, mutate, enforce, import, or publish Space design profiles.
