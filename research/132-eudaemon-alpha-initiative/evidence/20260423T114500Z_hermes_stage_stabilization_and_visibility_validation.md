# Hermes Stage Stabilization and Visibility Validation - 2026-04-23

## Context

This evidence records the next Initiative 132 Hermes planning step after bounded activation, capability discovery, and runbook validation.

The goal of this pass was to:

- make Hermes stage-stabilization signals explicit
- define a deterministic source-packet layer for bounded passes
- validate how Hermes should surface through Cortex Web visibility and approval primitives
- check whether the source-packet builder fits the current Nostra-native contribution model

## Result

Verdict: PASS with stage status `yellow`.

Hermes remains ready for bounded local advisory use, but not yet ready to graduate beyond the current stage envelope.

## What Was Added

Local Hermes planning/runtime artifacts:

- `HermesSourcePacketV1`
- `HermesStageStabilizationV1`
- `source-packets/initiative-132-hermes-stage-readiness.source-packet.v1.json`
- `stabilization/initiative-132-hermes-stage-stabilization.v1.json`
- `design-notes/hermes-cortex-web-visibility-routing.md`

Initiative 132 governance was updated to record:

- source packets as the preferred bounded excerpt/fact layer when direct file reading is unnecessary
- explicit stage update/upgrade/deprecate signals
- Cortex Web heap/proposal/approval surfaces as the correct future projection path for Hermes outputs

## Validated Findings

### 1. Source-packet builder is the right ingestion fix for this stage

The earlier runbook validation showed that Hermes can drift into shell/code-style inspection if a pass implicitly asks it to gather facts from files. The new source-packet contract keeps ingestion deterministic without expanding Hermes authority.

Source packets remain local inputs only. They do not outrank root `ICP`, source manifests, or validation results.

### 2. Hermes should surface through existing Cortex Web primitives

The repo already contains the visibility and approval surfaces Hermes needs:

- heap block emission and context packaging
- approval-first heap solicitations
- steward feedback lineage
- steward-gate validation/apply
- A2UI approval telemetry
- viewspec/workflow proposal review routes
- agent-contribution approval routes

The best first path is Heap-first projection of Hermes advisory artifacts. Live run approval routes are a second-stage option only if Hermes later becomes part of a governed live-run workflow.

### 3. Source packets fit Nostra-native contribution plans, but not as a new primitive now

The closest long-term Nostra-native fit is:

- a Proposal-backed bounded bundle for deliberative review, or
- a durable Artifact/Report when preserving evidence or reference value

Chronicle or GlobalEvent is the right place to record packet creation, review, and promotion history, not the primary storage shape for the packet itself.

## Current Stage Signal

Current stage status remains `yellow` because:

- bounded pass reliability still needs a longer clean streak
- the new source-packet path is defined but not yet proven across repeated routine passes
- Hermes is not yet projected into Cortex Web as a first-class user-visible flow

The stage remains `green` on:

- guardrail integrity
- no mutation / no provider execution
- net architectural value

## Validation

- local Hermes JSON syntax checks: PASS
- local Hermes contract checks: PASS
- research portfolio consistency: PASS
- dynamic config contract: PASS
- gateway parity inventory sync: PASS
- skill policy: PASS
- terminology check on edited Initiative 132 docs: PASS

## Conclusion

Hermes is now better defined as a staged local observer:

- bounded input via manifests, audit units, and source packets
- bounded output via session + synthesis artifacts
- future user visibility through existing heap/proposal/approval primitives
- explicit lifecycle signals for when to update, upgrade, or deprecate the Hermes lane
