---
id: '071'
name: github-management-strategy
title: 'Research Initiative: GitHub Management Strategy & Contribution Lifecycle'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research Initiative: GitHub Management Strategy & Contribution Lifecycle

## 1. Executive Summary

This study defines the strategy for managing the Nostra project's source control on GitHub, ensuring alignment with the [Nostra Contribution Lifecycle Constitution](../034-nostra-labs/NOSTRA_CONTRIBUTION_LIFECYCLE_CONSTITUTION.md) and [Contribution Types](../008-nostra-contribution-types/PLAN.md).

**Core Goal:** To create a seamless bridge between **Nostra Web** (Governance, Ideation, Strategy) and **GitHub** (Execution, Code, Release), treating GitHub as a downstream execution environment for upstream intent defined in Nostra.

## 2. The Philosophy: Nostra as Head, GitHub as Hands

The central principle is that **GitHub is not the source of truth for "Why" or "What"—only "How".**

*   **Nostra Web**: Defines the *Intent* (`Idea`, `Issue`, `Project`, `Decision`).
*   **GitHub**: Executes the *Change* (Code, PRs, CI/CD).

### The "Lifecycle Bridge"
We must track the lifecycle from a user's raw `Feedback` in Nostra to a merged `Commit` in GitHub.

## 3. Recommended Processes & Best Practices

### A. The "Issue" Lifecycle
The lifecycle moves from *Signal* to *Code* to *Release*.

1.  **Ingestion (Nostra)**:
    *   User posts `Feedback` or `Idea` on Nostra Web.
    *   **Gap**: Currently no automated link to dev tracks.
    *   **Process**: Triage workflow converts `Feedback` -> Nostra `Issue`.

2.  **Handoff (Sync)**:
    *   **Proposed**: Automated sync or manual "Dev Export".
    *   Nostra `Issue` (ID: `NST-123`) -> GitHub Issue (Title: `[NST-123] Title`).
    *   **Rule**: Every GitHub Issue *must* link back to a Nostra parent (Issue, Project, or Bounty). No "orphan" GitHub issues allowed for feature work.

3.  **Implementation (Git Workflows)**:
    *   **Branching**: adoption of a standard naming convention.
        *   Format: `type/nostra-id-short-description`
        *   Example: `feat/NST-123-add-dark-mode`
    *   **Worktrees**: Mandatory for parallel Contexts.
        *   Developers often switch between "Planning" (Docs) and "Exec" (Code).
        *   **Standard**: Use `git worktree` to maintain separate checkouts for `hotfix`, `feature`, and `experiment` to avoid `git stash` hell and preserve context.
        *   *See Section 5 for Worktree details.*

4.  **Contribution (Pull Requests)**:
    *   **Title**: `feat(scope): Description [NST-123]`
    *   **Body**: Must reference the Nostra "Intent".
        *   "Implements Decision: [Link]"
        *   "Fixes Issue: [Link]"
    *   **Validation**: CI checks that the PR links to a valid Nostra ID.

5.  **Completion (Merge)**:
    *   **Constitution Check**: Does this merge preserve lineage?
        *   *Adopt*: **Squash & Merge** (generally) for clean history, BUT the commit message must be the "Saga" summary.
        *   *Exception*: Long-lived feature branches may need "Merge Commit" to preserve the "parallel curiosity" (Forking/history) principle.

### B. Constitutional Alignment
*   **"Erasure is not"**: Avoid `git push --force` on shared branches.
*   **"Lineage is sacred"**: Commit messages are not trash; they are part of the `Artifact`.
*   **"Interpretability"**: Code comments and Commit messages must explain *Why*, not just *What*.

## 4. Deep Research: The "Process" & Sync Gap

The "Process" gap identified ("Research docs in folders vs. Code in branches") is critical. Production systems require that **Research Documentation be treated as Code**.

### A. The "DevBrain" Sync Strategy (Doc-Code Bridge)
We propose a bidirectional synchronization protocol managed by a **CLI Agent** (tentatively `devbrain` or `nostra`).

**1. The "Single Source of Truth" Problem**
*   *Observation*: Code evolves faster than docs (e.g., `018-library-registry` spec drifting from `n-lib.json` implementation).
*   *Solution*: **Active Verification**. The CLI must validate that the *Intent* (Doc) matches the *Reality* (Code).

**2. The CLI Workflow**
*   **Pull (Downstream)**:
    *   `nostra pull research/071`
    *   *Action*: Fetches the artifact from Nostra Web.
    *   *Verification*: checks hash integrity.
    *   *Placement*: Hydrates into `.worktrees/research-071/` (Documentation Context).
*   **Push (Upstream)**:
    *   `nostra push research/071`
    *   *Action*: Pushes local Markdown edits to Nostra as a "Draft Proposal" or "Version Update".
    *   *Validation*: Runs a **"Doc Linter"** before push.
        *   *Check 1*: Do referenced files exist?
        *   *Check 2*: Do wikilinks resolve?
        *   *Check 3*: Are "Must Fix" todos addressed?

### B. The Optimal Traceability Path (Feedback to Commit)

We define the following **Gold Standard Traceability Loop**:

| Stage | Actor | Artifact | System Action |
| :--- | :--- | :--- | :--- |
| **1. Signal** | User | **Feedback** (`FB-101`) | Ingested via Loom. Assigned "Triage" status. |
| **2. Intent** | PM/Agent | **Issue** (`ISS-50`) | Linked to `FB-101`. `PLAN.md` created/updated. |
| **3. Context** | Developer | **Worktree** | `nostra start ISS-50`. Creates `.worktrees/fix-sync`. Fetches `PLAN.md`. |
| **4. Execution** | Developer | **Commit** | `fix: handle null (Ref: ISS-50)`. CLI enforces footer. |
| **5. Verification** | CI/CD | **Build** | CI checks: "Does `ISS-50` exist? Is it Active?". |
| **6. Closure** | Maintainer | **Merge** | PR Merge triggers Webhook -> Moves `ISS-50` to "Done". |
| **7. Loop** | System | **Notification** | User who created `FB-101` gets "Fixed" notification. |

