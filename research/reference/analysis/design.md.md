# design.md Reference Analysis

## Placement

- **Classification**: repo reference
- **Placement**: `research/reference/repos/design.md`
- **Upstream**: `https://github.com/google-labs-code/design.md.git`
- **Reviewed commit**: `8ecd4645b957e6a683a05fb9c79cd6c9028873d0`
- **Review date**: 2026-04-27
- **Authority mode**: recommendation_only

Standalone placement is deliberate. The current checkout has a reference catalog that mentions `ui-substrate` topic paths, but those topic folders are not present. Recreating that taxonomy would be a structural reference move, so this intake keeps `design.md` as a cross-topic repo reference until a steward chooses a broader design-systems topic restoration.

## Intent

`design.md` defines a plain-text format for transmitting visual identity to coding agents. A file combines YAML front matter for machine-readable design tokens with Markdown prose for rationale. The upstream CLI can lint a `DESIGN.md`, diff two versions, emit structured findings, and export tokens to Tailwind or DTCG-style token JSON.

The core upstream primitives are:

- `DESIGN.md` document: one portable design-system source with tokens plus rationale.
- Token groups: `colors`, `typography`, `rounded`, `spacing`, and `components`.
- Token references: `{path.to.token}` references across the YAML tree.
- Canonical sections: `Overview`, `Colors`, `Typography`, `Layout`, `Elevation & Depth`, `Shapes`, `Components`, and `Do's and Don'ts`.
- Component sub-tokens: `backgroundColor`, `textColor`, `typography`, `rounded`, `padding`, `size`, `height`, and `width`.
- Diagnostics: broken references, missing primary color, WCAG component contrast warnings, orphaned tokens, missing spacing or rounding, missing typography, section order, and token summary.
- Export surfaces: Tailwind theme output and DTCG-like token output.

## Possible Links To Nostra Platform and Cortex Runtime

Nostra and Cortex already have stronger authority primitives than upstream `design.md`:

- Nostra platform primitives: `Space`, contribution lineage, governance and stewardship roles, access control, and NDL constitutional surface rules.
- Cortex runtime primitives: Workbench, A2UI render surfaces, verified projection, theme policy metadata, safe-mode allowlists, token-version fallback, motion and contrast preferences, and per-space capability/navigation plans.
- Initiative 120 primitives: NDL as a constitutional interface layer, Tier 1 verified components, surface boundary doctrine, and anti-spoofing.
- Initiative 130 primitives: global capability catalog plus per-space capability overlays that compile deterministic navigation and surfacing plans.
- Initiative 132 primitives: Hermes advisory audit units, source manifests, source packets, and recommendation-only synthesis artifacts.

`design.md` fills a narrower gap: it gives agents a compact, lintable, version-diffable design intent packet. That is useful for modular Space-level design profiles because Nostra Spaces need controlled visual differentiation without letting visual identity override constitutional truth. The right integration is a bridge: `DESIGN.md` can describe allowed visual personality for a Space, while NDL/A2UI remains the authority for surface type, verified projection, governance badges, provenance, roles, and anti-spoofing.

## Initiative Links

- `120-nostra-design-language`: most direct fit. Use `design.md` as a candidate authoring envelope for NDL visual profile packs, not as a replacement for NDL verified projection.
- `074-cortex-ui-substrate`: relevant to theme policy hardening, token-version negotiation, safe-mode fallback, motion policy, and contrast preference checks.
- `123-cortex-web-architecture`: relevant to `cortex-web` Workbench and A2UI rendering because Tailwind export can inform host theme compilation.
- `130-space-capability-graph-governance`: relevant if Space capability overlays later select or constrain design profiles for specific work modes.
- `132-eudaemon-alpha-initiative`: relevant for Hermes advisory audits. Hermes can consume bounded `DESIGN.md` source packets and emit design-standard findings, but must not mutate repo, runtime, or public UI authority directly.
- `088-accessibility-strategy`: relevant because upstream contrast diagnostics are useful but incomplete for Nostra accessibility gates.
- `045-component-library-labs`: relevant as historical component-library validation context now production-migrated into Initiative 074.

