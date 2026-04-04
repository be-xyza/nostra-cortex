# Graph Promotion Criteria

Date: 2026-04-03
Status: active policy for Initiative `078` closeout

This note defines when supporting evidence should become graph-native in the Nostra commons model.

## Default Rule

Supporting evidence is **not graph-native by default**.

If an item can be handled correctly as retrieval support content, benchmark evidence, or a downstream document citation, we should keep it out of the canonical graph until promotion is justified.

## Promotion Criteria

Promote an item into the canonical graph layer only when all of the following are true:

1. It has stable identity as a `Contribution` or another canonical Nostra entity.
2. It participates in reusable semantic relations rather than a one-off benchmark edge.
3. It matters across multiple queries, Spaces, or consumers instead of only one evaluation case.
4. Its promotion improves semantic authority, provenance reasoning, or traversal quality, not only retrieval recall.

## Evaluation Questions

Before promoting an item, answer these questions explicitly:

- Does the item need to be traversed through the canonical triple facade?
- Does `136` or another downstream consumer need it as typed graph structure rather than as supporting citation content?
- Does the item belong to Nostra semantics, or is it better handled by Cortex retrieval/runtime evidence?
- Would adding it change the meaning of existing graph queries, topology projections, or bundle semantics?

If any answer is uncertain, stay in recommendation-only posture and keep the item out of the canonical graph for now.

## Current Classification

- `hypothesis-brief`: **not yet graph-native by default**
  - It is currently useful as support-document evidence in retrieval evaluation.
  - It has not yet shown repeated cross-consumer need as a first-class graph entity.
  - It remains eligible for future promotion if repeated benchmark and consumer evidence justify it.

## Architectural Boundary

- Nostra decides what becomes graph-native.
- Cortex may surface retrieval evidence, benchmark gaps, and candidate promotions.
- Cortex must not promote new graph-native evidence classes opportunistically just to improve a single benchmark.
