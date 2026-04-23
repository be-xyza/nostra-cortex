---
id: '003'
name: nostra-library-economics
title: 'Research: Nostra Library Economics'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Nostra Library Economics

**ID**: 003-nostra-library-economics (formerly schema-marketplace)
**Context**: Defining the economic layer for [Nostra Libraries (N-Libs)](../018-nostra-library-registry/RESEARCH.md).

## The Product: "Nostra Library" (N-Lib)
An N-Lib is not just a schema. It is a **Capabilities Package**.
Users don't pay for data types; they pay for **Outcomes**.

### Components & Value
| Component | Value Prop | Pricing Potential |
|-----------|------------|-------------------|
| **Workflow** | Orchestrates a complex process | High (Saves human time) |
| **Agent Skill** | Expert logic/prompting | Medium (Specialized knowledge) |
| **KG Schema** | Structured data storage | Low (Enabler, commodity) |
| **Seed Data** | Curated lists/configs | Medium (Startup accelerator) |

## The Enforcer: `nostra-cortex`
The engine must enforce the economics. If a user tries to run a "Paid Workflow", `nostra-cortex` must block execution unless a valid license is held.

### Mechanism Hypothesis
1.  **Install Time**: User pays X, mints a `License SBT` (Soulbound Token) or registers in `LicenseRegistry`.
2.  **Run Time**:
    -   Workflow Step triggers `AsyncExternalOp` (e.g., "Run Market Analysis Agent").
    -   Cortex checks: `Does User(Principal) own License(N-Lib-ID)?`
    -   If Yes: Execute.
    -   If No: Prompt user to buy.

## Royalty Splitting (The Viral Loop)
If I build a "Real Estate Deal Flow" Library that uses your "Zillow Scraper" Library, you should get paid.

-   **Deep Dependencies**: A recursive payment or license check is needed.
-   **Bundling**: Can I sell a "Bundle" that includes your license?

## Open Questions
1.  **Updates**: Does the license cover V1 only? Or lifetime updates? (Versioning question).
2.  **Refunds**: Technical impossibility on blockchain? Or held in escrow?
3.  **Governance**: Can a Space "vote" to buy a library for all members?
