---
id: "144-meta-workbench-refactoring"
name: "meta-workbench-refactoring"
title: "Meta Workbench Refactoring Plan"
type: "plan"
project: "cortex"
status: draft
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["ui", "spaces", "data-aggregation"]
reference_assets: []
evidence_strength: "hypothesis"
handoff_target: ["cortex-web", "cortex-eudaemon"]
authors:
  - "Antigravity Agent"
tags: ["workbench", "space", "meta", "ui", "navigation"]
stewardship:
  layer: "cortex"
  primary_steward: "system"
  domain: "frontend-backend-integration"
created: "2026-03-13"
updated: "2026-03-13"
---

# Meta Workbench Refactoring Plan

## Overview
This initiative defines the "Meta Workbench"—a user-scoped, cross-space operational layer within Cortex. It aims to detach the Cortex execution primitives (Workbench, Heap, Action Zones) from an enforced 1:1 `space_id` relationship. Instead, it enables a unified, global view of user actions, blocks, and notifications across *all* domains a user belongs to.

By defining the semantic boundary of a "Meta Workbench", we clarify how the frontend `cortex-web` aggregator interacts with the `cortex-eudaemon` gateway when a specific `space_id` is intentionally omitted or flagged as "meta".

---

## User Review Required

> [!IMPORTANT]
> **Default Fallback Behavior:**
> When the user opens the application, should the default route trigger the Meta Workbench (unified view), or attempt to remember and reload the last active Space?

---

## Proposed Architecture

### Meta Workbench Aggregator (Frontend - `cortex-web`)
- **`activeSpaceId` optionality**: UI stores and fetching hooks must support a `null` or explicit `"meta"` value.
- **Space Selector Expansion**: The Layout shell's space selector will include a dedicated entry for "Global / All Spaces".

### Cortex Gateway API (`cortex-eudaemon`)
- **Block Fetching (`get_cortex_heap_blocks`)**: Naturally supports cross-space loading when the `space_id` query constraint is omitted.
- **Navigation Plan (`get_space_navigation_plan`)**: Will intercept queries where `space_id == "meta"`, short-circuiting capability logic to return a predefined, globally scoped routing plan (e.g. Inbox, Unified Heap, Settings).

---

## Implementation Phases

### Phase 1: Semantic Grounding & API Adjustments
- [x] Analyze codebase boundaries.
- [ ] Implement backend `space_id == "meta"` routing logic interception in `server.rs`.
- [ ] Validate block projection payloads retain proper `workspace_id` identifiers for provenance traceability.

### Phase 2: React Context & Store Refactoring
- [ ] Refactor `uiStore.ts` to support global contexts.
- [ ] Introduce a "Meta Workbench" option into `ShellLayout.tsx` and the unified global space selector.
- [ ] Update data fetchers (e.g., `useHeapActionPlan.ts`) to selectively drop `space_id` from query parameters when in meta mode.

### Phase 3: UI Enhancement
- [ ] Add visual differentiators (e.g. badge colors or prefix headers) in the Heap Canvas to distinguish blocks originating from different workspaces in the unified view.

---

## Verification Plan

### Automated Tests
```bash
# Verify backend routing logic
cargo test --package cortex-domain
```

### Manual Verification
1. Boot the Eudaemon and launch `cortex-web` (`npm run dev`).
2. Navigate to a specific space (e.g. `nostra-governance-v0`) and verify isolated block rendering.
3. Switch Context to the "Meta Workbench".
4. Ensure blocks from *multiple* active spaces appear in the same view without causing authorization errors.
5. Verify the navigation sidebar reflects global routes, not space-specific capabilities.

---

## File Structure (Proposed)
```
cortex/apps/cortex-web/src/
├── components/commons/ShellLayout.tsx
├── store/uiStore.ts
└── hooks/useHeapActionPlan.ts
cortex/apps/cortex-eudaemon/src/
└── gateway/server.rs
```
