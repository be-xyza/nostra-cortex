---
id: 079
name: merchant-of-record
title: 'Research: Merchant of Record (MoR) Inspiration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Merchant of Record (MoR) Inspiration

## Objective
Research evolving Nostra/Cortex into a Merchant of Record system, drawing inspiration from [Polar](https://github.com/polarsource/polar) and [Blooms](https://github.com/im-Kazmi/Blooms).
Identify patterns, capabilities, processes, and features to port or enhance.
Determine integration strategy (library vs workflow) and modularity.

## Sources
- **Polar**: `polar/server/polar` (Local)
- **Blooms**: (To be analyzed)

## Analysis

### Polar (Python/FastAPI/Postgres)
Polar is a comprehensive Merchant of Record platform tailored for developers. Its architecture is highly modular and data-centric.

#### Key Data Models
1.  **Organization**: The central tenant entity.
    -   Manages settings (billing, emails, notifications).
    -   Has `status` (created, review, active) indicating a compliance workflow.
    -   Owns `Products` and `Customers`.
2.  **Product & Price**: Separation of concerns.
    -   `Product`: The logical item (Name, Description, Media).
    -   `ProductPrice`: The billing configuration (Amount, Currency, Type: Recurring/One-time).
    -   Allows a single product to have multiple prices (e.g., Monthly vs Yearly).
3.  **Subscription**: The recurring relationship.
    -   State machine: `trialing`, `active`, `past_due`, `canceled`, `incomplete`.
    -   Supports `Meters` for usage-based billing.
    -   Links `Customer` to `Product` and `Price`.
4.  **Benefits**: A unique feature.
    -   Products grant `Benefits` (e.g., Discord Access, File Download).
    -   `BenefitGrant` tracks the fulfillment of these benefits to a user.

#### Key Workflows
-   **Checkout**: A complex service orchestrating customer creation, payment intent (Stripe), and subscription initialization.
-   **Webhooks**: Extensive event system (`checkout.created`, `subscription.updated`) to sync with external systems.
-   **Compliance**: `Organization` status checks (`is_under_review`, `blocked_at`) embedded in the model.

### Blooms (Next.js/Hono/Stripe)
Blooms focuses on a modern, lightweight MoR experience, positioning itself as an open-source alternative to LemonSqueezy.

#### Key Features
-   **Tech Stack**: Next.js (Frontend), Hono (Backend), Postgres, Stripe Connect.
-   **Focus**: Digital products, subscriptions, and license keys.
-   **Architecture**: Tightly coupled with Stripe for payment processing, abstracting it into a user-friendly dashboard.

## Strategic Fit for Nostra

Nostra can adopt **Polar's rich data model** and **Blooms' modern UX sensibility**.

### Core Capabilities to Port
1.  **Schema Design**: Adopt Polar's `Product` -> `Price` -> `Benefit` separation. This fits perfectly with Nostra's Graph architecture.
2.  **State Machines**: Implementing Subscription and Order lifecycles as definitive state machines.
3.  **Benefit System**: The "Benefit" concept maps well to Nostra's "Capabilities" or "Access Rights" – buying a product grants access to a specific workflow or data space.

## Integration Recommendations

### 1. Library Approach (`nostra.merchant`)
*   Create a standard library defining the core schemas (`Product`, `Order`, `Subscription`, `Invoice`).
*   This ensures data interoperability between different merchant agents.

### 2. Workflow Approach (The "Engine")
*   The business logic (checkout flow, dunning, benefit granting) should be implemented as **Workflows**.
*   Users "install" the Merchant capabilities by importing the library and deploying the standard workflows.

### 3. Modularity
*   **Opt-in**: The system is completely opt-in. A user only becomes a "merchant" by instantiating these schemas.
*   **Extensible**: Users can fork the standard checkout workflow to add custom logic (e.g., "Add user to custom DAO after purchase") without forking the entire platform.
