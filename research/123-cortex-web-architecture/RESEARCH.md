---
id: "123"
name: "cortex-web-architecture"
title: "Cortex Web Architecture Research"
type: "research"
project: "cortex"
status: active
authors:
  - "X"
tags:
  - "dual-host"
  - "host-neutral"
  - "workbench"
  - "dpub"
created: "2026-02-22"
updated: "2026-03-28"
---

# Research: Cortex Web Architecture

## Research Question
How should Cortex Desktop and Cortex Web proceed as concurrent execution hosts without violating Nostra/Cortex boundary doctrine or duplicating DPub graph logic?

## Findings
1. Research 118 requires host neutrality and thin-host adapters.
2. Desktop/eudaemon already expose shared Workbench API and contracts through gateway routes.
3. UI drift risks come from duplicate host-specific data models, not from renderer differences.
4. `cortex-web` now proves the host-neutral pattern with a canonical conversation surface: `/ws/chat` stays transport-only, the gateway resolves heap context and server-backed history, and the runtime dispatch path is provider-runtime first.
5. Existing web candidate code in `/Users/xaoj/ICP/apps/cortex-frontend` is historical and non-authoritative; `cortex/apps/cortex-web` is the active host.

## Conclusion
1. Keep desktop and web both active as Cortex execution hosts.
2. Make gateway runtime host-neutral and shared.
3. Keep graph/path/lens logic canonical in runtime/services, not in host UIs.
4. Keep conversation history canonical in runtime/heap-backed projections, with host-local state limited to cache and UX convenience.
5. Keep corpus authority in `/Users/xaoj/ICP/research` with deterministic derived artifacts.

## References
1. `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/PLAN.md`
2. `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/DECISIONS.md`
3. `/Users/xaoj/ICP/research/123-cortex-web-architecture/ANALYSIS.md`
4. `/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md`
5. `/Users/xaoj/ICP/shared/standards/TECHNOLOGY_NEUTRALITY.md`
