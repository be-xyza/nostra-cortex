---
version: alpha
name: Research Observatory
description: A dense but calm research Space profile for synthesis, review, source comparison, and evidence inspection.
colors:
  primary: "#102A2D"
  secondary: "#466466"
  tertiary: "#B84A3A"
  neutral: "#F4F1EA"
  surface: "#FFFDF7"
  on-surface: "#102A2D"
  evidence: "#2D6A78"
  warning: "#8A5A16"
  boundary: "#6D5F53"
typography:
  headline-display:
    fontFamily: Fraunces
    fontSize: 40px
    fontWeight: 650
    lineHeight: 1.05
    letterSpacing: 0em
  headline-md:
    fontFamily: Fraunces
    fontSize: 24px
    fontWeight: 600
    lineHeight: 1.15
    letterSpacing: 0em
  body-md:
    fontFamily: Source Serif 4
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.55
    letterSpacing: 0em
  label-md:
    fontFamily: IBM Plex Mono
    fontSize: 12px
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: 0.04em
rounded:
  none: 0px
  sm: 4px
  md: 8px
  lg: 12px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 40px
  gutter: 20px
  measure: 72
components:
  space-shell:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    padding: "{spacing.lg}"
  artifact-card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.md}"
    padding: "{spacing.md}"
  evidence-label:
    backgroundColor: "{colors.evidence}"
    textColor: "{colors.surface}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    padding: "{spacing.sm}"
  boundary-frame:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.boundary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    padding: "{spacing.sm}"
---

# SPACE_DESIGN.md: Research Observatory

## Overview

Research Observatory is a Space profile for careful synthesis work. It should feel like a quiet archive table with enough density for repeated review, source comparison, and evidence inspection. It is not a constitutional surface and must never imply final approval through visual styling alone.

## Colors

The palette uses mineral green and paper neutrals for low-fatigue reading, with clay red reserved for review attention. Evidence blue marks source-linked material. Boundary brown marks containment, uncertainty, and advisory status.

- **Primary (#102A2D):** Deep mineral green for primary text and durable headings.
- **Secondary (#466466):** Muted green-gray for captions, table rules, and inactive metadata.
- **Tertiary (#B84A3A):** Clay red for review attention, unresolved contradictions, and destructive-action warnings.
- **Neutral (#F4F1EA):** Warm paper field for Space shell backgrounds.
- **Surface (#FFFDF7):** Clean reading surface for artifact cards and source panels.
- **Evidence (#2D6A78):** Source-linked labels, citations, and verified evidence markers.
- **Boundary (#6D5F53):** Advisory frames, provenance rails, and execution containment labels.

## Typography

Display text uses a serif with editorial weight. Body copy remains readable for long notes. Metadata uses a monospace family so timestamps, hashes, paths, and source labels scan differently from prose.

## Layout

The layout favors compact panels, clear gutters, and readable measure. It should support side-by-side source comparison without decorative cards around whole page sections.

## Elevation & Depth

Depth should come from tonal layering, border clarity, and source grouping. Heavy shadows are avoided because they blur authority boundaries.

## Shapes

Corners are restrained. Repeated cards may use an 8px radius, while shell and constitutional-adjacent boundaries should stay square or nearly square.

## Components

This draft profile covers Space shell, artifact cards, evidence labels, and boundary frames only. Governance badges, identity verification, final approval states, and Tier 1 constitutional components must be rendered through verified projection.

## Do's and Don'ts

- Do use source labels, calm contrast, compact grouping, and clear boundary frames.
- Do show advisory and execution status with containment frames.
- Do not render approved, ratified, constitutional, verified identity, or steward-authorized claims from this profile.
- Do not use profile colors to imply graph truth, approval, or identity verification.
