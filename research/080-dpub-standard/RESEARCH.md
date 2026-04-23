---
id: 080
name: dpub-standard
title: 'Research Initiative 080: DPub Standard'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research Initiative 080: DPub Standard

## 1. Executive Summary

This initiative analyzes and formalizes the adoption of the "DPub Standard" within Nostra. We validate the proposal as a strictly additive, high-leverage architectural extension that unifies our existing primitives: **Contributions**, **Workflows**, **Rich Text**, and **Agents**.

**Conclusion**: The "DPub" is not a new primitive, but a **System Concept**—a higher-order composition of existing Nostra primitives that serves as the primary "Knowledge Application" for the platform.

## 2. Strategic Alignment

The proposal aligns with Nostra's core tenets:
1.  **"Everything is a Contribution"**: The DPub itself is a composite Contribution.
2.  **"Execution is First-Class"**: The DPub drives execution (Workflows) rather than just informing it.
3.  **"History is Sacred"**: The "Edition" concept leverages our immutable versioning to provide citation stability.
4.  **"AI-Native"**: The DPub is structurally designed to be read, maintained, and expanded by Agents.
5.  **"Library Standard"**: DPubs serve as the canonical documentation format for Nostra Libraries (018), allowing "Manuals" to be versioned alongside code.

## 3. System Architecture Mapping

We map the proposal's concepts directly to Nostra's existing technical specifications:

| User Proposal Concept | Nostra Architectural Primitive | Implementation Path |
| :--- | :--- | :--- |
| **DPub** | `Contribution<DPub>` | New Variant in `008-types` |
| **Chapter / Section** | `Contribution<Essay>` / `NostraBlock` | Re-use `030` Rich Text AST |
| **Edition** | `VersionHash` + `Snapshot` | Immutable reference to a specific Graph State |
| **Living Layer** | `Graph Overlay` | Dynamic queries (e.g., "Latest comments on version X") |
| **Workflow Binding** | `WorkflowDefinitionID` | Link to `013` Engine Definitions |
| **Agent Co-Author** | `AgentBinding` / `SystemPrompt` | Standard Agent Config linked to the DPub context |

### 3.1. The "DPub" as a Graph View
Technically, a DPub is a **Root Node** in the Contribution Graph that defines a specific traversal path (Table of Contents) through a set of children nodes (Chapters/Essays).
*   **Storage**: The DPub object stores the *structure* (the order and hierarchy of IDs).
*   **Content**: The content lives in the linked Contributions.
*   **Benefit**: This allows the same "Essay" to be a Chapter in a DPub, a standalone Article, and a Reference in a Course—simultaneously.

## 4. The "Edition" Architecture

The "Edition" is the critical innovation. It solves the tension between **Living Knowledge** (Web) and **Stable Citation** (Print).

### Implementation Strategy
1.  **Draft Mode (Living)**: The `HEAD` of the DPub points to the usage of `Latest` versions of all child chapters.
2.  **Edition Publication (Frozen)**:
    *   Trigger: `Publish Edition` Workflow.
    *   Action: Recursively resolve all child links to their *current concrete ContributionVersionID*.
    *   Result: A Merkle-dag root hash that guarantees that *this* specific combination of words and code will never change, even if the underlying essays evolve.

## 5. First-Class Agent Roles

We will bind specific System Agents to the DPub schema:

*   **Librarian**: Maintains the integrity of links, updates bibliographies, and suggests "Further Reading" based on new incoming graph nodes.
*   **Summarizer**: Auto-generates the "Abstract" for the DPub and each Chapter for the Explorer View.
*   **Challenger**: (Optional) Scans the "Marginalia" (comments) and generates "Counter-Argument" nodes for the author to review.

## 6. Adoption Recommendation

We recommend immediate adoption of this standard. It provides the high-level "Product Container" that `013` (Workflow) and `030` (Editor) currently lack. It turns "using Nostra" from a technical task (running workflows) into a familiar one (writing/reading a dpub).

## 7. Implementation Results (2026-01-24)

The DPub Standard has been successfully validated through the **Library Lab** implementation.

### 7.1. Performance & Scale
- **Dataset**: 9 Foundational Constitutional Documents (JSON-LD format).
- **Technique**: Recursive expansion of `NostraBlock` AST into Dioxus RSX.
- **Outcome**: Near-instantaneous rendering of complex documents with rich text, headings, and legacy HTML support.

### 7.2. The "Edition" Prototype
- Successfully implemented **Version Badges** (v0.1-alpha, v0.2-experimental) and **Phase Indicators** (Deliberative, Active, Frozen).
- Validated that users distinguish between "Living" documents and "Published Editions" through clear UI signals.

### 7.3. Knowledge Graph Connectivity
- DPubs now include `KnowledgeGraph` metadata, allowing the reader to display "Cortex Overlays" (Priority 4) that validate node lineage and normative references.
