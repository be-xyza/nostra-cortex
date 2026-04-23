---
id: '064'
name: remotion-video-engine
title: 'Feedback: Remotion Video Engine Integration (064)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback: Remotion Video Engine Integration (064)

## 2026-01-24: Portability vs Performance
- **Source**: AI Agent (Logic Audit)
- **Question/Concern**: `ffmpeg-next` C bindings are causing deployment issues in some CI environments.
- **Resolution**: Switched to a "No Bloat" FFmpeg CLI fallback in `MediaService`.
- **Decision**: → DEC-002

## 2026-01-23: React Adoption
- **Source**: User
- **Question/Concern**: Is this a proposal to adopt React?
- **Resolution**: Explicitly clarified that Nostra remains Dioxus/Rust based; only patterns are ported.
- **Decision**: → DEC-001
