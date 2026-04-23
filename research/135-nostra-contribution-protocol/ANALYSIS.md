# NCP Integration Analysis

## Overview
This document maps the proposed **Nostra Contribution Protocol (NCP)** patterns against the existing **Nostra Cortex** codebase and architectural primitives. It identifies where the patterns are already grounded and where friction remains.

## Pattern Mapping

| NCP Pattern | Nostra/Cortex implementation | Validation Status |
|-------------|-----------------------------|-------------------|
| **Contribution Proposal (CP)** | `EntityType.Proposal` in [kip.mo](file:///Users/xaoj/ICP/nostra/backend/modules/kip.mo) | **Grounded**: Native type exists in KIP engine. |
| **Validation Bundle** | `nostra.contribution.proof` in [nostra_schemas.mo](file:///Users/xaoj/ICP/nostra/backend/data_layer/config/nostra_schemas.mo) + `test_catalog_latest.json` | **Grounded**: Infrastructure for execution evidence is in place. |
| **Reasoning Graph** | `KipCommand.propositions` and `nostra.argument` schema | **Grounded**: Graph relationships (`@ "pred" "target"`) are core to KIP. |
| **Merge Decision Engine** | `KipExecutor.validator` + `siqs_engine.mo` | **Ready for Implementation**: The orchestration logic (SIQS) exists; the algorithmic decision logic needs hookup. |
| **Governance Tiers** | `Commons` and `Institutions` schemas | **Grounded**: Hierarchical constraint mapping is supported. |

---

## Validated Assumptions

### 1. The "CP" is a Hypothesis
The NCP claim that a contribution is an "Hypothesis" about the future state of the system is perfectly aligned with `nostra.hypothesis` in [nostra_schemas.mo:256](file:///Users/xaoj/ICP/nostra/backend/data_layer/config/nostra_schemas.mo#L256).

### 2. Decision logic belongs in the "Executor"
The `KipExecutor` in `kip.mo` already separates the `parse`, `validate`, and `execute` phases. The "Merge Decision Engine" is functionally equivalent to a rigorous `validator` implementation that checks SIQS `IntegrityRule`s.

### 3. Identity is the Prerequisite
The `AGENTS.md` restriction on `git commit/push` reflects the lack of a mature **Identity & Trust** layer. NCP's proposal for "Trust Scores" is the correct path for loosening these constitutional restrictions.

---

## Identified Friction Points

### Friction 1: Code vs Knowledge Storage
NCP assumes code changes are "Knowledge updates". However, current Git infra stores diffs in `.git`, while Nostra stores metadata in canisters.
**Resolution**: Use the [Contribution Proposal Envelope](file:///Users/xaoj/ICP/research/135-nostra-contribution-protocol/RESEARCH.md) pattern: store the metadata/intent in Nostra and reference the Git commit SHA as the `proof`.

### Friction 2: Orchestration Complexity
Running a full agent consensus for every merge is expensive in the current environment.
**Resolution**: Adopt the [Multi-Tiered Merge Decision Engine](file:///Users/xaoj/ICP/research/135-nostra-contribution-protocol/RESEARCH.md) with a "T0: Automated" bypass for high-confidence, low-risk patches (once Trust Scores are active).

---

## Recommended Phase 1 Alignment
1. **Extend `test_catalog_latest.json`**: Add an `intent_ref` field to link test runs to KIP Proposals.
2. **Implement `T-Review` in A2UI**: Create a "Steward Dashboard" that surfaces KIP Proposals for human evaluation, functioning as the first "Manual" Decision Engine.
3. **Draft NCP-to-KIP Mapping**: Formally define how a Git Pull Request is translated into a KIP `UPSERT` command for a `Proposal` entity.
