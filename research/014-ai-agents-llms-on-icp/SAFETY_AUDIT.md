---
source: research/reference/topics/agent-systems/ironclaw
audit_date: 2026-02-21
status: active
initiative: 014-ai-agents-llms-on-icp
---

# Agent Safety Audit — IronClaw Comparison

Audit of IronClaw's prompt injection defense layers against Cortex's current security posture. This document informs the `cortex-agents` crate design before any live agent execution begins.

---

## Defense Layer Comparison

| Defense Layer | IronClaw | Cortex Current | Assessment |
|---------------|----------|----------------|------------|
| **Severity routing** | Block / Warn / Review / Sanitize (classification-based) | `GateOutcome`: `Pass \| Warn \| RequireReview \| RequireSimulation \| Block` in `ports.rs` | **Covered** — the four ironclaw severity levels map exactly onto `GateOutcome`. No new routing needed. |
| **Tool output wrapping** | Tool results are tagged with context markers before being injected into the LLM prompt | `CortexTool.parse()` boundary exists (JSON → ActionTarget). No explicit tag wrapper yet. | **Partial** — the parse boundary is the right place for a wrapper. Not yet implemented. |
| **Pre-LLM pattern detection** | `aho-corasick` + regex on inbound content before it enters the context window | Not defined in `cortex-domain` or `cortex-runtime` | **Gap** — pre-LLM input stage has no detection layer |
| **Content sanitization** | Tiered policy rules (severity-assigned, configured per-agent) | Not defined | **Gap** — deferred to governance gate |
| **Credential protection** | Credentials injected at host boundary; never visible to WASM tool code | `AuthorityGuard.execute_guarded()` handles mutation without exposing credentials | **Covered** — the authority guard boundary is equivalent |
| **SSRF prevention** | Allowlist proxy for WASM tool HTTP calls | Not yet defined (no tool HTTP calls exist yet) | **Future gap** — relevant when tools make outbound HTTP calls |

---

## Disposition

### Covered by existing architecture

`GateOutcome` and `EpistemicAssessment` in `ports.rs` provide severity routing and epistemic assessment at the **mutation** level — after the agent has reasoned and proposed an action. This is the most critical boundary and it is already modeled.

Credential protection is covered by the `AuthorityGuard` pattern — agents never hold raw credentials.

### The real gap: pre-LLM input stage

IronClaw's most distinctive defense is **before** the LLM sees any content: inbound messages are scanned for injection patterns before they enter the context window. Cortex currently has no equivalent.

**Recommendation**: When the `cortex-agents` crate is created (122 Phase 1), add a lightweight tool-output wrapper that:
1. Tags tool results with a clear delimiter (e.g., `<tool_result id="…">`) before context injection
2. Strips or escapes known injection patterns from tool output strings before the delimiter wrap

This is a light pre-processing pass — not a full policy engine. The governance gate remains the authoritative enforcement point.

### SSRF gap (future)

Not relevant until tools make outbound HTTP calls. When that point arrives, require a configuration-driven allowlist for tool HTTP origins (modeled on ironclaw's proxy approach). Document this as a pre-requisite for any tool that lists `http` in its capability manifest.

---

## What We Are Not Building

Consistent with 122 §6:

- Not adding a standalone Policy Engine to `cortex-domain` or `cortex-runtime`
- Not importing `aho-corasick` into the core libraries
- Not building a prompt management layer

The governance gate is the policy engine. Pre-LLM defense is a light contextual wrapper — a formatting decision, not an inference system.

---

## Sequencing

| Action | When |
|--------|------|
| Tool-output delimiter wrapper | On `cortex-agents` crate creation (122 Phase 1) |
| SSRF allowlist for tool HTTP | When first tool declares `http` capability |
| Pattern detection consideration | After baseline agent execution is live and measurably needed |
