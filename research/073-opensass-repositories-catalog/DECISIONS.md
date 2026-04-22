---
id: '073'
name: opensass-repositories-catalog
title: 'Decisions: OpenSass Repositories Catalog'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: OpenSass Repositories Catalog

## Architectural Decisions
- [x] **ADOPT Dioxus Modular Pattern**: We will evaluate adopting the `feature-gated` strategy from `input-rs` for our own UI components to ensure cross-framework compatibility (Dioxus/Tauri + Web).
- [ ] **INTEGRATE OpenSass Components**: Prototype the integration of `table-rs` and `input-rs` into the `Knowledge Engine` lab to accelerate UI development.

## Infrastructure Decisions
- [x] **CLI Reference**: Use `opensass/cli` as a structural reference for the planned `nostra-cli`.
- [ ] **Hybrid Persistence**: Investigate if the MongoDB patterns in `aibook` can be adapted for a "Local First" caching layer in Cortex Desktop before syncing to ICP.
