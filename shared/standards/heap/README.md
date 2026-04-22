# Heap Block Standards

This folder contains **governed, stable schemas** for structured Heap blocks.

Principles:
- Heap blocks should be **schema-tagged** (`schema_id`) and **renderable generically** (no bespoke widget branches).
- Operational surfaces remain **Workbench-first**; Heap blocks are compact, portable summaries + pointers.
- Schemas are **append-only** and should remain backward compatible (additive fields).

## Gate Summary Block
See `gate_summary_block.schema.json` for a canonical structured payload that can represent both:
- SIQ gate summaries
- Testing gate summaries
