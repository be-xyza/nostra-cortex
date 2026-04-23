# NCP Integration: Reality Critique

This document critically evaluates the 3 proposed NCP integration patterns against the current architectural realities, constraints, and policies of the Nostra/Cortex ICP ecosystem.

---

## 1. The "Contribution Proposal" Envelope

**The Proposal:** Wrap Git diffs in a YAML/JSON CP envelope containing intent, reasoning (`nostragraph://`), validation tests, and ontology impact.

**The Reality:**
- The project currently relies on standard Git/GitHub mechanics.
- Agents (like the deepmind agents) operate within local IDE environments or via MCP integrations, communicating context ephemerally rather than persisting "reasoning graphs" into an immutable ledger.

**The Friction:**
- **Storage Dissonance:** If the CP is stored in the Git repository (e.g., as a `.cp/` metadata file in the PR), it creates a circular dependency: the CP must describe the diff that includes the CP itself.
- **URI Resolution:** The `nostragraph://reasoning/...` schema does not currently exist as a live, resolvable protocol in Cortex.

**Pragmatic Path Forward:**
Leverage the existing **Test Catalog Contract** defined in `AGENTS.md`.
1. CP metadata shouldn't live *in* the diff array; it should live in the `logs/testing/runs/<run_id>.json` structure as an `agent_intent` and `agent_reasoning` block.
2. The Git commit message or PR body simply contains the pointer: `X-Nostra-Run-ID: <run_id>`. This prevents git-pollution while bootstrapping the CP model using existing JSON artifact patterns.

---

## 2. Multi-Tiered Merge Decision Engine

**The Proposal:** Replace the binary "Merge" button with an algorithm evaluating Governance Tiers, Agent Consensus, and Risk Coefficients.

**The Reality:**
- Nostra has an active Research initiative for Shared Libraries (`018-nostra-library-registry`) which outlines governance-mediated merges.
- However, we currently lean on GitHub Actions (`.github/workflows/test-suite.yml`) for gating.

**The Friction:**
- **Orchestration Cost:** Executing true "Agent Consensus" (having 3 independent agents critique a diff and assign a risk coefficient) is computationally massive and slow. It requires a Temporal workflow (`gastown-orchestration`) capable of multi-agent fan-out/fan-in before a PR even becomes visible to a steward.
- **Platform Hostage:** If GitHub is the host, we cannot natively hide the native "Merge Pull Request" button. We can only block it via Branch Protection Rules requiring a specific status check (e.g., `nostra-decision-engine`).

**Pragmatic Path Forward:**
Do not build a bespoke merge engine yet. Instead, build a single GitHub Action (or Git Hook) that triggers a Temporal workflow. The workflow executes the `test_catalog_latest.json` checks (already documented as mandatory in `AGENTS.md`) and posts the resulting "Risk Coefficient" as a PR Comment. Humans still push the merge button, but they do so based on the CP risk report, serving as a transitional "cyborg" phase.

---

## 3. Decoupling Authority from Code Generation (Agent Trust Scores)

**The Proposal:** Create a Steward Dashboard. Humans set intent; agents write code. Agents with high "Trust Scores" earn the right to auto-merge T0 (formatting/safe) changes.

**The Reality:**
- `AGENTS.md` explicitly forbids autonomous execution: *"git commit/push | ❌ NO | Agents should not push code without review."*
- There is currently no `nostra/backend` canister tracking persistent, mathematically-proven Agent Trust Scores. The `Log Registry` (Research 019) tracks systemic errors, not entity reputation.

**The Friction:**
- **Constitutional Violation:** Enabling T0 auto-merges requires a direct constitutional amendment to `AGENTS.md` and the existing stewardship doctrines.
- **Identity Spraw:** How is an "Agent Identity" cryptographically verified when executing locally on a developer's machine vs. running purely on the Cortex runtime?

**Pragmatic Path Forward:**
Before touching code generation autonomy, we must build the **Identity & Reputation Substrate**.
1. Mint a specific developer/agent capability map (tied to `Space Capability Graph Governance` - Research 130).
2. Maintain the hard ban on `git push` for agents.
3. The Steward Dashboard should be implemented inside A2UI as a "Review Queue" (`cortex-desktop`), where agents prepare perfectly formatted PRs, complete with CP envelopes and test run links, requiring only a single human "Ratify" click. True zero-human autonomy should be deferred to Phase 4.

---

## Conclusion & Proposed Path

The "Nostra Contribution Protocol" is architecturally exactly what Nostra needs conceptually, but requires significant bridging infrastructure before it can replace Git.

**The Pragmatic Route (Phase 1 Integration):**
1. **Do not replace Git yet.** Keep it as the storage layer.
2. **Implement CP Envelopes strictly inside `test_catalog_latest` run structures.** This links intention and reasoning to a specific Git commit hash via CI runs, entirely out-of-band so we don't pollute the diffs.
3. **Build the Steward Review Queue as an A2UI execution surface.** Hook into the GitHub API (`github-mcp-server`) to render PRs alongside their CP validation bundles. The human clicks "Merge" in GitHub, but the decision is entirely derived from the Nostra CP dashboard.
4. **Enforce `AGENTS.md`.** No auto-merging until an Identity & Trust registry is built as a core platform service defining agent provenance.
