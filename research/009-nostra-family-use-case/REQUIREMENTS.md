---
id: 009
name: nostra-family-use-case
title: 'Requirements: Nostra Family Platform'
type: use-case
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-25'
---

# Requirements: Nostra Family Platform

## Functional Requirements

### FR-1: Privacy & Access Control
- **FR-1.1**: System must support "Private Sub-Spaces" or "Restricted Channels" visible only to a subset of members (e.g., "Medical").
- **FR-1.2**: System must support "Guest Roles" for temporary access (e.g., Doctor).

### FR-2: Asset Management
- **FR-2.1**: System must provide a "Gallery View" for `Artifacts` of type `Image`.
- **FR-2.2**: System must allow tagging `Artifacts` with `People` (for Family Tree).

### FR-3: Financial Coordination
- **FR-3.1**: System must support tracking "Financial Goals" (Fundraisers).
- **FR-3.2**: System must allow "Pledge" tracking on Initiatives.

### FR-4: Decision Making
- **FR-4.1**: System must support `Polls` for collective choices (Vacations).

## Non-Functional Requirements
- **NFR-1**: Mobile-First UI (families are on phones).
- **NFR-2**: Zero-knowledge encryption for "Sensitive" tagged content (Medical).

## Polymorphic Block Mapping (2026-02-25)

> Alignment with Initiative 124: Universal Polymorphic Block.

| Requirement | Polymorphic Block Payload Type | Notes |
|:---|:---|:---|
| FR-2.1 (Gallery View) | `media` | Filter blocks by `payload_type == media` for gallery rendering. |
| FR-2.2 (People Tagging) | `relations.tags` | Tagging uses graph edge projection on any block type. |
| FR-3.1 (Financial Goals) | `structured_data` | Pledge/goal tracking stored as structured JSON blocks. |
| FR-4.1 (Polls) | `a2ui` | Interactive polls rendered as A2UI widget blocks. |
