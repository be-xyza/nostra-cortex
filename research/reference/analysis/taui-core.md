# taui-core Analysis

## Placement
`research/reference/topics/agent-systems/taui-core`

## Intent
Analyze TAUI (Terminal Agent UI) standard and its core runtime as a comparative architecture against Nostra's A2UI (Abstract Agent UI). Specifically, evaluate its stateless document architecture, schema discipline, and runtime event handling against our current terminal-host mapping strategy.

## Possible Links To Nostra Platform and Cortex Runtime
- **A2UI Terminal Document Discipline**: TAUI replaces the entire document state on update, pushing state management purely to the agent loop. This aligns with our goal of keeping host projections thin and authority outside the renderer.
- **Strict Schema Gatekeeping**: The installed validator rejects unsupported nodes and versions up front, which is directly relevant to the terminal-document boundary we are adding around A2UI terminal-safe payloads.

## Initiative Links
- Relates to `118-cortex-runtime-extraction`, `124-polymorphic-heap-mode`, and operator terminal-host promotion work such as a future `cortex-tui` or `cortex-operator-cli`.

## Pattern Extraction
- **Stateless Architecture**: `setDocument(spec)` completely obliterates and rewrites UI bounds without requiring component-level focus or internal React/TUI state.
- **Spec Gatekeeping**: Formal versioning (`1.0`) on the schema boundary enforces a strict contract before rendering.

## Adoption Decision
Intake as `recommendation_only`. Evaluated as an architectural foil and inspiration for A2UI's terminal-bound implementations. We should borrow the schema-discipline idea, but not adopt the TAUI runtime directly into Cortex at this phase. The installed runtime here proved strict document validation, but it did not prove production-ready raw terminal event normalization.

## Known Risks
- It is a highly specific terminal standard; semantic transplant to our multi-medium A2UI (which spans heap blocks, graph, and web) could create architectural drift.
- The installed `@taui-standard/core` package currently faults when fed raw terminal byte sequences directly, so runtime maturity is weaker than the schema layer suggests.

## Suggested Next Experiments
Build a narrow translator inside `cortex/experiments/taui-comparative/` that maps one real terminal-safe A2UI payload into a TAUI document, and keep the evaluation focused on schema discipline and document lifecycle rather than assuming raw-event normalization is already solved upstream.
