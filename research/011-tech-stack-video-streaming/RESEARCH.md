---
id: '011'
name: tech-stack-video-streaming
title: 'Research: Video and Audio Streaming on ICP'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Video and Audio Streaming on ICP

**Date**: 2026-01-15
**Status**: DRAFT
**Context**: Investigating `ic-video-storage`, `ic-video`, and other ICP tools for a potential video/audio streaming platform.

## 1. Executive Summary

Research into `ic-video-storage` and `rashansmith/ic-video` reveals two distinct technical approaches for video on ICP:
1.  **On-Chain Storage & Streaming (`ic-video-storage`)**: Uses a "canister-per-video" architecture for sovereign, high-integrity storage. Best for high-value content, NFTs, and archival data.
2.  **P2P Real-Time Communication (`rashansmith/ic-video`)**: Uses WebRTC for ephemeral video calls, using ICP only for signaling and hosting the frontend. Best for meetings and live collaboration.

For the **Lion History** and **Nostra** projects, a hybrid approach is recommended:
*   **Lion History** (Reference Scenario - Cultural Reconstruction): Adopt the `ic-video-storage` pattern (one canister per "Oral Tradition" or "Mythic Data" artifact) to ensure sovereign, immutable preservation of reconstructed narratives.
*   **Nostra** (Knowledge Management): Adopt the WebRTC pattern for "Collaboration Spaces" and a modified storage pattern for attaching "Evidence" to research nodes.

---

## 2. Library Analysis

