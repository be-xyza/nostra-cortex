---
id: '011'
name: tech-stack-video-streaming
title: Feedback & Open Questions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback & Open Questions

## Open Questions

### Q1: Arweave Offloading
- **Question**: Should we support storing encrypted blobs on Arweave for cost savings?
- **Trade-off**: Breaks "Pure ICP" sovereignty but 100x cheaper for massive files.
- **Status**: Deferred.

### Q2: Wasm Spoofing
- **Question**: Can the "Smart Witness" be spoofed?
- **Mitigation**: Use `vetKeys` to enforce a cryptographic "Chain of Viewing" where Segment N+1 key requires Segment N proof.
- **Status**: Future Scope.
