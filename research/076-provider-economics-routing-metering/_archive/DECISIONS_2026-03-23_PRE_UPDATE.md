---
id: '076'
name: openrouter-icp-feasibility
title: 'Decisions: OpenRouter-like Service on Nostra/Cortex'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-15'
updated: '2026-02-15'
---

# Decisions: OpenRouter-like Service on Nostra/Cortex

## Decision Log

### D-001: Core Architecture Pattern
**Date**: 2026-01-17
**Status**: PROPOSED
**Decision**: Adopt a **Hybrid Canister + TEE Gateway** architecture.

**Context**:
OpenRouter-like services need to:
1. Store and manage API keys securely
2. Make HTTP outcalls to external LLM providers
3. Route requests with low latency
4. Handle streaming responses

**Options Considered**:

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Pure On-Chain | All logic in canisters | Fully verifiable, simple | API keys visible to subnet, no streaming |
| B. Pure TEE | All logic in TEE gateway | Secure, streaming support | Less verifiable, more complex |
| **C. Hybrid** | Routing on-chain, execution in TEE | Best of both, secure + verifiable | More components |

**Rationale**:
- Routing decisions should be on-chain for transparency and auditability
- API keys MUST be in TEE to prevent exposure
- HTTP outcalls with sensitive payloads route through TEE gateway
- ic-rmcp notes that "API keys can be seen by nodes in subnet" - this is unacceptable for production

**Consequences**:
- Need to deploy both canister and TEE components
- Additional latency for TEE hop (~10-20ms)
- Increased operational complexity

---

### D-002: MCP Integration Approach
**Date**: 2026-01-17
**Status**: PROPOSED
**Decision**: Use **ic-rmcp** as the primary interface for AI agents.

**Context**:
The router needs to expose its capabilities to AI agents and the Nostra ecosystem.

**Options Considered**:

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Custom Candid API | Direct canister calls | Simple, efficient | Not standard, agents need custom integration |
| B. REST-like HTTP | HTTP query endpoints | Widely compatible | Not AI-native |
| **C. ic-rmcp MCP Server** | Standard MCP protocol | Agents discover tools, compatible with OpenAI/Claude integrations | Newer protocol, some limitations |

**Rationale**:
- MCP is becoming the standard for AI tool discovery
- ic-rmcp is purpose-built for ICP
- Aligns with existing `001-multi-project-architecture` Phase 5 plans
- Enables Nostra agents to discover and use the router via standard protocol

**Consequences**:
- Must implement ic-rmcp Handler trait
- Limited to MCP capabilities (no sessions, no bidirectional comms)
- Excellent integration path with Claude, GPT, and other MCP-capable clients

---

### D-003: Payment Model
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Use **ICP Cycles** directly for MVP metering and billing.

**Context**:
Users need to pay for LLM usage. We need a mechanism to charge for requests.

**Options Considered**:

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Free (cycles only) | Users only pay ICP cycles | Low barrier | Unsustainable, no profit |
| **B. Direct Cycles Billing** | Charge cycles per request | Native to ICP, simple MVP | Volatile vs USD, hard to price models accurately |
| C. Pay-per-use (x402) | Micropayments per request/token | Fair pricing, scales with usage | Requires payment infrastructure (Overhead for MVP) |

**Rationale**:
- User explicitly requested "right now use ICP cycles".
- Simplifies MVP architecture (removes anda_x402 dependency for now).
- Can migrate to x402/stablecoin architecture in later phases.
- "Burn rate" model: Canister burns cycles proportional to compute + API cost.

**Consequences**:
- Need stable oracle or manual exchange rate update for Cycle <-> USD conversion (since LLMs charge in USD).
- Users need to attach cycles to calls or maintain a cycle balance in the canister.

---

### D-004: API Key Storage
**Date**: 2026-01-17
**Status**: PROPOSED
**Decision**: Store API keys in **IC-COSE** within **IC-TEE** enclaves.

**Context**:
API keys for OpenAI, Anthropic, etc. are highly sensitive. Exposure would allow unauthorized usage and incur costs.

**Options Considered**:

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Canister stable memory | Keys in encrypted canister storage | Simple | Keys visible to subnet nodes |
| B. User-provided keys | Users bring their own keys | No custody risk | Poor UX, key exposure in requests |
| **C. TEE enclave (IC-COSE)** | Keys stored in hardware-secured enclave | Maximum security, attestation | Operational complexity |

**Rationale**:
- ic-rmcp documentation explicitly warns about subnet visibility
- IC-COSE provides encryption at rest with TEE-backed decryption
- Users trust the router with their requests; we must protect their data
- Attestation provides verifiable security guarantees

**Consequences**:
- Requires IC-TEE deployment
- Key rotation requires TEE coordination
- Higher operational bar

---

### D-005: Provider Adapter Pattern
**Date**: 2026-01-17
**Status**: PROPOSED
**Decision**: Implement **modular adapter pattern** with common interface.

**Context**:
Each LLM provider (OpenAI, Anthropic, Google, etc.) has different APIs, auth, and response formats.

**Design**:
```rust
trait ProviderAdapter {
    async fn complete(&self, request: NormalizedRequest) -> Result<NormalizedResponse, ProviderError>;
    async fn health_check(&self) -> HealthStatus;
    fn models(&self) -> Vec<ModelDefinition>;
    fn name(&self) -> &str;
}
```

