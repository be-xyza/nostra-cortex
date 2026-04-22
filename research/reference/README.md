---
id: ""
name: reference
title: Research Reference Workspace
type: general
project: nostra
status: draft
authors:
  - User
tags: []
created: "2026-02-06"
updated: "2026-04-22"
---

# Research Reference Workspace

`research/reference` is the canonical metadata and analysis surface for non-core repositories, topic bundles, and static knowledge artifacts. It is not a storage root for heavyweight live repository mirrors.

## Canonical Files

- `research/reference/index.toml` tracks repository metadata.
- `research/reference/index.md` provides the human-readable repository catalog.
- `research/reference/analysis/*.md` records placement, rationale, links, risks, and next experiments.
- `research/reference/knowledge/index.toml` tracks static knowledge artifacts with metadata sidecars.

## Mirror Boundary

External repositories are volatile runtime state. Fetch or clone them only through the Cortex repo ingestion boundary, governed by `research/127-cortex-native-repo-ingestion/ingestion_registry.toml`, into `cortex-memory-fs/sandboxes/` or an ignored local cache.

Local cache default: `.cache/reference-mirrors/`

Dry-run migration helper:

```bash
bash scripts/migrate_reference_mirrors_to_cache.sh
```

Apply mode is intentionally conservative and refuses nested mirror collisions:

```bash
bash scripts/migrate_reference_mirrors_to_cache.sh --apply
```

## Agent Contract

- `reference intake`: analyze -> classify -> place -> register metadata -> refresh docs/index.
- `reference intake validate`: `python3 scripts/check_reference_metadata_v2.py --strict`.
- Default authority mode: `recommendation_only`.
- Sensitive structural actions require steward escalation and decision logging.

Detailed policy is in `docs/reference/README.md`.
