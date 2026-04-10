# Logs Directory

This directory is an operator/runtime surface for generated logs, gate outputs, and transient evidence. It is not the Git authority surface for mutable runtime state.

## Categories
| Folder | Purpose |
|--------|---------|
| `alignment/` | Contract and alignment scanner outputs |
| `knowledge/` | Knowledge-engine and graph pilot runtime outputs |
| `siq/` | System Integrity + Quality gate outputs |
| `testing/` | Test catalog runs and gate summaries |

## Authority Rules
- Mutable runtime outputs in `logs/` stay local and are ignored by Git.
- `*_latest.*` files are convenience projections, not durable evidence.
- When evidence must be preserved, promote an immutable copy into a governed initiative surface with `bash scripts/promote_evidence_artifact.sh`.
- Existing docs may reference canonical runtime paths under `logs/`, but those paths should be reproducible from command contracts rather than committed as mutable repository state.

## Conventions
- **Format**: timestamped outputs should prefer immutable names when they need to survive regeneration.
- **Retention**: local operator logs older than 7 days can be purged unless explicitly promoted.
- **Sensitive Data**: never log private keys, principals, or user content.

## Reference
See [AGENTS.md](file:///Users/xaoj/ICP/AGENTS.md) and [repo-clean-state.md](file:///Users/xaoj/ICP/docs/architecture/repo-clean-state.md) for project-wide conventions.
