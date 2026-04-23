---
id: '057'
name: development-brain
title: 'Decisions: 057 Development Brain'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: 057 Development Brain

## DEC-001: Configuration Storage and Singleton
- **Decision**: Use `OnceLock` for a global `CONFIG` singleton in `config_service.rs`.
- **Rationale**: Ensures configuration is loaded once at startup and efficiently accessible throughout the worker across threads.
- **Status**: Implemented.

## DEC-002: Service Fallback Pattern
- **Decision**: Implement a `VectorConfig` with a `fallback_strategy` list of `VectorProviderType`.
- **Rationale**: Realizes the "Resilient Configuration" vision where a missing service automatically triggers a less compute-intensive fallback (e.g., Elna -> RegexScan).
- **Status**: Implemented in structs; logic partially integrated in `VectorService`.

## DEC-003: Infrastructure Orchestration
- **Decision**: Initialize key services (Kip, Vector, Skills) in `main.rs` using the `ConfigService` to choose providers.
- **Rationale**: Centralizes environment-aware logic at the entry point.
- **Status**: Implemented.
