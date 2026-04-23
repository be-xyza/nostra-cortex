# Resolution: Skills Sync Service Cross-Initiative Analysis

**Date**: 2026-01-18
**Initiative**: 016-nostra-skills-sync-service-use-case
**Studies Completed**: 16.1 (Platform Gaps), 16.2 (Claude-Brain Tech), 16.3 (Agent Memory Architecture)

---

## Executive Summary

The Skills Sync Service (016) has evolved from a simple SKILLS.MD synchronization use case to a comprehensive **Agent Intelligence Service** encompassing:
1. **Skills Sync** — Curated skill bundles from Nostra Space
2. **Agent Memory** — Observations stored in Personal Libraries with log references

This resolution maps 016 to all dependent initiatives and defines the optimal implementation path.

---

## 1. Dependency Graph

```
┌──────────────────────────────────────────────────────────────────┐
│                    016: SKILLS SYNC SERVICE                       │
│             (Hybrid Sandbox: Nostra Commons + Cortex)             │
└──────────────────────────────────────────────────────────────────┘
         │
         ├──────────────────────────────────────────────────────────┐
         │  REQUIRES                                                │
         ▼                                                          ▼
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│ 126: Agent Harness  │    │ 122: Cortex Runtime │    │ 121: Cortex Memory  │
│ L1/L2 Proposals     │    │ Temporal Skeletons  │    │ Git-Backed Local FS │
│ Replay Protocols    │    │ Semantic Merging    │    │ Trajectory Logs     │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
         │                          │
         ▼                          ▼
┌─────────────────────┐    ┌─────────────────────┐
│ 013: Workflow Engine│    │ 018: Library System │
│ Execution Scopes    │    │ Canonical Commons   │
└─────────────────────┘    └─────────────────────┘
```

---

## 2. Initiative Status Summary

| Initiative | Status | Blocking 016? | Notes |
|:-----------|:-------|:--------------|:------|
| **126 - Agent Harness** | Active | Yes | Provides the AuthorityGuard for Bounties |
| **122 - Cortex Runtime** | Active | Yes | Orchestrates the semantic merge deterministically |
| **121 - Cortex Memory** | Active | Yes | Provides the local agent storage replacing backend injection |
| **018 - Library System** | Active | Partial | Hosts the Canonical `SKILLS.MD` in the Commons |
| **013 - Workflow Engine** | Active | Yes | Defines the subscription paths |

---

## 3. Recommended Implementation Path

### Path A: Cortex-First (Recommended) ✅

Focus on establishing the local execution execution boundaries before hooking them into Nostra governance.

```
PHASE   INITIATIVE   DELIVERABLE
───────────────────────────────────────────────────────────────
  1     121          Cortex Memory FS foundational operations
        122          Agent Runtime Temporal bindings
  2     126          Authority Guard and AgentExecutionRecord testing
  3     016          Workspace Orchestration (registry.local.toml overlay)
  4     016          Draft Skills Sync workflow definition on Nostra side
───────────────────────────────────────────────────────────────
```

---

## 4. Specific Next Steps

*   **Cancel**: All immediate actions attempting to inject `ObservationEntity` schemas and `logRefs` deeply into the `018` Node backend environments. These assumptions are superseded by `121-cortex-memory-fs`.
*   **Execute**: Establish the `sync_skills_registry.py` overlay support for `registry.local.toml` to safely build skills without bypassing governance.
*   **Execute**: Scaffold the `nostra/commons/workflows/` canonical registry to bring Agent Workflows into parity with Agent Skills.

---

## 5. Resolved Design Decisions

| Decision | Resolution | Source |
|:---------|:-----------|:-------|
| Where to store agent memories? | Local `cortex-memory-fs` (not Nostra Graph) | 121 |
| How to handle semantic merging? | Delegated to `cortex-agent-runtime-kernel` temporal tasks | 122 |
| How to prevent Sybil Bounties? | Authority Guard evaluating Replay Artifacts via Proposals | 126 |
| How to test safely? | Deep-merge `registry.local.toml` using `sync_skills_registry.py` | Workspace |

---

## 6. Risks & Mitigations

| Risk | Impact | Mitigation |
|:-----|:-------|:-----------|
| 121/122 delays | Blocks semantic merging | Rely on deterministic git merges in Phase 1 |
| 126 delays | Blocks bounty payouts | Rely on manual Steward PR reviews for Phase 1 |

---

## 7. Success Metrics

| Metric | Target | How to Measure |
|:-------|:-------|:---------------|
| `registry.local.toml` separation | 0 accidental canonical PRs | PR inspection |
| Cortex Memory separation | 0 telemetry blobs in Nostra space | Graph inspection |

---

## 8. Updated 016 Task List

Based on this resolution, the revised task.md for 016 should be:

```markdown
# Task: Nostra Skills Sync Service (Hybrid Pivot)

## Studies
- [x] Study 16.1: Platform Gaps (Shifted to 122/126)
- [x] Study 16.3: Agent Memory Architecture (Superseded by 121)
- [x] Cross-Initiative Resolution

## Implementation
### Phase 1: Workspace Orchestration
- [ ] Support `registry.local.toml` in `sync_skills_registry.py`
- [ ] Scaffold `nostra/commons/workflows/registry.toml`
- [ ] Implement `sync_workflows_registry.py`

### Phase 2: Execution Sandboxing (Cortex)
- [ ] Integrate 121 Memory FS mapping
- [ ] Delegate Semantic Merge to 122 Runtime

### Phase 3: Canonical Sybil Bounties (Nostra)
- [ ] Implement 126 Authority Guard proposals
- [ ] Define Governance parameters for Bounty execution
```
