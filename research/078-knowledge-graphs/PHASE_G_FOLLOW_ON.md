# Phase G Follow-On

Date: 2026-04-03
Status: deferred follow-on after Initiative `078` closeout

This note records the work that was intentionally left out of the `078` closeout so the initiative can finish cleanly without re-opening the ratified graph baseline.

## Deferred Items

### 1. Retrieval-Policy Inputs from `M23`

Phase F proved the bounded graph retrieval pilot without fully wiring `KnowledgeBundle` retrieval-policy inputs into runtime execution.

Phase G should:

- map bundle retrieval-policy fields into the internal retrieval planner explicitly
- define how policy affects graph-only, vector-only, and hybrid execution
- keep policy consumption downstream of the frozen bundle contract rather than expanding the contract itself

### 2. Low-Confidence and Incomplete-Coverage Handling

Phase F proved citation-bearing retrieval and graph-native benchmark coverage, but it did not yet add a formal response strategy for weak or incomplete graph evidence.

Phase G should:

- define low-confidence thresholds for graph and hybrid retrieval
- define what counts as incomplete graph coverage
- add bounded fallback behavior for uncertain answers
- preserve provenance/citation visibility when the system degrades gracefully

## Guardrails

- Do not reopen the ratified ontology, bundle, or triple-query contracts unless a stewarded incompatibility is discovered.
- Do not promote new graph-native evidence classes just to improve a benchmark.
- Keep `037` as the current user-facing knowledge path until a later graduation decision is made.
- Keep Explore consuming derived topology payloads, not raw retrieval internals.

## Entry Criteria

Start Phase G only when:

1. the `078` closeout PR is reviewed,
2. the current bounded benchmark/evidence lane is accepted,
3. and the next work is clearly about operational retrieval behavior rather than graph contract definition.
