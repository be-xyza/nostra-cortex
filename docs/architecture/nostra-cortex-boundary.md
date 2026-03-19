# Nostra-Cortex Boundary Contract

## Purpose
This document defines the canonical boundary to prevent terminology and architecture drift.

## Canonical Definitions
- Nostra = platform authority (what exists)
- Cortex = execution runtime (how work runs)
- Nostra Cortex = product umbrella (external-facing only)
- `Nostra`: Platform authority layer. Defines canonical entities, contribution model, governance, schemas, and policy constraints. Nostra defines what exists.
- `Cortex`: Execution runtime layer. Executes workflows, agents, orchestration, and runtime services. Cortex defines how work runs.
- `Nostra Cortex`: Product umbrella label for external-facing references only.

## Authority Model
- Canonical truth for governance and platform state lives in Nostra protocol authority surfaces (canisters and contracts).
- Cortex may enforce and validate locally for UX/runtime behavior, but cannot override protocol authority.

## Naming Rules
- Platform crates and services: `nostra-*`
- Execution crates and services: `cortex-*`
- Avoid slash-separated Nostra and Cortex naming in canonical docs. Use explicit phrasing:
  - `Nostra platform`
  - `Cortex runtime`
  - `Nostra Cortex` (umbrella only)

## Documentation Rules
- Canonical docs must include boundary-consistent wording:
  - `/Users/xaoj/ICP/AGENTS.md`
  - `/Users/xaoj/ICP/nostra/spec.md`
  - `/Users/xaoj/ICP/research/README.md`
  - `/Users/xaoj/ICP/docs/reference/README.md`
- New architecture or protocol docs should reference this file when defining system layers.

## Technology vs Product Boundaries

Beyond Nostra-Cortex naming, the system must distinguish between **technologies** (protocol-level patterns) and **products** (hosted platforms built on those technologies). The primary case:

### Git (Technology) vs GitHub (Product)

| Layer | Term | Definition | Example Usage |
|---|---|---|---|
| **Protocol** | `git` | Distributed version control system. Content-addressable DAG of commits, branches, remotes. | `upstream = "https://example.com/repo.git"` — a git remote URL |
| **Product** | `GitHub` | Microsoft-owned hosted platform built on git. Adds Issues, PRs, Actions, Discussions, Pages, API. | `github_mcp_service` — integration with GitHub's REST/GraphQL API |

### Naming Rules
- **Reference index** (`research/reference/index.toml`): The `upstream` field is a **git remote URL**. It may point to GitHub, GitLab, Codeberg, or any git host. Do not treat it as a GitHub-specific field.
- **MCP services**: Name services after what they integrate with. `github_api_service` for GitHub product features. `git_operations_service` for protocol-level VCS operations. Avoid ambiguous names like `github_mcp_service` that conflate the two.
- **Spec references**: Platform-layer docs (e.g., `nostra/spec.md`) must use provider-neutral examples. Prefer "Git Repository" over "GitHub Repo" when describing generic external data sources.
- **Workflow tooling**: CI pipelines in `.github/workflows/` are GitHub-product-specific by definition — this is correct and expected. No renaming needed.

### Why This Matters
Nostra's contribution model (fork, merge, lineage) is **inspired by** git's architectural pattern but is **not dependent on** git or any git hosting product. Maintaining this distinction:
- Preserves platform sovereignty (Nostra is not a GitHub feature)
- Enables future provider diversity (GitLab, self-hosted forges, IPFS-backed repos)
- Prevents semantic conflation in specs and documentation

## Enforcement
- CI terminology lint (`scripts/check_nostra_cortex_terminology.sh`) must pass for canonical docs.