## Pattern Extraction

Useful patterns to adopt:

1. **Dual-layer authoring**: normative tokens plus explanatory prose. This fits NDL because agents need exact values and human rationale.
2. **Structured diagnostics for agents**: findings include severity, path, and message. This maps well to Hermes source-linked findings, SIQ-style evidence, and future Workbench review panels.
3. **Version diffing**: token-level diff can catch visual regressions before a Space design profile changes.
4. **Export adapters**: Tailwind and DTCG export provide a model for host-neutral token compilation.
5. **Lenient preservation**: unknown sections are preserved and unknown component properties warn rather than fail. This supports modular extension.

Required Nostra/Cortex extensions before adoption:

1. `surface_type` and NDL tier checks so execution surfaces cannot spoof constitutional components.
2. Provenance fields tying a profile to a Space, steward, version, hash, and approval state.
3. A capability binding that says where a profile may be used: Space shell, Workbench, artifact viewer, proposal preview, game/labs surface, or constitutional surface.
4. Motion policy, contrast preference, reduced-motion, focus visibility, and keyboard operability checks.
5. Anti-dark-pattern checks for governance actions and approval affordances.
6. A stricter component vocabulary for Nostra primitives such as decision surfaces, proposal cards, steward gates, contribution badges, provenance stamps, and agent output boundaries.

## Adoption Decision

Adopt as a **reference pattern**, not as a direct authority standard.

Recommended path:

1. Define `NDL_DESIGN_PROFILE.md` or `SPACE_DESIGN.md` as a Nostra-owned profile format inspired by upstream `DESIGN.md`.
2. Keep upstream primitives for tokens, prose sections, lint findings, diff, and export adapters.
3. Add Nostra-specific fields for `space_id`, `profile_id`, `profile_version`, `authority_mode`, `surface_scope`, `approved_by`, `lineage_ref`, `ndl_tier_policy`, and `a2ui_theme_policy`.
4. Use upstream lint as an optional first pass, followed by a Nostra-owned linter that enforces NDL, A2UI, accessibility, and stewardship constraints.
5. Allow Hermes to review design profiles only through source packets and advisory findings. Hermes should propose lint improvements or skill changes, not apply them.

The repo should not become the canonical Nostra design framework. It is best used as a small, portable design intent contract while NDL remains the constitutional interface layer.

## Known Risks

- Upstream is alpha-stage and may change section names, schema shape, or CLI behavior.
- The CLI uses TypeScript/Bun-oriented tooling, which is not a natural fit for Rust/WASM core enforcement.
- The upstream component schema is intentionally generic and does not know Nostra authority boundaries.
- Contrast linting is useful but too narrow to satisfy accessibility, motion, focus, density, or governance-legibility requirements.
- The token/prose format could create false confidence if a profile is treated as approved visual authority without steward lineage and hash verification.
- Font examples and design-prose freedom may conflict with existing frontend-design guidance unless Nostra adds local style constraints.

## Suggested Next Experiments

1. Create one `SPACE_DESIGN.md` prototype for a non-constitutional Space and run upstream `lint`, `diff`, and `export`.
2. Draft `NdlDesignProfileV1` as a JSON schema that wraps or imports the `DESIGN.md` token model while adding Nostra authority metadata.
3. Add a local linter spike with checks for `surface_scope`, Tier 1 component spoofing, unsafe governance affordances, missing provenance, reduced-motion, keyboard focus, and minimum contrast.
4. Add a Hermes source-packet template for design profile audits: inputs are the profile, NDL rules, A2UI theme policy, Space capability overlay, and target host.
5. Compare Tailwind export against `cortex-web` theme compilation to see whether profile tokens can generate deterministic host theme artifacts.
