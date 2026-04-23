# Implementation Plan - Cortex UI Substrate Stabilization + Default Theme Formalization

## Goal
Complete full Cortex Desktop UI stabilization for initiative 074 while formalizing the existing `cortex` theme as the canonical default (no replacement theme).

## Scope
- Desktop-first full sweep of Cortex Desktop components.
- Formalize shared theme tokens for canonical default usage.
- Remove structural hardcoded styling (`bg-[#...]`, `dark:`, `border-white/*`) from desktop components.
- Enforce ASCII-first production iconography.
- Add blocking conformance gate integrated with closeout and test catalog.

## Canonical Paths
- Desktop components: `cortex/apps/cortex-desktop/src/components/**`
- Desktop theme types: `cortex/apps/cortex-desktop/src/components/a2ui/theme.rs`
- Desktop runtime theme injection: `cortex/apps/cortex-desktop/src/services/theme.rs`
- Offline utility fallback: `cortex/apps/cortex-desktop/assets/tailwind_fallback.css`
- Shared theme tokens: `shared/a2ui/themes/cortex.json`, `shared/a2ui/themes/nostra.json`
- Closeout + test-gate scripts: `scripts/cortex-desktop-closeout-check.sh`, `scripts/generate_test_catalog.sh`

## Phase Contract
1. Governance + lineage updates (`DECISIONS.md`, archived copies).
2. Default theme formalization (`text_on_accent` and symmetric token compatibility).
3. Full desktop token adoption sweep.
4. ASCII-first iconography enforcement.
5. Blocking conformance script (`check_cortex_ui_theme_conformance.sh`) + closeout integration.
6. Verification and gate refresh in blocking mode.

## Key Changes
1. Shared token contract (additive):
   - `text_on_accent` in `cortex.json` and `nostra.json`.
2. Desktop token plumbing:
   - `ThemeColors` includes `text_on_accent`.
   - Runtime CSS includes `--text-on-accent`.
   - Fallback utilities include `text-[var(--text-on-accent)]`.
3. Component conformance:
   - Structural surfaces use `--bg-*`, `--text-*`, `--border-*` tokens.
   - Accent controls use `text-[var(--text-on-accent)]`.
4. Enforcement:
   - Release-blocker check fails on banned classes and non-ASCII usage.

## Verification Plan
### Automated
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml --tests`
- `bash scripts/check_cortex_ui_theme_conformance.sh`
- `bash scripts/cortex-desktop-closeout-check.sh`
- `bash scripts/check_test_catalog_consistency.sh --mode blocking`

### Manual
- Validate theme switch (`cortex` and `nostra`) for readability and accent controls.
- Validate console keyboard behavior (`Enter`) and visible focus.
- Validate conformance script fails on intentional temporary regression.

## Acceptance Criteria
1. Canonical default `cortex` theme is explicitly formalized and consumed end-to-end.
2. Zero banned-pattern and non-ASCII violations in desktop components.
3. Closeout and test-catalog consistency pass in blocking mode.
4. No regressions to existing A2UI policy behavior (safe mode, token-version compatibility, motion/contrast metadata handling).

## Lineage
This plan remains under `research/074-cortex-ui-substrate/*` and supersedes earlier `074-themes-system` path references.
