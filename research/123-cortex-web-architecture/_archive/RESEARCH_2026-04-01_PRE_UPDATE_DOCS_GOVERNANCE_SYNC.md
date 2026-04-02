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
updated: "2026-02-22"
---

# Research: Cortex Web Architecture

## Research Question
How should Cortex Desktop and Cortex Web proceed as concurrent execution hosts without violating Nostra/Cortex boundary doctrine or duplicating DPub graph logic?

## Findings
1. Research 118 requires host neutrality and thin-host adapters.
2. Desktop already exposes full Workbench API and contracts through gateway routes.
3. UI drift risks come from duplicate host-specific data models, not from renderer differences.
4. Existing web candidate code in `/Users/xaoj/ICP/apps/cortex-frontend` is transitional and incomplete (no full entrypoint/build lane).

## Conclusion
1. Keep desktop and web both active as Cortex execution hosts.
2. Make gateway runtime host-neutral and shared.
3. Keep graph/path/lens logic canonical in runtime/services, not in host UIs.
4. Keep corpus authority in `/Users/xaoj/ICP/research` with deterministic derived artifacts.

## References
1. `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/PLAN.md`
2. `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/DECISIONS.md`
3. `/Users/xaoj/ICP/research/123-cortex-web-architecture/ANALYSIS.md`
4. `/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md`
5. `/Users/xaoj/ICP/shared/standards/TECHNOLOGY_NEUTRALITY.md`
