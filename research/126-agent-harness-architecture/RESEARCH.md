---
id: "126"
name: "agent-harness-architecture"
title: "Research: Agent Harness Architecture"
type: "research"
project: "nostra"
status: complete
authors:
  - "User"
tags: ["agent-harness", "governance", "replay", "evaluation"]
created: "2026-02-24"
updated: "2026-02-24"
---

# Research: Agent Harness Architecture

## Objective
Define a governed, replayable, and low-bloat execution harness for agent workflows in Cortex while preserving Nostra platform authority boundaries.

## Positioning
- Initiative 126 extends initiative 122's MVK runtime direction.
- 126 does not replace 122; it adds authority ladder enforcement, lifecycle observability, evaluation-loop gating, and replay protocol coverage.
- v1 cutline is intentionally narrow: `L0-L2` runtime support with `L3-L4` fail-closed.

## Key Questions
1. How should `AgentExecutionRecord` map into canonical global events without introducing a parallel envelope?
2. What is the minimal enforceable authority contract for v1 that preserves governance boundaries?
3. How can evaluation and replay be introduced with deterministic artifacts and low integration risk?

## Conclusions
1. Lifecycle records should emit through CloudEvent-compatible canonical envelope contracts with strict required payload keys.
2. L1 should materialize proposal bridge artifacts locally until governance canister proposal submit APIs are available.
3. L2 should require governance scope checks plus evaluation-loop allow decisions before apply.
4. Replay artifacts should persist deterministic hashes and lineage references for every terminal run outcome.

## Implementation Outcome (2026-02-24)
- Delivered gateway-first implementation across domain/runtime/gateway/services with no parallel infrastructure.
- Enforced v1 authority cutline (`L0-L2` implemented, `L3/L4` fail-closed).
- Added lifecycle sink rollout controls (best-effort default, fail-closed optional) and validation coverage.
- Verified parity and protocol contract alignment through gateway parity + descriptor/coverage checks.

## Residual Risk
- Unrelated workspace issue persists outside 126 scope: `cortex-domain` test-path include mismatch in branding fixture includes.

## Evidence and References
- [PLAN.md](./PLAN.md)
- [REQUIREMENTS.md](./REQUIREMENTS.md)
- [DECISIONS.md](./DECISIONS.md)
- [VERIFY.md](./VERIFY.md)
- [research/122-cortex-agent-runtime-kernel/RESEARCH.md](../122-cortex-agent-runtime-kernel/RESEARCH.md)
