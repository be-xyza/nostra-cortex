---
id: '064'
name: remotion-video-engine
title: 'Decisions: Remotion Video Engine Integration (064)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: Remotion Video Engine Integration (064)

## DEC-001: Architecture Pattern
- **Decision**: Port Remotion's abstract patterns (Composition, Sequence, Interpolation) to pure Rust.
- **Rationale**: Avoids React/Node.js dependencies while maintaining Remotion's superior DX and mental model.
- **Status**: Implemented.

## DEC-002: Rendering Strategy
- **Decision**: Use a "No Bloat" FFmpeg CLI fallback for video stitching in Workers.
- **Rationale**: Direct C bindings (`ffmpeg-next`) proved difficult to port/deploy in all environments. CLI fallback ensures maximum portability and reliability.
- **Status**: Implemented.

## DEC-003: Frontend Integration
- **Decision**: Use Dioxus Canvas for preview rendering and WebCodecs (if available) for client-side export.
- **Rationale**: Seamlessly integrates with Nostra's existing Rust frontend stack.
- **Status**: Implemented (`MediaLab`).

## DEC-004: Distributed Processing
- **Decision**: Orchestrate frame rendering via Temporal Workflows.
- **Rationale**: Enables massive horizontal scaling for complex video generation on the IC.
- **Status**: Scaffolding integrated into `MediaService`.