## 5. Research Cohesion Strategy (The Answer to Drift)

**Problem**: How do we prevent 100+ research documents from becoming a "Web of Lies" where file A references a deprecated version of file B?
**Answer**: Treat Research as a Software Library with explicit dependencies.

### A. Research Versioning (SemVer for Thought)
We apply **Semantic Versioning** to Research Initiatives.
*   **Format**: `RES-071@1.2.0`
*   **Change Rules**:
    *   **Major (1.x)**: Fundamental paradigm shift (e.g., "We no longer use Canisters").
    *   **Minor (x.2)**: New strategy added (e.g., "Added Executable Docs").
    *   **Patch (x.x.3)**: Typos, clarifications.

### B. The Dependency Graph
Just as `Cargo.toml` defines code dependencies, we introduce `manifest.yaml` for Research Initiatives.

**Example: `071-github-strategy/manifest.yaml`**
```yaml
id: "nostra.research.071"
version: "1.0.0"
dependencies:
  - "nostra.research.008@^2.0.0" # Contribution Types
  - "nostra.research.034@^1.0.0" # Labs Constitution
assertions:
  - type:file_exists: "RESEARCH.md"
  - type:validates_against: "nostra.research.034/NOSTRA_CONTRIBUTION_LIFECYCLE_CONSTITUTION.md"
```

### C. Executable Imports (Transclusion)
Instead of copy-pasting, we use **Executable Imports** to inject authoritative content.
```markdown
<!-- @import: nostra.research.008/PLAN.md#contribution-types-table -->
```
*   **Drift Prevention**: If the source table in `008` changes, this document updates automatically on the next build.
*   **Traceability**: The reader can click through to the source of truth.

### D. The Cohesion Layer (The "Linker")
The `devbrain` CLI acts as the **Linker**.
*   **Run**: `nostra audit`
*   **Action**: Scans all `manifest.yaml` files.
*   **Check**: "Research 071 relies on 008@^2.0.0, but 008 is currently @3.0.0 (Major Change). **BROKEN**."
*   **Result**: The build fails. The owner of 071 is notified: "Your dependency changed. Review and upgrade."

**Why this is Optimal**: It turns "Social Cohesion" (remembering to tell people things changed) into "System Cohesion" (the build breaks if you don't).

## 7. Deployment & Opportunities

### A. Gaps & Opportunities

| Gap | Current State | Opportunity (The "Fix") |
| :--- | :--- | :--- |
| **Doc Drift** | Docs are static text files. | **Executable Docs**: `PLAN.md` should contain "Testable Assertions" that the CLI runs (e.g., "File X should exist"). |
| **Context Switching** | Manual `cd` between research/code. | **Context Injection**: Opening a Worktree should auto-open the relevant `RESEARCH.md` in the editor. |
| **Attribution** | Commits are just lines of text. | **Signed Lineage**: Using crypto-signing to prove that *this* commit authorized by *that* generic Nostra Account. |

### B. Git Features to Incorporate (Refined)

1.  **Git Worktrees (Mandatory)**:
    *   As defined in `skills/using-git-worktrees`.
    *   **New Rule**: "Every Research Initiative gets a Worktree." (e.g., `.worktrees/research-071`).
2.  **Git Notes**:
    *   Use `git notes` to attach metadata (Nostra IDs, status) to commits *without* changing the commit hash. This allows updating "Status" (e.g., "Deployed") on historical commits.
3.  **Hooks**:
    *   `pre-push`: Block push if `RESEARCH.md` has uncommitted changes that contradict the code.

### C. Multi-Agent Integration (The "Agent Zero" Link)
This Worktree strategy is the foundational requirement for **Parallel Agent Execution** (Ref: `057-development-brain`, Phase 6).

*   **The Architecture**:
    *   **Orchestrator**: The Workflow Engine (`013`) dispatches logical tasks.
    *   **Runner**: The `nostra-worker` (`057`) picks up tasks.
    *   **Isolation**: The worker uses `WorktreeService` to spin up a dedicated worktree for that specific Agent Task (e.g., `~/.nostra/hooks/task-123`).
*   **Why this matters**: It allows 5 agents to work on 5 different features simultaneously without fighting over the same file system `CWD`.
*   **Visibility**: These active worktrees are surfaced in the **Cortex System Monitor** (`033`) under "Agent Activity".

## 8. Implementation Plan (Strategy)

1.  **Formalize the "Nostra ID"**: The bridging key.
2.  **Develop `git-nostra` CLI**: A wrapper tool.
    *   `nostra start NST-123` -> Creates worktree `.worktrees/feat/NST-123...`.
    *   `nostra submit` -> Pushes and opens PR with template pre-filled from Nostra data.
3.  **Define the "Repository Structure"**:
    *   Monorepo (Likely).
    *   Documentation alongside code (Preserves context).

## 9. Conclusion
To Manage GitHub "The Nostra Way" is to treat it as a subservient execution engine. The "Brain" is Nostra; the "Hands" are Git. The strategy is to strictly enforce the linkage using tooling (Worktrees, CLI) and Process (Naming Conventions), ensuring no code line exists without a "Constitutional" ancestor.
