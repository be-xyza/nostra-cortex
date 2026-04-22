# AGENTS.md

## Unified Project Vision
The **ICP Ecosystem Project** (comprising Nostra, Cortex, and Research) aims to build a decentralized, collaborative knowledge and execution engine "Nostra" on the Internet Computer.

- **Nostra**: Collaborative application for Spaces, Ideas, Projects, and Workflows (See [nostra/spec.md](file:///Users/xaoj/ICP/nostra/spec.md)).
- **Motoko Maps KG (Archived)**: Personal knowledge graph with AI integration (See [archive/motoko-maps-kg/spec.md](file:///Users/xaoj/ICP/archive/motoko-maps-kg/spec.md)).
- **Research**: A formal pipeline for evolving the ecosystem (See [research/README.md](file:///Users/xaoj/ICP/research/README.md)).

---

## Nostra & Cortex: Naming Standard

> **Nostra Cortex** is the complete system. Internally, we distinguish two layers:

| Layer | Name | Definition | Namespace |
|-------|------|------------|-----------|
| **Platform** | **Nostra** | Platform authority: data model, contributions, governance, spaces, constitutional framework (defines what exists) | `nostra-*` |
| **Execution** | **Cortex** | Execution runtime: workers, agents, workflows, apps, runtime services (defines how work runs) | `cortex-*` |

### Key Boundaries
- **Nostra** defines *what* exists: contributions, spaces, schemas, permissions
- **Cortex** defines *how* work runs: workflows execute, agents process, apps interact
- The **Workflow Engine** is where Nostra's declarative definitions meet Cortex's runtime execution
- Provider inventory, runtime host topology, auth bindings, discovery diagnostics, and execution binding state are **Cortex execution infrastructure**. Detailed visibility and all mutations for these surfaces are operator-only unless an explicitly redacted contract says otherwise.

### Semantic Definitions
- **Space** (Nostra): The sovereign, user-facing container for platforms, providing domains for communities, data isolation, and governance rules. Under no circumstances should this be called a "workspace" in the UI.
- **Workspace** (Developer Layer): A strictly structural developer-side term defining canonical code boundaries (e.g. `nostra/` workspace, `cortex/` workspace). Never surface this term to users!
- **Workbench** (Cortex): The execution environment and application shell (e.g., React/Vite/A2UI) where tools operate on data within a specific Space.

### Naming Conventions
| Component Type | Namespace | Examples |
|---------------|-----------|----------|
| Platform libraries | `nostra-*` | `nostra-core`, `nostra-media`, `nostra-schema` |
| Execution libraries | `cortex-*` | `cortex-worker`, `cortex-agents` |
| Local Daemon | `cortex-desktop` | Headless Temporal worker and local gateway |
| Apps & Labs | `cortex-*` | `cortex-flashcards`, `cortex-monitor` |
| Frontend shell | `nostra-frontend` | Platform UI container (Web) |
| Web execution host | `cortex-web` | Execution-layer web host (React/Vite/A2UI) |
| Combined product | `Nostra Cortex` | External/marketing only |



## Tech Stack

### Backend (Canisters & Workflows)
| Component | Technology |
|-----------|------------|
| Language | **Motoko** (Canisters), **Rust** (Agents/Workers) |
| Workflows | **Hybrid Workflow Authority** (Artifact Pipeline + `nostra-workflow-core` / Execution Adapters) |
| Package Manager | `mops` (Motoko), `cargo` (Rust) |
| Deployment | `icp-cli` (`icp` command, Internet Computer), `temporal` (Workflows) |
| **Config** | `ConfigService` (OnceLock, JSON-matrix) |

### Frontend (Polyglot)
| Component | Technology |
|-----------|------------|
| Host Environment | **React / Vite** (`cortex-web` - Unified Frontend Strategy) |
| UI Protocol | **A2UI** (Abstract Agent UI - JSON) |
| Renderer | **Lit + Shoelace** (Web Components Reference) |
| Visualization | **D3.js v7** (Cosmic Graph via Bridge) |
| Styling | **Tailwind CSS** (Utility-first) |
| Design Standard | **[frontend-design](file:///Users/xaoj/ICP/nostra/commons/skills/frontend-design/SKILL.md)** (Mandatory) |

### AI Agents & Gaming
| Component | Technology |
|-----------|------------|
| Architecture | **Rust** (ArcMind/LDC Labs Pattern) |
| Gaming Bridge | **Godot 4.3** + **Nakama** (via JSON-RPC) |
| Vector DB | **Time-Sliced Indexing** (Micro-batched) |
| **Active Experiments** | `godot_bridge`, `hrm_scheduler`, `nostra-media` |
| **Legacy** | *Python scripts (deprecated: `eudaemon-alpha`, `gardener_agent.py`, `knowledge_graph_agent.py`)* |

---

## Constitutional Framework

**Nostra operates under six foundational constitutions** that govern all agent behavior, knowledge integrity, and system evolution. These are not suggestions—they are operational doctrine.

### Quick Reference
- **Labs Mode**: Imagination wide open, break patterns on purpose
- **Production Mode**: Patterns respected, deviations must justify leverage
- **Sensitive Actions**: Rename, Merge, Archive, Delete, Scope change → Escalate
- **Safe Default**: If authority unclear → Recommendation-only mode
- **Memory**: Preserve lineage, surface uncertainty, resist retrospective certainty

## Agent Topology & Boundary Clarification

We strictly enforce a Bimodal Agent Strategy separating code from data:

1. **Developer Agent (Operator Authority)**
   - **Substrate**: The System Definition (Currently: local Codebase).
   - **Function**: Governs and writes structural protocol changes governed directly by IDE/MCP preflight contracts. It operates locally.
   - **Constraints**: Inherits explicit Steward-Gating. Must pass `nostra-cortex-dev-core` preflight before code permutation.

2. **Runtime Agent (Execution Authority)**
   - **Substrate**: The Instantiated Graph (Currently: Temporal/Hetzner loop via `cortex-eudaemon`).
   - **Function**: Synthesizes and routes workflows over user data.
   - **Constraints**: Possesses ZERO authority to read or mutate the System Definition (codebase). Operates purely autonomously over the graph network.

### Full Framework
Use these canonical sources for complete constitutional guidance:
1. **Labs Constitution** - Experimental culture
2. **UI/UX Manifesto** - Designing for meaning and time
3. **Stewardship & Roles** - Authority as responsibility
4. **Contribution Lifecycle** - Creation to archive governance
5. **Agent Behavior & Authority** - Your operational charter
6. **Knowledge Integrity & Memory** - Truth preservation across time

Primary references:
- `AGENTS.md`
- `docs/architecture/standards.md`
- `docs/architecture/nostra-cortex-boundary.md`

**All agents must align with these constitutions before acting.**

---

## Dos and Don'ts

### Dos
- **Follow the Research Pipeline**: Before starting a major task, read the relevant `research/NNN-name/PLAN.md`.
- **[NEW] Follow System Standards**: Adhere to [docs/architecture/standards.md](file:///Users/xaoj/ICP/docs/architecture/standards.md) for Modularity and Confidence.
- **Use Clean Request Worktrees**: Start new implementation requests in `.worktrees/` via `bash scripts/start_request_worktree.sh --branch <branch>` unless the task is an explicit repo-wide stewardship operation.
- **Protect Execution Infrastructure Surfaces**: Treat provider/runtime/auth admin routes as operator surfaces. Agent-facing routes, automations, and UI affordances must not expose or mutate execution topology, binding, or discovery state without operator authorization.
- **Reference Intake Governance**: For non-core repositories, follow `docs/reference/README.md` and keep `research/reference/index.toml`, `research/reference/index.md`, and `research/reference/analysis/<repo>.md` in sync.
- **Archive Before Update**: Always archive the target file(s) to `_archive/` before modification.
    - **Research**: `PLAN.md`, `REQUIREMENTS.md`, etc.
    - **Core**: `AGENTS.md`, `nostra/spec.md`, `archive/motoko-maps-kg/spec.md`.
    - **Protocol**: `*_archive/Filename_{YYYY-MM-DD}_PRE_UPDATE.md`
- **Log Decisions**: Record architectural choices in `DECISIONS.md` with rationale.
- **Use Frontend Design Skill**: For all UI/UX tasks, you MUST read and follow `frontend-design` skill to ensure distinctive aesthetics.
- **Use `ic-agent` (Rust)**: For all frontend-backend communication.
- **Sync Types**: Treat canister Candid `.did` files as the source of truth for public interfaces (e.g., `nostra/backend/*/*.did`, `nostra/streaming/streaming.did`, `nostra/registry/candid/registry.did`, `nostra/log-registry/candid/log_registry.did`). Keep Motoko domain types in `nostra/backend/**/types.mo` and Rust bindings in `nostra/src/declarations/**` + `nostra/frontend/src/types.rs` aligned with those contracts.
- **Manage Cycles**: Implement `freezing_threshold` and conservative cycle limits per call (See `docs/best-practices/general.md`).
- **Use Standard Specs**: Adhere to `ResourceRef` and `Event` standards in `shared/`.
- **Log Errors**: Use the centralized [Log Registry](research/019-nostra-log-registry/PLAN.md) for agent error reporting.
- **Canister Logging**: Enable controller-only canister log visibility in the active ICP project manifest, and keep any legacy manifest config aligned during migration.
- **Promote Immutable Evidence**: Keep mutable runtime outputs in `logs/` local, and preserve durable evidence by promoting immutable copies into governed initiative surfaces.

### Don'ts
- **No Python Agents**: Do not propose or implement new Python-based agents; use Rust/WASM. Off-chain ML inference kernels (e.g., HRM via PyTorch MPS) and one-off utility scripts are exempt when execution is sandboxed outside canisters.
- **No Hardcoded IDs**: Do not hardcode canister IDs in source; use environment variables or dynamic lookup.
- **No Default-Readable Runtime Topology**: Do not make provider inventory, runtime host data, auth bindings, discovery diagnostics, or resolved runtime status readable to general or agent-facing surfaces by default.
- **No Unbounded Loops**: Avoid iterating over potentially infinite data structures in Canisters (DoS risk).
- **No Direct DOM**: Avoid direct JS DOM manipulation outside of standard React paradigms or legacy `dioxus::eval` bridges.
- **Reserved Keywords**: Do not use `actor` or `query` as variable names in Motoko.
- **No Dirty Tree as Memory**: Do not rely on a long-lived dirty worktree or `git stash` as the primary persistence mechanism for important in-flight work.
- **No Tracked Generated Runtime Outputs**: Do not keep generated build artifacts, mutable log projections, or local test outputs tracked in governed repo paths.

---

## Reference Intake Protocol (Agents)

Use this for any non-core repository entering the research reference extension.

### Required Lifecycle
1. **Analyze** candidate intent and relation to the Nostra platform and Cortex runtime.
2. **Classify** using scorecard + placement matrix.
3. **Place** in `research/reference/repos/<repo>` or `research/reference/topics/<topic>/<repo>`.
4. **Register** metadata in `research/reference/index.toml` + `research/reference/index.md`.
5. **Document** rationale in `research/reference/analysis/<repo>.md`.

### Scorecard Fields (0-5 each)
- `ecosystem_fit`
- `adapter_value`
- `component_value`
- `pattern_value`
- `ux_value`
- `future_optionality`
- `topic_fit`

### Placement Matrix
- Use existing topic when `topic_fit >= 4`.
- Create a new topic only when `topic_fit <= 3` and at least 2 related repos are expected within 60 days.
- Use `research/reference/repos/<repo>` when cross-topic and still valuable.
- Reject intake when total value score (`ecosystem_fit + adapter_value + component_value + pattern_value + ux_value + future_optionality`) is below 12 and no active research initiative references it.

### Required Narrative Fields
- `why_here`
- `links_to_nostra_cortex`
- `known_risks`
- `suggested_next_experiments`
- `primary_steward`
- `authority_mode`
- `escalation_path`
- `lineage_record`
- `initiative_refs`

### Operational Contract
- Command contract: `reference intake` = analyze -> classify -> place -> register metadata -> refresh docs/index.
- Command contract: `knowledge intake` = categorize -> folderize -> metadata -> register in knowledge index.
- Default mode is `recommendation_only`; sensitive structural actions (`rename`, `merge`, `archive`, `delete`, scope/root changes) require steward escalation.
- If policy changes, archive and update `AGENTS.md` first.
- Detailed procedure and current local validation contract: `docs/reference/README.md`.

---

## Test Catalog Contract (Agents)

Use this for all local IDE agent test execution evidence in v1.

### Canonical Files
- `/Users/xaoj/ICP/logs/testing/test_catalog_latest.json`
- `/Users/xaoj/ICP/logs/testing/runs/<run_id>.json`
- `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json`

### Required Command Contract
- `test catalog refresh`:
  1. Write/append run artifact under `logs/testing/runs/`.
  2. Regenerate `test_catalog_latest.json`.
  3. Recompute `test_gate_summary_latest.json`.
  4. Run `scripts/check_test_catalog_consistency.sh` in requested mode.

### Required Fields
- Catalog entries MUST include:
  - `test_id`, `name`, `layer`, `stack`, `owner`, `path`, `command`
  - `artifacts`, `gate_level`, `destructive`, `tags`
  - `last_seen_commit`, `updated_at`
- Run artifacts MUST include:
  - `run_id`, `started_at`, `finished_at`, `agent_id`, `environment`, `git_commit`
  - `results[]` with `test_id`, `status`, `duration_ms`, `error_summary`
  - `artifacts`, `warnings`, `schema_version`
- Gate summary MUST include:
  - `generated_at`, `mode`, `catalog_valid`, `run_artifacts_valid`
  - `required_blockers_pass`, `overall_verdict`, `failures`, `counts`

### Naming Rules
- `test_id`: stable repo-scoped id, deterministic from test target path.
- `run_id`: UTC timestamp + slug; no path separators.
- Optional A2UI projection surface IDs:
  - `system_test_catalog:<run_id>`
  - `system_test_gate:<run_id>`

### Failure Policy
- If artifact generation or consistency checks fail, agents MUST mark the test operation incomplete and return explicit failure reasons.
- In blocking mode, missing/invalid artifacts or failing release blockers MUST fail the operation.

---

## Clean-State Operations Contract (Agents)

Use this for request lifecycle hygiene, recovery, and evidence promotion.

### Authority Model
- Authored source, governed docs, and research artifacts are Git authority.
- Mutable runtime/build/test outputs are local operator surfaces, not Git authority.
- `logs/` remains the canonical runtime output location, but mutable artifacts under `logs/` must stay reproducible and local.
- When evidence must be preserved, promote an immutable copy into a governed initiative surface instead of tracking the mutable runtime output directly.

### Request Lifecycle
1. Create a recovery snapshot before bulk hygiene work:
   - `bash scripts/create_repo_recovery_snapshot.sh`
2. Start request work in a clean request worktree:
   - `bash scripts/start_request_worktree.sh --branch codex/<request-name>`
3. Save durable pause points:
   - `bash scripts/checkpoint_request.sh`
4. Close requests with hygiene checks and emitted closeout evidence:
   - `bash scripts/close_request.sh`
5. Prune stale worktrees:
   - `bash scripts/worktree_gc.sh`

### Worktree Rules
- `.worktrees/` is the canonical request-worktree location for this repo.
- The shared root worktree is reserved for repo-wide stewardship tasks only.
- Every pause point must have a durable checkpoint via a WIP commit or an explicit patch bundle.
- Unpushed branches and unstaged/untracked work must be surfaced before closeout or handoff.

### Evidence Promotion
- Promote immutable evidence with:
  - `bash scripts/promote_evidence_artifact.sh --source <path> --initiative <research-dir>`
- Docs and research should reference promoted immutable evidence or the command contract that regenerates the runtime output, not tracked mutable `*_latest.*` artifacts.

---

## Commands

### Research & Planning
```bash
# Start new research initiative from template
cp -r research/templates research/NNN-feature-name

# Run test suite (General)
cargo test
```

### Backend (Motoko/Rust)
```bash
# Build all canisters (check for errors) via `icp-cli`
icp build

# Deploy all canisters (Locally) via `icp-cli` - WARNING: Modifies state
icp deploy

# Add a Motoko dependency
mops add <package_name>
```

### Frontend (React/Web)
```bash
# Install dependencies
npm -C cortex/apps/cortex-web install

# Run Development Server
npm -C cortex/apps/cortex-web run dev
```

### Repo Hygiene
```bash
# Create a recovery bundle before bulk cleanup
bash scripts/create_repo_recovery_snapshot.sh

# Start clean request work in a dedicated worktree
bash scripts/start_request_worktree.sh --branch codex/my-change

# Save a durable checkpoint before context switching
bash scripts/checkpoint_request.sh

# Close a request with hygiene verification
bash scripts/close_request.sh
```

---

## Safety and Permissions

| Action | Safe to Auto-Run? | Description |
|--------|-------------------|-------------|
| `cargo check`, `icp build` | ✅ **YES** | Read-only build checks through `icp-cli`. |
| `dx build` | ✅ **YES** | Compiles frontend assets. |
| `cat research/.../*.md` | ✅ **YES** | Reading research context is encouraged. |
| `icp deploy` | ❌ **NO** | Modifies local canister state/cycles. Ask user. |
| `icp canister call ...` | ❌ **NO** | Executes canister logic. Ask user. |
| `git commit/push` | ❌ **NO** | Agents should not push code without review. |

---

## Project Structure

```
ICP/
├── AGENTS.md           # This file (Master Guide)
├── nostra/             # Nostra V2 Specification & Docs
│   ├── worker/         # AI Worker Node (Rust)
│   ├── streaming/      # WebSocket Streaming Canister
├── cortex/             # Cortex execution-layer experiments and runtime prototypes
├── src/                # Root Rust workspace crates
├── research/           # Research Initiatives (Source of Truth)
│   ├── 001...NNN/      # Active initiatives
│   └── README.md       # Research Workflow Guide
├── docs/               # Curated docs split by Nostra, Cortex, Environment, Reference
├── research/reference/ # Canonical non-core repositories and topic bundles
│   ├── topics/         # Capability/domain bundles (including vendor ecosystems)
│   ├── repos/          # Standalone cross-topic repositories
│   ├── knowledge/      # Static knowledge artifacts (Papers, Standards)
│   ├── analysis/       # Per-repository analysis notes
│   ├── inbox/          # Temporary unresolved intake
│   ├── index.toml      # Machine-readable repo catalog
│   └── index.md        # Human-readable repo catalog
├── sdk/                # Shared SDKs/Libraries
├── libraries/          # Shared Rust libraries used by Nostra workspace dependencies
├── shared/             # Unified Standards (types, schemas)
├── canisters/          # IC canister projects
├── scripts/            # Workspace scripts and checks
├── tests/              # End-to-end and simulation tests
├── logs/               # Error logs (See logs/README.md)
└── ic-rmcp/            # MCP Server Implementation
```

---

## Examples

### Good Code: Research Plan
`research/001-multi-project-architecture/PLAN.md`
- Clear phases, defined verification steps, and integration roadmap.

### Good Code: Backend Seed
`archive/motoko-maps-kg/backend/main.mo`
- `seedICPData` pattern for initializing data without external dependencies.

### Legacy (Do Not Copy)
- **Python Agents**: `knowledge_graph_agent.py`, `gardener_agent.py`. (Use Rust/WASM instead).
- **Old Cycles**: `ExperimentalCycles.add` (Use `Cycles.add` or `with cycles`).

---

## API / Resources

### Local References
- [ICP Best Practices](file:///Users/xaoj/ICP/docs/best-practices/general.md) - Security, scaling, and cycles.
- [Nostra Spec](file:///Users/xaoj/ICP/nostra/spec.md) - Full V2 architecture.
- [Research README](file:///Users/xaoj/ICP/research/README.md) - How to contribute to research.

### External Documentation
- [Internet Computer Docs](https://internetcomputer.org/docs/)
- [React Documentation](https://react.dev/)
- [Motoko Library (Mops)](https://mops.one/)
