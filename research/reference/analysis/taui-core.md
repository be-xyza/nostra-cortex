# taui-core Analysis

## Placement
`research/reference/topics/agent-systems/taui-core`

## Intent
Analyze TAUI (Terminal Agent UI) standard and its core runtime as a comparative architecture against Nostra's A2UI (Abstract Agent UI). Specifically, evaluate its stateless document architecture and event normalization against our current AST-based mapping strategy to terminal elements.

## Possible Links To Nostra Platform and Cortex Runtime
- **A2UI Stateless Architecture**: TAUI replaces the entire document state on update, pushing state management purely to the agent loop. This strongly aligns with our ambition to decouple memory from the Projection plane.
- **Event Normalization**: Provides an established boundary for catching raw CSI/terminal sequences and dispatching clean "TAUI Actions".

## Initiative Links
- Relates strictly to `cortex-eudaemon` runtime projection strategies and potential future `cortex-cli` efforts.

## Pattern Extraction
- **Stateless Architecture**: `setDocument(spec)` completely obliterates and rewrites UI bounds without requiring component-level focus or internal React/TUI state.
- **Spec Gatekeeping**: Formal versioning (`1.0`) on the schema boundary enforces a strict contract before rendering.

## Adoption Decision
Intake as `recommendation_only`. Evaluated as an architectural foil and inspiration for A2UI's terminal-bound implementations. We will not natively adopt the TAUI schema directly into Cortex (as A2UI handles spatial properties beyond terminals), but the lifecycle semantics are highly valuable.

## Known Risks
- It is a highly specific terminal standard; semantic transplant to our multi-medium A2UI (which spans Heaper blocks, graph, and web) could create architectural drift.

## Suggested Next Experiments
Build a mock adapter inside `cortex/experiments/taui-comparative/` to pipe an A2UI payload schema into `taui-core`'s `TAUIRuntime` and evaluate its raw initialization and event loop relative to our previous `pi-tui` poc.
