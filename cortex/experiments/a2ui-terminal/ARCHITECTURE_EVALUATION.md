# A2UI Terminal Evaluation

## Validated Findings

The `a2ui-terminal` experiment aligns strongly with the validated `pi-mono` pattern of multiple frontends over one shared agent/runtime substrate. Local validation for this pass:

- the prototype installs and starts successfully in terminal
- keyboard selection works end to end
- the surface is structurally much smaller than `cortex-web`
- the repo already contains terminal-host infrastructure in `/Users/xaoj/ICP/nostra/apps/cortex-desktop/src/services/terminal_service.rs`

This supports promoting the experiment as a Cortex host adapter candidate, not as a replacement for `cortex-web`.

## Pattern Validation

### Confirmed `pi-mono` carryover
- shared runtime substrate with multiple hosts
- host-specific rendering over a common agent/runtime core
- operator-side terminal rendering as a first-class adapter, not a fallback hack

### Additional local patterns worth adopting now
- payload-envelope normalization from `/Users/xaoj/ICP/cortex/apps/cortex-web/src/store/eventProcessor.ts`
- payload-type routing from `/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/heap/PayloadRenderer.tsx`
- artifact/workflow handoff routing from `/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/heap/heapArtifactRouting.ts` and `/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/workflows/artifactRouting.ts`
- host-side terminal process management from `/Users/xaoj/ICP/nostra/apps/cortex-desktop/src/services/terminal_service.rs`

### Patterns to keep recommendation-only
- `in-place-ttt` host-side adaptive inference is unrelated to A2UI surface rendering for this phase
- rich browser-specific widgets such as spatial canvases, workflow inspectors, capability maps, and schema editors should remain `cortex-web` surfaces

## Surface Matrix

### Terminal-first now
- small A2UI approval flows using `Container`, `Box`, `Text`, `Spacer`, `SelectList`
- rich text and task summaries
- pointer previews
- compact structured-data summaries

### Terminal summary plus web handoff
- structured data requiring drill-down
- charts and telemetry summaries
- media artifacts
- workflow-linked artifacts

### Web-required
- spatial/tldraw surfaces
- workflow artifact inspector routes
- capability maps and capability matrices
- schema editing
- contribution graph exploration
- evaluator DAG views
- any A2UI tree with unsupported widget families

## Recommendation

Promote this experiment only as an operator-side Cortex host adapter with explicit handoff to `cortex-web` when fidelity, navigation, or richer interaction is required.

That keeps the architecture aligned with:

- Cortex host neutrality
- parity over duplication
- operator-side execution infrastructure boundaries
- `cortex-web` as the richer workbench host for spatial, artifact, and workflow inspection

## Next Promotion Slice

The next good implementation slice is not more widget breadth. It is runtime integration:

1. feed a real heap or workflow payload into the planner
2. run the terminal adapter through the existing Cortex terminal service
3. emit explicit handoff URLs for `cortex-web`
4. capture startup/render timings against one equivalent `cortex-web` artifact flow

## TAUI Adjustment

TAUI remains a reference input for strict stateless document validation, not a runtime dependency decision.

- adopt now: strict terminal document gatekeeping
- do not adopt now: TAUI runtime/event-normalization package as production host infrastructure

## Current Runtime-Backed Proof

- desktop-first ACP integration remains the first promotion slice
- the existing ACP terminal endpoints remain unchanged
- the current live end-to-end proof was executed through an ACP-compatible `cortex-eudaemon` gateway
- `cortex-desktop` already contains the relevant terminal service and ACP policy surfaces, but this branch does not yet expose it as a runnable gateway binary
- the adapter must prove:
  - terminal-safe note/task and structured-data flows through ACP
  - workflow-heavy surfaces emit a `cortex-web` handoff route
