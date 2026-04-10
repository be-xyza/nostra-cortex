---
id: conversation-surface-runtime-authority
title: Conversation Surface Runtime Authority
owner: Systems Steward
updated_at: 2026-04-09
---

# Conversation Surface Runtime Authority

## Purpose
Preserve runtime ownership of chat context resolution, persistence, projection, and provider-runtime-first dispatch.

## Triggers
- Changes to `/ws/chat`
- Changes to `/api/cortex/chat/conversations*`
- Changes to heap-backed conversation persistence or mixed-content rendering
- Changes to provider-runtime dispatch or fallback behavior

## Inputs
- The conversation-surface decision record and relevant runtime/web host changes
- Affected chat projection or heap-block contracts

## Lanes
- `dispatch-change`: provider-runtime or fallback generation behavior changed.
- `projection-change`: chat projection, mixed-content rendering, or heap-backed persistence changed.
- `ownership-risk`: client-local state threatens runtime authority over conversation history.

## Analysis Focus
- Runtime ownership of context resolution and persisted history.
- Provider-runtime-first dispatch versus compatibility fallback behavior.
- Host rendering changes versus source-of-truth changes.

## Steps
1. Confirm chat context resolution still happens server-side before generation.
2. Confirm provider-runtime Responses remains primary and compatibility fallback remains explicit.
3. Verify conversation history ownership remains runtime-backed rather than client-owned.
4. Capture any rendering or projection changes needed in `cortex-web`.

## Outputs
- Statement of runtime authority preservation for conversation history and dispatch
- Follow-up validation tasks for affected hosts

## Observability
- Record dispatch path selection and persistence ownership outcomes.
- Capture whether the change touched history projection, transport, or rendering only.
- Note recurring client-local ownership drift as a structural regression.

## Self-Improvement
- If the same ownership regressions recur, add more targeted conversation-surface checks.
- If host rendering changes frequently without protocol change, split render-only follow-up into a narrower workflow.

## Required Checks
- `bash scripts/check_cortex_dual_host_parity.sh`
