---
id: 095
name: personal-space-substrate
title: 'Research Initiative 095: Personal Space Substrate'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-01'
updated: '2026-02-25'
---

# Research Initiative 095: Personal Space Substrate

> **Status**: Active | **Phase**: Research & Architecture
> **Created**: 2026-02-01 | **Last Updated**: 2026-02-01

## Executive Summary

This initiative researches and validates the architectural fit of "Personal Space" capabilities within the Nostra protocol. The core decision is to implement Personal Space as a **semantic substrate** for meaning, history, and attribution, while explicitly **excluding** asset custody, financial settlement, and legal enforcement layers to avoid architectural bloat and regulatory coupling.

---

## 1. Problem Statement

How do we support personal assets (RWAs, files, credentials, NFTs) in a way that enriches the user's knowledge graph without turning Nostra into a wallet, exchange, or legal registry?

### The Core Tension

- **Pro**: Users need a "home" for their digital property, credentials, and creative works.
- **Con**: Adding financial logic, token balances, and custodial enforcement creates massive complexity and regulatory surface area ("bloat").

---

## 2. Architectural Decision

**Decision**: Nostra will function as a **Personal Space Semantic Layer**.

We will support **Asset References** (claims about assets), not Assets themselves.

### Truth Table: Scope of Support

| Domain | Covered Now | Needs Extension | Out of Scope (Intentional) |
| :--- | :---: | :---: | :---: |
| Personal Knowledge | ✅ | — | — |
| Files / Docs / ePubs | ✅ | — | — |
| Credentials (Semantic) | ✅ | Minor | — |
| Rewards (Symbolic) | ✅ | — | — |
| **Assets as Reference** | ⚠️ | **✅ (This Initiative)** | — |
| RWA Verification | ❌ | ⚠️ | ✅ |
| Asset Custody | ❌ | ❌ | ✅ |
| Financial Logic | ❌ | ❌ | ✅ |

---

## 3. Implementation Strategy

### 3.1 New Contribution Types

We will introduce specific types to handle "Property as Reference."

#### `AssetReference`
A semantic pointer to an external asset. Examples: "My deed", "My NFT", "My diploma".

*   **Usage**: referencing a deed held in a legal registry, or an NFT held in an Ethereum wallet.
*   **Fields**:
    *   `externalSystem`: (e.g., "Ethereum", "LandRegistry")
    *   `externalIdentifier`: (e.g., Token Address, Parcel ID)
    *   `verificationType`: (Self-Asserted, Oracle-Backed)

#### `CredentialReference`
A formal claim of achievement or capability.

*   **Usage**: "PhD in Computer Science", "Certified Kubernetes Administrator".
*   **Fields**:
    *   `issuer`: Organization ID
    *   `issueDate`: Time
    *   `evidence`: Link to artifact

### 3.2 Personal Space Archetype

A Personal Space is simply a `Space` configured with:
*   **Privacy**: Private by Default
*   **Roles**: Single Owner (Sovereign)
*   **Context**: Rooted in the User's Personal Graph

---

## 4. Bloat Analysis (Validation)

**Is this bloat?**

*   **❌ YES** if we added: Native token balances, NFT minting, On-chain custody, RWA enforcement logic.
*   **✅ NO** if we add: A semantic wrapper (`AssetReference`) that allows the graph to *talk about* assets without *holding* them.

**Conclusion**: This initiative explicitly adopts the "Reference" approach to avoid bloat while achieving semantic completion.

---

## 5. Next Steps

1.  [x] **Schema Implementation**: Add `#assetReference` and `#credentialReference` to `graph.mo` and `kip.mo`.
2.  [x] **Frontend Support**: Create cards for displaying these references in the UI (A2UI Schema updated).
3.  [x] **Integration**: Allow these references to be attached to Profiles (Personal Spaces).

---

## 6. Polymorphic Block Resolution (2026-02-25)

> Alignment with Initiative 124: Universal Polymorphic Block.

The `AssetReference` and `CredentialReference` types map directly onto Polymorphic Block payloads:

| Contribution Type | Polymorphic Payload | Implementation |
|:---|:---|:---|
| `AssetReference` | `pointer` | The `pointer` string contains the external URI (e.g., `eth://0x.../tokenId`). Verification metadata lives in `relations` or an accompanying `structured_data` block. |
| `CredentialReference` | `structured_data` | The JSON payload stores `issuer`, `issueDate`, `evidence` fields. The block is renderable via a standard A2UI `CredentialCard` widget. |

This ensures Personal Space data flows through the same CRDT sync, persistence, and graph edge projection as all other Nostra content types.
