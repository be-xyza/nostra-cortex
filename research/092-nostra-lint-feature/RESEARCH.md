---
id: 092
name: nostra-lint-feature
title: 'Research Study: Nostra Lint Feature (The Verification Plane)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-17'
updated: '2026-02-17'
---

# Research Study: Nostra Lint Feature (The Verification Plane)

**Initiative ID**: 092
**Date**: 2026-02-01
**Status**: COMPLETED (MVL)
**Author**: Architect (Winston)

> **Implementation**:
> *   [Source Code](../../scripts/nostra-lint.js)
> *   [Cortex Integration](../../cortex/apps/cortex-desktop/src/services/lint_service.rs)

## 1. Introduction
The Nostra ecosystem is rapidly expanding, encompassing the A2UI protocol, autonomous agents, decentralized publications ("dPubs"), and a growing set of constitutional mandates. As complexity increases, we must transition from implicit conventions to explicit, executable guarantees.

This study proposes **"Nostra Lint"**, not merely as a code formatter, but as a **Protocol Integrity & Constitutional Enforcement System**—the "Verification Plane" of the Nostra operating system.

## 2. The Nostra Verification Plane
To avoid semantic overloading, we define "Nostra Lint" as the UX entry point (`nostra lint`) for a multi-layered verification stack:

### Layer 1: Structural Validator (Deterministic, Fast, Fail-Hard)
*   **Scope**: File system topology, naming conventions, metadata presence.
*   **Examples**:
    *   "Is `research/` sequentially numbered?"
    *   "Does `agent.md` exist and match the schema?"
    *   "Are all Wikilinks in Obsidian notes resolvable?"
*   **Failure**: Blocks build/commit.

### Layer 2: Protocol Validator (Schema, Typing, Fail-Hard)
*   **Scope**: A2UI payloads, KIP commands, Workflow step definitions.
*   **Examples**:
    *   "Does this `SurfaceUpdate` reference a valid Component ID?"
    *   "Is the KIP `UPSERT` command well-formed?"
*   **Mechanism**: Wraps strict validators (e.g., `validator.ts`) into the CLI.

### Layer 3: Semantic Analyzer (Contextual, Graph & Time Aware)
*   **Scope**: Graph logic, Temporal correctness, Cross-entity references.
*   **Examples**:
    *   **Graph Linting**: "Does this Idea node rely on a Decision node that doesn't exist?" (Orphaned edges).
    *   **Temporal Linting**: "Does this workflow sleep >24h without a wake condition?" (Durable execution safety).
*   **Failure**: Warning/Error depending on config.

### Layer 4: Constitutional Auditor (Advisory-First, Governance-Driven)
*   **Scope**: Alignment, Safety, Privacy, Interpretation.
*   **Examples**:
    *   "Does this User Story violate the 'User Agency' constitution?"
    *   "Is the Agent prompting for unsafe actions?"
*   **Mechanism**: LLM-based analysis or heavy static analysis.
*   **Failure**: Graded output (❌ Violation, ⚠️ Concern, ℹ️ Advisory). **Defaults to non-blocking** to avoid political/interpretative deadlocks, unless promoted by governance.

## 3. Advanced Capabilities (Enrichments)

### 3.1. Lint Targets
Nostra Lint treats more than just *files* as targets. It validates the **System Graph**:
1.  **Contribution Graph**: Validating the lifecycle of entities (Idea → Proposal → Decision). Preventing invalid phase transitions.
2.  **Temporal Flows**: Compile-time safety for long-running workflows. Ensuring `await` points are safe and cross-canister calls are idempotent where required.

### 3.2. Space Sovereignty (Composable Constitutions)
Lint rules must honor the sovereignty of Spaces.
*   **Composition**: `Global Rules` + `Space Constitution` + `Repo Overrides`.
*   **Example**: A "Research Space" might enforce `RESEARCH.md` formatting but ban `DEPLOY` commands. A "DAO Space" might require 2-of-3 signatures on all `Proposal` artifacts.

### 3.3. Lint as a Contribution
Because "History is Sacred," verification itself is a contribution.
*   **Artifact**: A `LintReport` is generated, signed, and optionally stored.
*   **Schema**:
    ```json
    {
      "target": "hash(file_or_graph_node)",
      "rule_id": "NOSTRA-001",
      "severity": "ERROR",
      "explanation": "Missing required metadata",
      "auto_fix_applied": true,
      "signer": "agent:architect",
      "timestamp": "2026-01-31T..."
    }
    ```
*   **Benefit**: Provides an unforgeable "Proof of Compliance" token that can gate deployment or interactions with high-stakes contracts.

## 4. Implementation Strategy

### 4.1. Minimum Viable Linter (MVL)
**Status**: ✅ IMPLEMENTED (2026-02-01)
Focus on **Layer 1 (Structural)** and **Layer 2 (Protocol)** immediately.
*   **Tech Stack**: CLI Tool (Node.js/Rust) -> `nostra lint`.
*   **Rules**: Defined in YAML/JSON for now.
*   **Constraints**:
    *   **Auto-fix**: Strictly limited to distinct, reversible operations (renumbering, scaffolding). NEVER auto-fix intent or constitutional issues.

### 4.2. Evolution Path
1.  **v1 (The Guardian)**: CLI + VS Code Extension. Structural & Protocol checks.
2.  **v2 (The Auditor)**: Integration of Graph and Temporal linters. Rules potentially compiled to WASM/Rust crates.
3.  **v3 (The Sovereign)**: Space-scoped constitutions and "Lint as Contribution" reporting.

## 5. Roadmap & Recommendation
1.  **Define Grammar**: Formalize the file/folder structure schema.
2.  **Prototype MVL**: Built a CLI that runs `validator.ts` and checks `research/*` numbering.
3.  **Define "What NOT to Lint"**: Explicitly document boundaries to prevent scope creep into subjective territory.
