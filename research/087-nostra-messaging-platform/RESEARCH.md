---
id: 087
name: nostra-messaging-platform
title: 'Research: Nostra Unified Messaging Platform'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Nostra Unified Messaging Platform

## Metadata
- **ID**: 087
- **Title**: Nostra Unified Messaging Platform
- **Author**: Antigravity (Architect)
- **Status**: ARCHITECTED
- **Date**: 2026-01-28
- **Related**: 086-cobudget-integration-patterns, 028-a2ui-integration-feasibility, 074-themes-system, 085-nostra-file-infrastructure

---

## Executive Summary

This initiative defines the architecture for Nostra's **Unified Messaging Platform**, a powerful, interactive communication layer built on A2UI. Unlike traditional "static text" messaging, Nostra messages are **living, interactive surfaces** that leverage ICP-native capabilities (canisters, stable storage, vetKeys) to enable rich workflows directly within the inbox.

This platform unifies user-to-user DMs, space-to-user notifications, and system alerts into a single, cohesive experience.

---

## Vision: Messages as Interactive Surfaces

Traditional messaging is passive. Nostra messaging is active:

> **A Nostra Message is an A2UI Surface** — a themeable, time-aware artifact that can contain components, request feedback, display visualizations, trigger workflows, and expire or transform based on context.

### Core Capabilities
1.  **Interactive**: Embedded forms, buttons, and tools (not just links).
2.  **Themed**: Inherits context (Space/Sender) branding via [074-themes-system](../074-themes-system/RESEARCH.md).
3.  **Secure**: End-to-end encrypted via vetKeys, with granular consent models.
4.  **Workflow-Aware**: Triggers Cortex workflows (`bounty.claim`, `proposal.vote`) directly.

---

## Architecture Specification

### 1. The `NostraMessage` Envelope

The core data structure connecting the system.

```typescript
interface NostraMessage {
  // Identity & Routing
  message_id: Principal;
  thread_id: string;
  sender: NostraAddress;       // e.g. "alice.design-guild.nostra"
  recipients: NostraAddress[]; // e.g. ["bob.core.nostra", "dev-team.space.nostra"]

  // Content Components
  subject: string;
  preview_text: string;        // Fallback for notifications

  // A2UI Payload
  display_surface: {
    surface_id: string;
    catalog_version: "0.9";
    components: A2UIComponent[];
    data_model: Record<string, any>;
    theme?: ThemeBinding;
  };

  // Lifecycle & Metadata
  lifecycle: MessageLifecycle;
  metadata: Record<string, string>;

  // Security Layer
  encryption?: EncryptionEnvelope; // vetKeys scheme
}
```

### 2. Addressing & Routing

**Standard**: `user.space.nostra`

*   **User Identity**: `alice`
*   **Context Scope**: `design-guild` (Space)
*   **Network**: `nostra`

Routes messages to the specific "Inbox" context of a user within a Space, allowing separation of concerns (Work vs. DAO vs. Personal).

### 3. Persistence Strategy

**Hybrid & Configurable**:
*   **Ephemeral**: (NATS) for typing indicators, presence.
*   **Index-Only**: (Canister) for system logs, low-value notifications.
*   **Full Persistence**: (Canister Index + File Storage) for DMs, Proposals, Bounties.

---

## Direct Messaging Strategy

**Philosophy**: "Text-First, App-Enabled"

Nostra DMs aim for the speed and simplicity of modern chat apps (Signal, Telegram) while enabling powerful A2UI integrations when needed.

### 1. Unified Protocol, Specialized Rendering

We use the single `NostraMessage` envelope for **both** Chat and Apps, but distinguish by `content_type`:

*   **Conversational (Chat)**:
    *   **Payload**: Plain text, Markdown, or simple media.
    *   **Rendering**: Native chat bubble (high performance, standard UI).
    *   **Experience**: Instant, scrolling, familiar.
*   **Interactive (App)**:
    *   **Payload**: Full A2UI `display_surface`.
    *   **Rendering**: Embedded "Card" or "Frame" within the chat stream.
    *   **Experience**: Structured, transactional (Voting, Payments).

### 2. Trust Tiers & Security

To prevent "A2UI Phishing" (malicious forms sent by strangers), we enforce strict rendering limits based on relationship status:

| Trust Tier | Sender Status | Allowed Content | A2UI Capabilities |
|------------|---------------|-----------------|-------------------|
| **Tier 0: Stranger** | Unknown / First Contact | Plain Text Only | **Disabled** (No components) |
| **Tier 1: Contact** | Accepted Request | Rich Media, Links | **Restricted** (Polls, Static Info) |
| **Tier 2: Verified** | Friend / Whitelisted | Full Capabilities | **Full** (Payments, complex forms) |

