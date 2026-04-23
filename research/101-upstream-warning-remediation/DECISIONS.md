---
id: "101-upstream-warning-remediation-decisions"
name: "upstream-warning-remediation-decisions"
title: "Decision Log: Upstream Warning Remediation"
type: "decision"
project: "nostra"
status: active
authors:
  - "User"
  - "Codex"
tags: [dependencies, security, remediation]
created: "2026-02-05"
updated: "2026-02-05"
---

# Decision Log: Upstream Warning Remediation

Track architectural decisions with rationale for future reference.

---

## DEC-001: Start with Upstream Tracking Before Upgrades
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Upgrade upstream crates immediately.
2. Track upstream releases and stage upgrades safely.

**Decision**: Track upstream releases and stage upgrades behind `dfx build` + `cargo test` before merging.

**Rationale**: Keeps changes low‑risk while still progressing toward warning reduction.

**Implications**: Warning reduction is paced by upstream releases; we maintain a clean audit trail.

---

## DEC-002: Confirm Latest Upstream Versions (No Immediate Major Bumps)
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Upgrade immediately to latest `pocket-ic` major.
2. Confirm upstream latest versions, keep current major lines, and stage major upgrades.

**Decision**: Confirm upstream latest versions (`ic-agent` 0.45.0, `candid` 0.10.21, `ic-transport-types` 0.45.0, `pocket-ic` 12.0.0) and defer major `pocket-ic` upgrades until test harness validation is completed.

**Rationale**: `pocket-ic` jumped multiple majors, which can introduce test harness changes. The current Nostra lockfile already matches latest `ic-agent`/`candid` lines, so the safest next step is to stage major upgrades behind explicit validation.

**Implications**: Next work focuses on a scoped upgrade branch for `pocket-ic` (and downstream `ic-transport-types`) rather than production changes.

---

## DEC-003: Track Unmaintained Advisories Without Local Overrides
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Patch/override transitive crates locally (`backoff`, `instant`, `paste`, `serde_cbor`).
2. Track advisories and wait for upstream replacements.

**Decision**: Track advisories and avoid local overrides unless upstream provides compatible replacements or a security incident demands action.

**Rationale**: These crates are transitive dependencies (primarily via `ic-agent`, `candid`, `pocket-ic`). Local overrides risk incompatibilities and unstable builds.

**Implications**: Document advisories as accepted risk, revisit after upstream releases or when moving to new major SDK lines.

---

## DEC-004: Adopt PocketIC 12.0.0 in Dev/Test Dependencies
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Stay on `pocket-ic` 6.x and accept older test harness.
2. Upgrade to `pocket-ic` 12.0.0 and adjust test-kit call handling.

**Decision**: Upgrade `pocket-ic` to 12.0.0 for dev/test dependencies and update `nostra-test-kit` to the new `update_call` return type.

**Rationale**: Upgrade succeeded with `cargo check`, `cargo test -p nostra-test-kit`, and `dfx build` while keeping production canister builds intact.

**Implications**: Unmaintained warnings (`backoff`, `instant`, `paste`, `serde_cbor`) persist because they remain transitive; track as accepted risk until upstream replacements arrive.
