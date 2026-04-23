---
id: 008
name: nostra-contribution-types
title: 'Requirements: Nostra Contribution Types'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Nostra Contribution Types

## Functional Requirements

### FR-1: New Contribution Types
- System must support `Decision` type with enhanced schema:
  ```rust
  Decision {
    id, spaceId,
    context: [ContributionId],      // Related contributions
    options: [Option],              // Alternatives considered
    selected: OptionId,             // Chosen option
    rationale: Text,                // Reasoning
    authority: AuthorityRef,        // Who decided
    reversibility: ReversibilityPolicy,  // Can it be undone?
    effective_at: Time,             // When it takes effect
  }
  ```
  **Security Constraint**: `Decision` and its ratification markers are Tier 1 NDL components. They MUST always be rendered within a `constitutional` surface. Agents and execution spaces are prohibited from spoofing this artifact class.
- System must support `Poll` type with properties: question, options, voting mechanism, expiry, scope.
- System must support `Bounty` type with properties: amount, currency, condition, completion oracle, resolution event.

### FR-2: Representation Layer
- Contributions must support multi-modal representations (text, video, audio, mixed).
- System must store `contentRef` pointers and optional `transcriptRef` for media content.
- Transcripts must be indexable and searchable.

### FR-3: Content Contribution Types
- System must support `Essay` type: long-form, sectioned, with abstract/thesis.
- System must support `Post` type: short-form, contextual, lightweight graph weight.
- System must support `MediaEssay` type: video/audio with transcript alignment.
- System must support `BlogEntry` type: serialized publication item.

### FR-4: Contribution Phase Metadata
- All contributions must support a `phase` metadata field.
- Valid phases: `Exploratory`, `Deliberative`, `Decisive`, `Executable`, `Archival`.
- Default phase assignment based on contribution type.

### FR-5: Evolution Paths
- Posts can evolve into Essays (preserving lineage).
- Essays can evolve into MediaEssays (adding representations).
- Reflections can be promoted to Artifacts or Decisions.

### FR-6: Deliberative Extensions
- System must support `Proposal` type: formal change request with `scope`, `impact`, `decision_required_by`.
- System must support `Review` type: formal evaluation with `targetId`, `verdict` (approve/reject), `score`, `reviewer`.

### FR-7: Execution Extensions
- System must support `Pledge` type: financial commitment with `target`, `amount`, `currency`, `contributor`.
- System must support `Service` type: monetized workflow with `pricing_model`, `access_tier`, `workflowId`.
- System must support `Event` type: time-bound coordination with `startTime`, `endTime`, `location`, `participants`.

### FR-8: Entity Layer
- System must support non-contribution `Entity` nodes:
  - `Person`: Identity wrapper for deceased/external figures.
  - `Organization`: Profile for external bodies.
  - `Library`: Data source wrapper (e.g., repository URL, metadata).
  - `Module`: A domain boundary within a library, e.g., `react-dom`
  - `Component`: An isolated executable, class, or renderable unit.
  - `Function`: A specific algorithmic export.
  - `Concept`: A canonical algorithm standard, e.g., `CRDT` or `OAuth`.

### FR-9: Metadata Extensions
- All contributions must support `confidence` field (Float 0.0-1.0) for plausibility scoring.

## References
- [PLAN.md](./PLAN.md)
- [DECISIONS.md](./DECISIONS.md)
- [002-nostra-v2-architecture/REQUIREMENTS.md](../002-nostra-v2-architecture/REQUIREMENTS.md)