> **Security Rule**: Tier 0 messages initiate a "Message Request" state. The receiver must explicitly "Accept" to upgrade to Tier 1 and enable rich content.

### 3. "Social Media" Features via A2UI

We map standard mobile messaging features to A2UI constructs:

*   **Reactions**: Managed via `message_metadata` updates (not full components).
*   **Replies**: Natively handled via `thread_id` and `parent_id`.
*   **Media**: `Image`, `Video`, and `Audio` components are optimized for "Chat Bubble" containers (rounded corners, click-to-expand).

---

## Interactive Component Patterns

These patterns demonstrate how standard A2UI components are composed to solve cross-initiative use cases.

### Pattern 1: Governance & Voting (General)
*Use Case: DAO Proposals, Quick Polls*

*   **Components**: `Card`, `MultipleChoice`, `Button`
*   **Data Model**: `{ "proposal_id": "123", "options": [...] }`
*   **Action**: Triggers `governance.vote` workflow.
*   **Visuals**: Progress bars for current results (if public).

### Pattern 2: Commerce & Bounties (086-Cobudget)
*Use Case: Task Requests, Sale Offers, Service Listings*

*   **Components**: `Card` (Elevated), `Text` (Reward styling), `Button` (Primary CTA)
*   **Example**: "Fix Login Bug - Reward: 500 ckUSDC"
*   **Action**: `bounty.claim` or `payment.send`.
*   **Lifecycle**: Expires when claimed or deadline reached.

### Pattern 3: DevOps & System Alerts (033-Monitor)
*Use Case: CI/CD Status, Canister Health, Error Reporting*

*   **Components**: `Alert` (Status Color), `CodeBlock` (Log snippet), `Row` (Actions)
*   **Example**: "🚨 Build Failed: frontend-canister-v2"
*   **Actions**:
    *   [Rollback] (Triggers `deployment.rollback`)
    *   [View Logs] (Deep link to observer)
    *   [Silence] (Snooze alert)

### Pattern 4: Financial Allocations (086-Cobudget)
*Use Case: Distributing funds to team members*

*   **Components**: `List` (Recipients), `Slider` (Amount/Percentage), `Text` (Total)
*   **Example**: "Distribute Q1 Bonus: 10,000 ckBTC"
*   **Interactivity**: Sliders adjust allocation per user; total validation.
*   **Action**: `finance.distribute_batch`.

### Pattern 5: Access Control & Permissions (085-Files)
*Use Case: Requesting access to private artifacts*

*   **Components**: `Card` (File Preview), `UserAvatar` (Requestor), `Row` (Approve/Deny)
*   **Example**: "@bob requests access to 'Q3-Strategy.pdf'"
*   **Actions**:
    *   [Approve (Read-Only)]
    *   [Approve (Edit)]
    *   [Deny]

### Pattern 6: Theme Previews (074-Themes)
*Use Case: Sharing themes or proposing UI changes*

*   **Components**: `Container` (Preview Frame), `Button` (Apply)
*   **Example**: "Check out the new 'Dark Neon' theme!"
*   **Interactivity**: The message itself renders *using* the proposed theme for immediate preview.
*   **Action**: `theme.apply_local`.

---

## Security & Privacy (ICP-Native)

> See detailed analysis in **[Risk Assessment & Mitigations](./RISK_ASSESSMENT.md)**.

### vetKeys E2E Encryption
Private messages use ICP's threshold encryption (vetKeys).
*   **Keys**: Derived per-message from `(sender, recipient, message_id)`.
*   **Access**: Only the recipient can derive the decryption key via canister call.
*   **Plausible Deniability**: Sender signatures are verified but can use ring signatures for anon contexts.

### Action Verification
A2UI actions in messages are sandboxed:
1.  **Allowlist**: Only specific registered workflows can be triggered.
2.  **Consent**: Sensitive actions (payments) require secondary confirmation.
3.  **Origin**: Sender identity is cryptographically verified.

---

## Implementation Roadmap

### Phase 1: Foundation
*   Define `NostraMessage` Candid schema.
*   Implement basic Inbox Canister (CRUD).
*   Port rustmailer event hooks to canister logic.

### Phase 2: A2UI Integration
*   Embed `display_surface` in message schema.
*   Implement client-side renderer for message context.
*   Build patterns 1 (Voting) and 2 (Bounties).

### Phase 3: Security & Federation
*   Integrate vetKeys for encryption.
*   Add multi-inbox/persona routing.
*   Implement Matrix/Email bridging (Federation).

---

## Decision Record

*   **Addressing**: `user.space.nostra` (Selected)
*   **Persistence**: Configurable Hybrid (Selected)
*   **Framework**: A2UI (Selected)
*   **Encryption**: vetKeys (Selected)
