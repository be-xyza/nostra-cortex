---
name: Eudaemon Alpha Phase 6 Bring-Up
about: Track the Hetzner bring-up and post-go-live hardening gates for Eudaemon Alpha
title: "Phase 6 Hetzner Bring-Up"
labels: ops, eudaemon-alpha, phase-6
assignees: ""
---

## Phase 0: Stabilize Operator Setup

- [ ] Clean root clone created with `--recurse-submodules`
- [ ] `bash scripts/verify_phase6_clean_clone.sh <clean-clone-path>` passes
- [ ] SSH alias recorded from `docs/cortex/eudaemon-alpha-ssh-config.example`
- [ ] Branch protection follow-up recorded if unavailable on current GitHub plan

## Phase 1: Hetzner Bring-Up

- [ ] SSH login succeeds to the Hetzner host
- [ ] Root repo cloned on-box at `/srv/nostra/eudaemon-alpha/repo`
- [ ] `cortex-gateway.service` installed and enabled
- [ ] companion bootstrap script completed
- [ ] `/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env` populated with production-safe values

## Phase 2: Governance Bootstrap and First Live Cycle

- [ ] `bootstrap-governance` executed for the target Space
- [ ] actor registry contains `agent:eudaemon-alpha-01`
- [ ] target Space exists with the correct `members` and `archetype`
- [ ] gateway accepts the registered agent with enforcement enabled
- [ ] `cortex-gateway.service` starts cleanly
- [ ] `eudaemon-alpha-agent.service` starts cleanly
- [ ] solicitation block is discoverable
- [ ] worker emits a `ConfigProposalBlock`
- [ ] memory persists under `/srv/nostra/eudaemon-alpha/state/agent-memory`

## Phase 3: Post-Go-Live Hardening

- [ ] Heap-driven config refresh is live in production
- [ ] agent activity notification panel is visible in `cortex-web`
- [ ] A2UI feedback projection is bound into the next-cycle context flow
- [ ] chronicle promotion path is defined after local draft validation

## Phase 4: Temporal and Migration Readiness

- [ ] Temporal validation is tracked as a separate post-go-live phase
- [ ] Rust-native migration gate is defined against parity and Hetzner hosted validation
