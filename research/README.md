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

### Active Initiatives & Status Counts

> [!NOTE]
> The active initiative list and lifecycle counts are maintained in the canonical index: **[`RESEARCH_INITIATIVES_STATUS.md`](RESEARCH_INITIATIVES_STATUS.md)**.
> 
> Please refer to that file for the current, accurate list of all initiatives and their statuses.

## Agent Operating Rules

1. Read target `PLAN.md` before proposing or implementing changes.
2. Archive target files to `_archive/` before modification.
3. Log architectural or governance choices in `DECISIONS.md`.
4. Keep `status` normalized in plan frontmatter.
5. Update `RESEARCH_INITIATIVES_STATUS.md` when statuses change.
