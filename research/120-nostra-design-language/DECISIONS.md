# Decisions

## 2026-04-27 - Space Design Profiles Use NDL Authority Wrapper

**Decision**: Adopt `SPACE_DESIGN.md` as a draft authoring profile for Space-level visual identity, paired with `NdlDesignProfileV1` as the Nostra-owned authority wrapper. The upstream `design.md` format is used only as an authoring and lint-pattern reference. NDL verified projection, A2UI theme policy, stewardship lineage, and surface boundaries remain the authority layer.

**Rationale**:

- Nostra Spaces need modular visual profiles without letting visual styling impersonate constitutional state.
- The upstream `design.md` pattern gives agents a compact token-plus-rationale packet, but it does not know Nostra authority boundaries.
- A wrapper keeps Space lineage, steward status, surface scope, and anti-spoofing rules next to the design tokens.

**Consequences**:

- Draft Space profiles remain `recommendation_only` until steward approval.
- Tier 1 constitutional components cannot be enabled by Space design tokens.
- Hermes may audit profiles through bounded source packets, but cannot approve, mutate, or enforce them.
- Runtime enforcement requires a later Nostra-owned linter and Cortex Web fixture validation.

## 2026-04-27 - Space Design Work Starts From Locked Design Reality

**Decision**: Treat current NDL, A2UI, branding, accessibility, ViewSpec, Cortex Web, and Space capability contracts as the locked design reality for Space-level design-standard analysis. Space design profiles, template packs, and imported design elements are recommendation-layer candidates until they pass local linting, fixture validation, and steward review.

**Rationale**:

- The system already has authoritative design and theme contracts; new Space-level work should compose with them rather than displace them.
- External design elements can be useful, but they must enter through analysis records that preserve provenance and strip unsupported authority claims.
- Hermes is useful for design-standard meta-cognition, but only as a bounded advisory reviewer.

**Consequences**:

- Add `DesignRealitySnapshotV1` and `DesignAuditUnitV1` as planning primitives before runtime adoption work.
- Keep `NdlDesignProfileV1` as a prototype name for `SpaceDesignProfileV1` until the primitive is promoted.
- Do not wire profile selection into Cortex Web runtime until current-reality checks, import analysis, and steward gates exist.
