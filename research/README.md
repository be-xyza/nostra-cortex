# Research Pipeline

This folder contains research, planning, and decision artifacts for the **Nostra Cortex** ecosystem (`Nostra` = platform authority, `Cortex` = execution runtime).

## Folder Structure

```
research/
├── README.md
├── RESEARCH_INITIATIVES_STATUS.md
├── PORTFOLIO_STRUCTURE_STRATEGY.md
├── PORTFOLIO_MATRIX.md
├── templates/
└── NNN-initiative-name/
    ├── PLAN.md
    ├── RESEARCH.md
    ├── REQUIREMENTS.md
    ├── DECISIONS.md
    ├── FEEDBACK.md
    ├── VERIFY.md
    └── _archive/
```

## Pipeline Workflow

```mermaid
graph LR
    R["RESEARCH"] --> P["PLAN"]
    P --> F["FEEDBACK"]
    F --> D["DEVELOP"]
    D --> V["VERIFY"]
    V --> R
```

## Portfolio Navigation

- Status truth source: `PLAN.md` frontmatter `status`.
- Index source: `RESEARCH_INITIATIVES_STATUS.md` (generated from plans).
- Structural strategy: `PORTFOLIO_STRUCTURE_STRATEGY.md`.
- Operational triage view: `PORTFOLIO_MATRIX.md`.

### Normalized Lifecycle Statuses (as of 2026-02-07)
- `active`: 18
- `draft`: 75
- `paused`: 1
- `deferred`: 1
- `completed`: 5
- `superseded`: 3
- `placeholder`: 2

### Active Initiatives

| ID | Directory | Link |
|---|---|---|
| 007 | 007-nostra-spaces-concept | [Plan](007-nostra-spaces-concept/PLAN.md) |
| 013 | 013-nostra-workflow-engine | [Plan](013-nostra-workflow-engine/PLAN.md) |
| 019 | 019-nostra-log-registry | [Plan](019-nostra-log-registry/PLAN.md) |
| 021 | 021-kip-integration | [Plan](021-kip-integration/PLAN.md) |
| 040 | 040-nostra-schema-standards | [Plan](040-nostra-schema-standards/PLAN.md) |
| 041 | 041-nostra-vector-store | [Plan](041-nostra-vector-store/PLAN.md) |
| 042 | 042-vector-embedding-strategy | [Plan](042-vector-embedding-strategy/PLAN.md) |
| 043 | 043-icp-compute-platform-alignment | [Plan](043-icp-compute-platform-alignment/PLAN.md) |
| 066 | 066-temporal-boundary | [Plan](066-temporal-boundary/PLAN.md) |
| 067 | 067-unified-protocol | [Plan](067-unified-protocol/PLAN.md) |
| 069 | 069-gastown-orchestration-analysis | [Plan](069-gastown-orchestration-analysis/PLAN.md) |
| 096 | 096-offline-sync | [Plan](096-offline-sync/PLAN.md) |
| 097 | 097-nostra-cortex-alignment | [Plan](097-nostra-cortex-alignment/PLAN.md) |
| 098 | 098-rsa-mitigation | [Plan](098-rsa-mitigation/PLAN.md) |
| 099 | 099-upstream-dependency-cleanup | [Plan](099-upstream-dependency-cleanup/PLAN.md) |
| 101 | 101-upstream-warning-remediation | [Plan](101-upstream-warning-remediation/PLAN.md) |
| 104 | 104-blackwell-guardrails | [Plan](104-blackwell-guardrails/PLAN.md) |
| 105 | 105-cortex-test-catalog | [Plan](105-cortex-test-catalog/PLAN.md) |
| 119 | 119-nostra-commons | [Plan](119-nostra-commons/PLAN.md) |

## Agent Operating Rules

1. Read target `PLAN.md` before proposing or implementing changes.
2. Archive target files to `_archive/` before modification.
3. Log architectural or governance choices in `DECISIONS.md`.
4. Keep `status` normalized in plan frontmatter.
5. Update `RESEARCH_INITIATIVES_STATUS.md` when statuses change.