**Rationale**:
- Clean separation between routing logic and provider specifics
- Easy to add new providers
- Testable with mock adapters
- Matches OpenRouter's internal architecture

**Consequences**:
- Each provider requires adapter implementation
- Must maintain adapters as APIs evolve
- Normalization may lose provider-specific features

---

### D-006: Fallback Strategy
**Date**: 2026-01-17
**Status**: PROPOSED
**Decision**: Implement **cascading fallback with circuit breakers**.

**Context**:
LLM providers have outages. The router must handle failures gracefully.

**Strategy**:
1. Primary model fails → retry with exponential backoff (max 2 retries)
2. If still failing → circuit breaker opens for that model
3. Fallback to next-best model matching capability requirements
4. If no fallback available → return error with explanation

**Circuit Breaker Settings**:
- Threshold: 3 failures in 60 seconds
- Cooldown: 5 minutes
- Half-open: Allow 1 request to test recovery

**Rationale**:
- Users expect high reliability
- OpenRouter's key value prop is resilience
- Circuit breakers prevent cascading failures

**Consequences**:
- Need health monitoring infrastructure
- State management for circuit breakers
- May route to more expensive fallback models

---

### D-007: Streaming Support
**Date**: 2026-01-17
**Status**: DEFERRED
**Decision**: Defer streaming to Phase 2+; initial release uses non-streaming.

**Context**:
OpenRouter supports streaming responses. ICP canisters have limitations on HTTP response streaming.

**Options Considered**:

| Option | Description | Feasibility |
|--------|-------------|-------------|
| A. Canister streaming | Direct from canister | Not supported by IC |
| B. TEE gateway streaming | TEE handles stream, client connects to TEE | Feasible but complex |
| C. Polling | Client polls for chunks | Works but poor UX |
| **D. Defer** | Non-streaming MVP first | Simplest path to value |

**Rationale**:
- Streaming adds significant complexity
- Many use cases (entity extraction, reports) don't require streaming
- TEE gateway streaming is feasible for Phase 2
- Ship value faster with non-streaming first

**Consequences**:
- Initial UX may feel slower for long generations
- Chat interface will need "generating..." indicator
- Clear Phase 2 feature

---

## Technology Choices

| Category | Choice | Rationale |
|----------|--------|-----------|
| Canister Language | Rust | ic-rmcp is Rust, TEE libs are Rust |
| MCP SDK | ic-rmcp | Purpose-built for ICP |
| TEE Framework | IC-TEE (LDC Labs) | Production-ready, active development |
| Key Storage | IC-COSE | Encryption at rest with TEE |
| Payments | anda_x402 | x402 micropayments, multi-token |
| Agent Registry | anda_registry | Decentralized discovery |
| Metrics DB | anda-db | Optimized for agent memory |

---

### D-008: User-Provided API Keys
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Support **BOTH** service-managed and user-provided API keys.

**Context**:
Some users may want to use their own enterprise credits or avoid service markups. Others want convenience.

**Rationale**:
- User explicitly requested support for both.
- Maximizes flexibility:
    - **User Keys**: Privacy-conscious, brings own rates. Key passed ephemerally or stored encrypted per-session.
    - **Service Keys**: Convenience, "batteries included".

**Consequences**:
- Request object needs optional `api_key` field.
- If `api_key` is present, bypass billing logic (charge only small fee for routing).
- Need distinct security handling for ephemeral keys (ensure they aren't logged).

### D-012: MVP Provider Prioritization
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Prioritize **OpenAI** and **Anthropic** for MVP.

**Context**:
MVP cannot support all providers immediately.

**Rationale**:
- User request: "prioritize OpenAi, Anthropi".[sic]
- These two cover >80% of current high-quality use cases (Reasoning, Coding, Creative).

### D-013: Integration Scope
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Cover ALL AI needs outlined in `013`, `014`, `008`, `020`.

**Context**:
The router must service the entire Nostra ecosystem.

**Scope**:
- **013 (Workflow)**: Async task execution.
- **014 (Agents)**: Chat completions, tools.
- **008 (Contribution)**: Entity extraction, type classification.
- **020 (Library)**: Summarization, semantic indexing.

---

### D-009: Token Economics
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Support **Both** ICP Cycles and Nostra Token (Future).

**Context**:
Should payments be exclusively in ICP native cycles or a custom Nostra token?

**Rationale**:
- MVP uses ICP Cycles for simplicity.
- Future support for Nostra Token is required (User feedback).
- Specifics of Nostra token integration are deferred to tokenomics and merchant-of-record planning.

### D-010: On-Chain Model Integration
**Date**: 2026-01-17
**Status**: DECIDED
**Decision**: Integrate **DeepSeek** (1.5B/7B) as an on-chain provider.

**Rationale**:
- User Explicit "YES".
- Demonstrates unique ICP capability ("AI on blockchain").
- Provides a decentralized fallback when external APIs fail.

### D-011: Multi-Modal Support
**Date**: 2026-01-17
**Status**: RESEARCH NEEDED
**Decision**: Launch feasibility study for multi-modal capabilities.

**Context**:
Vision and audio support are becoming standard, but face technical challenges on ICP (payload size, processing).

**Action**:
- Investigate technical environment and challenges.
- Assess capabilities of TEE gateway for large payloads.
