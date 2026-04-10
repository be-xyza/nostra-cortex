# Logs Directory

This directory contains error logs and debug information for ICP ecosystem projects.

## Log Categories
| Folder | Purpose |
|--------|---------|
| `nostra/` | Nostra Cortex backend/worker logs |
| `motoko-maps-kg/` | Knowledge Graph project logs |
| `siq/` | System Integrity + Quality artifacts and gate runs |

## Conventions
- **Format**: `YYYY-MM-DD-HH-MM-{component}.log`
- **Retention**: Logs older than 7 days can be purged.
- **Sensitive Data**: Never log private keys, principals, or user content.

## Reference
See [AGENTS.md](file:///Users/xaoj/ICP/AGENTS.md) for project-wide conventions.
