---
id: '073'
name: opensass-repositories-catalog
title: 'Research: OpenSass Repositories Catalog'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: OpenSass Repositories Catalog

## Objective
Catalog all 27 repositories under the `opensass` organization to compare against the current Nostra implementation and validate/enrich the tech stack.

## Context
OpenSass focuses on Rust-based full-stack and WASM components (Dioxus, Axum, MongoDB).

## Findings
Searching for repositories... (Processing 27 results)

### Technical Stack Analysis
| Layer | OpenSass Standard | Nostra/Cortex Match | Relevance |
|-------|-------------------|----------------------|-----------|
| Frontend | Dioxus (v0.6/0.7) | Dioxus (Tauri) | **Critical** |
| Backend | Axum / Dioxus Fullstack | Rust Worker / Axum | **High** |
| Database | MongoDB / BSON | ICP Canisters / Stable Mem | Medium (Pattern Match) |
| UI Libs | modular-rs (input, table, etc.) | Custom Labs | **High (Accelerator)** |
| Auth/Security | JWT / Argon2 | Internet Identity | Medium |

### Key Findings
1. **Dioxus Ecosystem Maturity**: OpenSass proves that Dioxus is production-ready for complex SaaS (aibook, eldflow). Their use of `0.6.3` features like `fullstack` and `router` aligns with our plans.
2. **Modular Component Pattern**: Repos like `input-rs` and `table-rs` use a "feature-gated" approach to support Yew, Dioxus, and Leptos. This is a superior pattern for Nostra's open-source library goals.
3. **Deployment Agnosticism**: Use of `railway.toml` and Docker-ready Axum backends shows a clear path for hybrid (on-chain/off-chain) deployments.

### Comparative Analysis: Cortex Desktop vs OpenSass
- **Alignment**: Both historically used Dioxus as the primary UI driver. *(Note: Cortex Desktop is migrating away from Dioxus to React per DEC-123-004, while Nostra Frontend retains Dioxus)*.
- **Acceleration**: We can adopt `input-rs`, `table-rs`, and `toast-rs` immediately to replace manual HTML/CSS in `library_lab.rs`.
- **Inspiration**: The `os` CLI (opensass/cli) is a great reference for a "Nostra CLI" that could manage project templates and component injection.
