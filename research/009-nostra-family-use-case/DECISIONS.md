---
id: 009
name: nostra-family-use-case
title: 'Decision Log: Nostra Family Platform'
type: use-case
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decision Log: Nostra Family Platform

## DEC-001: Family as a Space
**Date**: 2026-01-15
**Status**: ✅ Proposed

**Decision**: Map the "Family Unit" to a Nostra `Space`.
**Rationale**: Spaces provide the strongest boundary for identity, privacy, and shared context.

## DEC-002: Vacation Planning via Polls
**Date**: 2026-01-15
**Status**: ✅ Proposed

**Decision**: Use `Poll` contribution type for selecting vacation destinations.
**Rationale**: Simple consensus mechanism fits the user story better than complex voting governance.

## DEC-003: Fundraising via Initiatives
**Date**: 2026-01-15
**Status**: 🟡 Proposed

**Decision**: Model "Fundraisers" as `Initiatives` with metadata, rather than a new type.
**Rationale**: Avoid type explosion. Detailed requirements for "payments" are not yet clear, so "Initiative" (Strategic Effort) is the safest semantic fit.
