# Promotion and Migration Rubric

## Summary

Use this rubric when deciding whether a semantic primitive should be promoted,
renamed, migrated with compatibility, deferred, or left unchanged.

The goal is to reduce case-by-case philosophy and make rename timing explicit
when semantic drift is already visible.

## Two Change Classes

### Preventative Maintenance

Use this class when:
- semantic misalignment is already known
- a clearer replacement is already governed or explicitly chosen
- the current migration surface is still bounded
- delaying the change would increase propagation risk across routes, docs,
  tests, or user expectations

### Needs-Based Improvement

Use this class when:
- current users or operators are already blocked, confused, or harmed
- the benefit is immediate rather than primarily debt reduction
- the migration cost is justified even if the surface is broad

## Evaluation Factors

Score the change qualitatively against these factors:
- semantic misalignment known
- future conflict concrete
- replacement governed
- migration surface bounded
- delay increases drift
- compatibility cost

## Decision Outputs

- `rename now`
  Use when misalignment is known, the replacement is clear, and the migration
  surface is small enough that delay mostly compounds drift.

- `rename with redirect`
  Use when the same conditions hold but compatibility matters for existing links,
  tests, or habits.

- `defer and reserve`
  Use when the conflicting future concept is known but the current migration
  surface is too broad or the replacement term is not yet governed.

- `no action`
  Use when the current term is already principle-aligned or the proposed change
  would create more confusion than clarity.

## Worked Example

`gallery -> catalogue` for the experimental layout comparison surface in
`cortex-web` is a `rename with redirect` decision:
- semantic misalignment is known
- future conflict is concrete because `gallery` is reserved
- the replacement term `catalogue` is already governed
- the migration surface is still small
- a redirect from `/gallery` avoids unnecessary breakage while ending further
  propagation of the wrong canonical term
