---
id: 089
name: code-standards
title: Rust Code Standards
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-29'
updated: '2026-01-29'
---

# Rust Code Standards

This directory contains the unified Rust tooling configuration for the Nostra/ICP workspace.

## Files

| File | Tool | Purpose |
|------|------|---------|
| `rustfmt.toml` | `cargo fmt` | Formatting rules (Rust 2024 style) |
| `clippy.toml` | `cargo clippy` | Linting configuration |
| `deny.toml` | `cargo deny` | Supply chain governance |

## Usage

```bash
# Format all code
cargo fmt --all

# Run Clippy lints (deny warnings)
cargo clippy -- -D warnings

# Check dependencies (licenses, security, bans)
cargo deny check
```

## ADR Reference

See [ADR-001: Rust Linting Strategy](./089-code-standards/ADR-001-rust-linting-strategy.md) for the full decision record.
