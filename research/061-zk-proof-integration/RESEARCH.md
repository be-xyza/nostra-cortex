---
id: '061'
name: zk-proof-integration
title: 'Research Initiative 061: ZK Proof Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research Initiative 061: ZK Proof Integration

## 1. Goal Description
To establish a **capability layer** for Zero-Knowledge Proofs (ZKPs) within Nostra/Cortex that resolves the "Nostra Paradox" (Interconnection vs. Sovereignty) without imposing premature complexity. We anchor this strategy on the **zCloak Network / icp-zk-maze** pattern, utilizing ICP as a verification coprocessor while keeping Spaces sovereign.

## 2. Strategic Stack Decision
Based on deep efficiency and architectural analysis, Nostra adopts the following ZK stack:

### 2.1 Primary Anchor: zCloak Network (icp-zk-maze pattern)
*   **Role**: Identity & Credential Verification Spine.
*   **Status**: **Primary Reference Implementation** (Default Provider).
*   **Why**: Fully ICP-native "Cloaking Layer" that offloads verification to canisters.
*   **Fit**: Aligns with Nostra's "Linked-but-Private" membership model. Spaces generate proofs; Cortex (via zCloak) verifies them.

### 2.4 Parallel Track: Membership Claim Schema
*   **Objective**: Define a canonical "Membership Claim" schema used by all proof levels (0-3).
*   **Status**: **Defined** in `research/046-nostra-system-standards/PROPOSED_SCHEMAS.md#5-the-membership-claim-nostraidentitymembership`.
*   **Benefit**: Ensures that whether a claim is proven by signature (Level 0) or ZK (Level 3), the data structure remains identical for the Cortex Policy Engine.

### 2.2 Cryptographic Substrate: PLONK (Plonky2 / Arkworks)
*   **Role**: The mathematical engine for proof generation.
*   **Deployment**: Abstracted behind `ProofSpec` interfaces. Not hard-coded into Cortex logic.
*   **Why**: Efficient verification on ICP (approx 0.2s/proof) and strong browser Wasm support.

### 2.3 Attestation Plugin: zkTLS (the3cloud/zktls)
*   **Role**: Specialized plugin for *external* web integrity (e.g., proving GitHub contributions).
*   **Status**: Complementary tool, not the core identity backbone.

## 3. The Proof Ladder
We employ a tiered verification model to minimize compute costs:

| Level | Mechanism | Cost (Cycles/UX) | Use Case | Tech Stack |
| :--- | :--- | :--- | :--- | :--- |
| **0** | **Signature** | Minimal | Identity, Authorship | Ed25519 (Internet Identity) |
| **1** | **Hash Commitment** | Low | Data Integrity | SHA-256 / Poseidon (Merkle Trees) |
| **2** | **Attestation** | Medium | Compliance Assertions | Verifiable Credentials (VCs) |
| **3** | **ZK Membership** | High | **Non-inspectable Guarantees** | **zCloak / Plonky2 Circuits** |
| **4** | **zkTLS** | Very High | External Web Truths | zkTLS Provider |

## 4. Phased Integration Strategy

### Phase 0: Proof-Ready Architecture (Completed)
*   Defined `ProofCarryingMessage<T>` and `Policy::RequiresProof`.
*   Implemented `ProofLevel` enum in `types.rs`.

### Phase 1: Attested Labs (Current)
*   **Objective**: Test the "Gating" UX flow using **Level 0/1 (Signatures)**.
*   **Action**: Build `AttestedLab` component to simulate locked/unlocked states based on a valid `ProofEnvelope` signature.
*   **Key**: Validate the "Async Gating" pattern (locking UI while verifying).

### Phase 2: zCloak Integration (Completed/Integrated)
*   **Objective**: Deploy the first true Level 3 circuit.
*   **Action**: Integrate `zCloak` canister interfaces to verify a generic "Membership" proof.
*   **Reference Backend**: `zkmaze_backend` (`7n7be-naaaa-aaaag-qc4xa-cai`) on IC mainnet.
*   **Pattern**: `icp-zk-maze` (Client generates proof -> Canister verifies).
*   **Performance Targets**: Verification time ~0.2s, Cycle cost < 5M cycles (approx 2 cents).

#### 2.1 zCloak Candid Interface (Discovered)
The core verification API from the zCloak ZK Coprocessor:

```candid
// zk_verify: (verifier_signature, program_hash, public_inputs) -> (status, attestation, output_vec)
zk_verify: (text, text, text) -> (text, text, vec text);
```

**Parameters**:
| Param | Description |
|:------|:------------|
| `verifier_signature` | Canister signature of verifier service module |
| `program_hash` | Hash of the zkVM program (circuit identifier) |
| `public_inputs` | Serialized public inputs for verification |

**Returns**:
| Return | Description |
|:-------|:------------|
| `status` | Verification status ("success" / "failure") |
| `attestation` | Signed attestation hash (threshold ECDSA) |
| `output_vec` | Additional verification outputs |

#### 2.2 Supported ZK Systems
zCloak's Cloaking Layer supports: **Miden, RISC0, SP1, Jolt, PLONK** (our target), and is extensible for new systems.

#### 2.3 MembershipProof Schema (Proposed)
```rust
pub struct MembershipProof {
    pub claim: MembershipClaim,        // From nostra.identity.membership schema
    pub circuit_hash: String,          // Identifier for the ZK circuit used
    pub proof_bytes: Vec<u8>,          // Serialized PLONK proof
    pub nullifier_hash: String,        // Prevents double-proving
}

pub struct MembershipClaim {
    pub space_id: String,
    pub role: String,
    pub member_since: u64,
    pub merkle_root: String,
}
```

#### 2.4 Integration Flow
1. **Client**: User's Space generates a `MembershipProof` using PLONK circuit (browser Wasm).
2. **Frontend**: Packages proof into `ProofEnvelope<MembershipClaim>` with `ProofLevel::ZK`.
3. **Cortex**: Receives envelope, extracts proof, calls `zk_verify` on zCloak canister.
4. **zCloak**: Verifies proof, returns threshold-signed attestation.
5. **Cortex**: Caches attestation, grants access based on policy.

#### 2.5 Candidate Circuit: "Linked-but-Private" Membership
*   **Circuit Purpose**: Prove membership in a Space without revealing identity.
*   **Public Inputs**: `space_merkle_root`, `nullifier_hash`
*   **Private Inputs**: `member_principal`, `membership_proof_path`
*   **Constraint**: User is in the Merkle tree without revealing which leaf.

### Phase 3: Agent Constraint Proofs (Long Term)
*   **Objective**: Verify constraint compliance for autonomous agents.
*   **Scope**: Prove *path execution* (e.g., "I visited Node A then Node B") rather than full compute correctness.

## 5. Technical Implications
*   **No zkVM-first**: We reject running full RISC0/SP1 VMs for identity checks due to browser heavy-lift.
*   **Cortex Cleanup**: Cortex remains a *Policy Engine*, not a *Verification Engine*. It delegates verification to the zCloak layer.

## 6. Recommendation
**Proceed with Phase 1 (Attested Labs)** to validate the application flow, while simultaneously beginning the **zCloak Integration** research (Phase 2).
