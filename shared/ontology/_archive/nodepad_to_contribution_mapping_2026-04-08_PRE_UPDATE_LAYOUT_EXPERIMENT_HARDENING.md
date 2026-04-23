# Domain Ontology Mapping: Nodepad to Nostra Contribution

**Date**: 2026-04-09
**Status**: Experimental Mapping
**Reference Initiative**: `Nodepad Reference Intake`

## Objective
The current Nostra architectural baseline uses a unified, structural `Contribution` entity in knowledge blocks. However, to support heterogeneous spatial environments (like Heap Mode), we need robust sub-typing. `nodepad` relies on 14 distinct semantic content types to map aesthetic rules (colors, styles, interaction behavior) and to drive its background semantic synthesis.

This document proposes mapping those 14 structural archetypes onto our canonical `Contribution` object schema.

## Proposed `Contribution` Meta Extension

Under Nostra semantic architecture, rather than hardcoding tables for every type, we inject these types into the generic `Contribution` payload under a `metadata.semantic_type` property.

```json
{
  "$schema": "nostra-cortex-contribution-v2",
  "id": "urn:nostra:contribution:12345",
  "data": { ... },
  "metadata": {
    "semantic_type": "claim",
    "confidence_level": 0.8
  }
}
```

## The 14 Archetypal Types

Here is the canonical mapping from Nodepad semantics to A2UI rendering behavior.

| Nodepad Type    | Nostra Semantic Binding     | A2UI Accent Variable     | Visual / Rendering Behavior |
| ---------------- | ------------------------- | ------------------------ | --------------------------- |
| `entity`         | `Type::Entity`            | `var(--ui-accent-blue)`  | Standard card, high density |
| `claim`          | `Type::Claim`             | `var(--ui-accent-purple)`| Emphasized borders, confidence metrics |
| `question`       | `Type::Question`          | `var(--ui-accent-orange)`| Question mark glyph, high opacity background |
| `task`           | `Type::Actionable`        | `var(--ui-accent-green)` | Implicit checkbox rendering in generic list layout |
| `idea`           | `Type::Idea`              | `var(--ui-accent-yellow)`| Soft glow, organic layout clustering |
| `reference`      | `Type::ExternalLink`      | `var(--ui-accent-slate)` | Auto-fetches open-graph metadata, minimal presentation |
| `quote`          | `Type::Quotation`         | `var(--ui-accent-slate)` | Blockquote styling with attribution footers |
| `definition`     | `Type::Definition`        | `var(--ui-accent-cyan)`  | Sticky, high z-index in graph layout |
| `opinion`        | `Type::Subjective`        | `var(--ui-accent-pink)`  | Dotted borders or italic body copy |
| `reflection`     | `Type::Reflection`        | `var(--ui-accent-indigo)`| Muted text, ambient integration |
| `narrative`      | `Type::Document`          | `var(--ui-accent-amber)` | Prose-driven, wide block spacing |
| `comparison`     | `Type::Evaluation`        | `var(--ui-accent-teal)`  | Split column interior layout |
| `general`        | `Type::Null` (Default)    | `var(--ui-background)`   | Default plain-text behavior |
| `thesis`         | `Type::Synthesis`         | `var(--ui-accent-gold)`  | **System Generated** - highly visible, synthesized from the network |

## Agent Synthesis Rules

1. **Thesis Generation**: Only background agents (like Eudaemon) should generate `Type::Synthesis` blocks by looking at sub-graphs of `Type::Claim`, `Type::Idea`, and `Type::Reference`.
2. **Action Promulgation**: `Type::Actionable` elements detected inside `Type::Document` texts should be automatically extracted and linked via a `Dependency` relation.
3. **Graph Clustering**: the BSP spatial layout algorithm should group `Type::Question` blocks alongside their linked `Type::Claim` or `Type::Idea` answers.