### 2.1 `ic-video-storage` (Storage & Streaming)
*   **Repository**: `node_modules/ic-video-storage` / [IC-Kryptonic/Video-Canister](https://github.com/IC-Kryptonic/Video-Canister)
*   **Core Architecture**:
    *   **Spawn Canister**: Factory that creates new canisters.
    *   **Video Canister**: A dedicated smart contract that holds *one* video and its metadata.
    *   **Index Canister**: Maps User IDs to their Video Canisters.
*   **Mechanism**:
    *   Uploads video in chunks (chunk size configurable, e.g., 2MB).
    *   Stores chunks in the canister's stable memory.
    *   Streams video via HTTP requests to the canister (likely using `http_request` interface for browser compatibility).
*   **Pros**:
    *   **Sovereignty**: Each video is an independent smart contract. It can be transferred (NFT use case) or owned entirely by a user.
    *   **Isolation**: High traffic on one video doesn't directly block reads on another (different canisters).
    *   **Simplicity**: Codebase is focused on single-responsibility.
*   **Cons**:
    *   **Cost**: High initialization cost per video (see Cost Analysis).
    *   **Latency**: Creating a canister takes seconds. Not suitable for "instant" uploads of short clips.
    *   **Management**: Managing thousands of canisters is more complex than managing thousands of records in one database.

### 2.2 `rashansmith/ic-video` (Real-Time Communication)
*   **Repository**: [rashansmith/ic-video](https://github.com/rashansmith/ic-video)
*   **Core Architecture**:
    *   **WebRTC**: Uses browser-to-browser connections for the actual media stream.
    *   **Signaling**: Uses ICP to exchange "offers" and "answers" (Session Description Protocol) to establish the P2P connection.
*   **Pros**:
    *   **Cost**: Extremely low. Video data does not touch the blockchain.
    *   **Latency**: Real-time (sub-500ms), suitable for conversations.
*   **Cons**:
    *   **No Storage**: Calls are ephemeral. Recording requires a separate solution (client-side recording + upload).
    *   **Peer Dependency**: Quality depends on user internet connection, not server bandwidth.

---

## 3. Cost Analysis & Feasibility

### 3.1 "Canister-per-Video" Model (`ic-video-storage`)

This model requires burning cycles to create a canister and then funding it for storage.

| Item | Cost (Cycles) | Cost (USD Est.) | Notes |
|------|---------------|-----------------|-------|
| **Canister Creation** | ~200 Billion | ~$0.26 | One-time fee per video upload. |
| **Storage (1GB / Year)** | ~4 Trillion | ~$5.00 | Standard ICP storage cost. |
| **Ingress/Egress** | Variable | Low | Read requests (streaming) are free/cheap ("query" calls), but heavy http_request streaming might incur compute costs. |

**Feasibility Verdict**:
*   **High Feasibility**: For **"Premium/Archival"** content. E.g., preserving an important oral history recording for centuries on an immutable ledger.
*   **Low Feasibility**: For **"Social Media"** (TikTok/Instagram). Paying $0.26 every time a user posts a 10-second meme is financially unviable.

### 3.2 Alternative: "Shared Storage" Model (Custom Implementation)
Instead of one canister per video, use detailed large-storage buckets (e.g., Canistore approach).
*   **Cost**: Amortized creation cost. Only pay for storage ($5/GB/Year).
*   **Feasibility**: Higher for general-purpose streaming platforms.

---

## 4. Integration with Research Projects

### 4.1 Reference Scenario: **Lion History** (Cultural Reconstruction)
**Context**: "Lion History" is a **Reference Use Case** used to drive requirements for Nostra's capabilities. It simulates a user collecting suppressed or oral histories.
*   **Use Case**: "The Mythic Data Archive".
*   **Proposal**: Use `ic-video-storage` (Canister-per-Video) for **Oral Traditions** and **Mythic Artifacts**.
    *   When a Lion Historian records a "Songline" or "Ritual Reconstruction," that data becomes a **Sovereign Canister**.
    *   **Why?**: This ensures the narrative is **uncensorable** and **immutable**. It is not just a file in a database; it is a permanent "Monument" on the blockchain. Even if the platform evolves, the canister persists as a testament to that history.

### 4.2 Fit with **Nostra** (Knowledge Graph)
Nostra is the tool for creating and connecting these narratives.
*   **Live Collaboration**: Integrate `rashansmith/ic-video` logic (~WebRTC) for **"Reconstruction Councils"** (or generic "Research Rooms").
    *   *Usage*: A group of historians meeting live to discuss the "Plausibility" of a new theory.
    *   *Mechanism*: A Nostra "Space" can have an active "Meeting Room" tab powered by WebRTC.
*   **Evidence Attachment**: Modify `ic-video-storage` to be a "Media Attachment" system.
    *   *Scenario*: A researcher uploads a video of a traditional dance to support their `Idea` about "Ritual Continuity".
    *   *Integration*: The video becomes a `Node` in the Knowledge Graph (Type: `Artifact`, Subtype: `Video`).

### 4.3 Integration with `motoko-maps-kg`
The `motoko-maps-kg` project provides the underlying graph data structure. Here is how Video fits into the Schema:

1.  **Nodes (Entities)**:
    *   **New Entity Type**: `Artifact` (or distinct `Video` type).
    *   **Payload**: The `Video` Entity stores the `Principal ID` of the **Video Canister** (created via `ic-video-storage`).
    *   **Metadata**: `Duration`, `Format`, `Resolution` are stored as properties on the Graph Node.

2.  **Edges (Relationships)**:
    *   **Evidence Links**: A `Video` Node is linked to an `Idea` Node via an `Evidence` edge (e.g., "Supports").
    *   **Attribution Links**: A `Video` Node is linked to a `Person` Node via an `AuthoredBy` edge.

3.  **Workflow**:
    *   User uploads file in Nostra UI -> Frontend calls `Spawn Canister` -> Gets new `CanisterID`.
    *   Frontend uploads chunks to `CanisterID`.
    *   On completion, Frontend calls `motoko-maps-kg` backend: `create_entity(type=#Artifact, payload={canister: CanisterID, ...})`.
    *   The Video is now a first-class citizen of the Knowledge Graph.

---

## 5. Usage Attribution & Fee Capabilities

Implementing "Pay-Per-View" or "Pay-Per-Listen" (Royalty Distribution) requires precise tracking. On ICP, this presents a tradeoff between **granularity** and **cost**.

### 5.1 The Technical Challenge
*   **Query Calls (Free)**: Standard video streaming uses HTTP GET requests, which are typically "Query" calls on ICP. These are fast and free but **cannot strictly update the state**. You cannot increment a "view_counter" reliably inside a query call.
*   **Update Calls (Paid)**: To reliably update the ledger (e.g., "User X listened for 5 seconds"), you need "Update" calls. These are slower (2s latency) and cost cycles.

### 5.2 Attribution Models

#### Model A: Access-Gated (NFT/Token)
*   **Mechanism**: User makes a **one-time payment** to mint/buy a token (NFT). This token authorizes the user's Principal ID to fetch chunks from the video canister.
*   **Pros**: Simple architecture. Zero ongoing gas cost for tracking. Ideally suited for "buying a movie" or "subscribing to a channel".
*   **Cons**: Binary access (Have it or Don't). Cannot easily track "seconds listened" for pro-rata royalty splits.
*   **Used By**: Trax (NFT music editions), Canistore (Licensing).

#### Model B: Proof-of-Consumption (Periodic Heartbeats)
*   **Mechanism**: Client sends a specific "I am watching" Update Call every 60 seconds.
*   **Pros**: Real-time tracking.
*   **Cons**: High cost (1 Update call * user count * user minutes). Doesn't scale for millions of users.

#### Model C: Wasm-Based Client-Side Reporting (The "Smart Witness")
*   **Concept**: Move the logic from the Server (Canister) to the Client (App) using secure Wasm.
*   **Architecture**:
    1.  **In-App Tracking**: A secure Rust/Wasm module inside the Nostra/Lion History app monitors playback locally. It logs "start", "stop", and "duration" events in the browser's memory.
    2.  **Batch Reporting**: At the end of a session (or every ~10 mins), the Wasm constructs a compressed `AttributionReport`.
    3.  **Signature**: The app signs this report with the user's Principal (Project Identity).
    4.  **On-Chain Settlement**: The app sends **one single** Update Call to the canister with the report. The canister verifies the signature and credits the creators in bulk.
*   **Features/Implications**:
    *   **Scalability**: Reduces blockchain write load by 10x-100x compared to Model B.
    *   **Privacy**: The Wasm logic can "sanitize" data before sending. For example, reporting "User viewed 10 mins of content" without specifying *exactly* which timestamp they paused at, protecting detailed behavioral privacy.
    *   **Trust Model**: Relies on the client not being "hacked". However, since the Wasm is served by the canister, we can verify its hash.
*   **Challenges**:
    *   **Spoofing**: A sophisticated user could technically fake the "Wasm Report" signal.
    *   **Mitigation (Advanced)**: Use **ICP vetKeys**. To watch segment 2, the client *must* request a decryption key from the network. The network only grants Key 2 if the client provides a valid "Proof of View" for Segment 1. This creates a cryptographic "Chain of Viewing" that makes spoofing computationally expensive.

### 5.3 Recommendations for Fee Structure

For **Lion History** and **Nostra**, we recommend different attribution goals:

*   **Lion History (Source Provenance)**:
    *   **Goal**: Integrity & Genealogy ("Who reviewed this source?").
    *   **Example Scenario**: A **"Proof of Study"** gate for contributors.
    *   **Implementation**: Before a user can fork a narrative or cite an Oral History video as evidence, the **Wasm Report** must confirm they have played at least X% of the content.
    *   **Testable Consequence**: The system prevents "drive-by" citations. A citation edge created by a verified viewer gets a "Verified Witness" property, whereas others are marked "Unverified".

*   **Nostra (Royalty/Attribution)**:
    *   **Goal**: Academic Citation ("Who learned from this?").
    *   **Implementation**: Use **Graph-Based Attribution**. If a user links a video node `A` to their new research node `B`, that is a permanent "Attribution Edge".
    *   **Monetization**: If Node B earns money/tokens, a percentage can automatically flow back to Node A via the "ReferencedBy" edge in the Knowledge Graph. This is a **smart-contract driven royalty** based on *utility*, not just *consumption*.

---

## 6. Recommendations (Updated)
> [!NOTE]
> See [PLAN.md](./PLAN.md) for execution roadmap, [DECISIONS.md](./DECISIONS.md) for architecture, and [REQUIREMENTS.md](./REQUIREMENTS.md) for specs.

1.  **Refactor `ic-video-storage`**: Do not use the library as-is regarding the Index Canister (it may be centralized/abandoned). Fork the "Video Canister" code to create a compliant implementation under our control.
2.  **Prototype Hybrid System**:
    *   Use WebRTC for live "Nostra Meetings".
    *   Use "Canister-per-Item" for data-sovereign archives.
3.  **Explore Data Offloading**: See [FEEDBACK.md](./FEEDBACK.md).
