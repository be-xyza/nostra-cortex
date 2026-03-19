# Eudaemon Alpha Phase 6 on Hetzner

This runbook is the canonical bring-up guide for the current Phase 6 deployment target:

- **Host**: Hetzner VPS
- **Gateway**: Rust `cortex-gateway`
- **Agent**: Python Eudaemon Alpha worker
- **Service model**: Linux `systemd`
- **Gateway scope**: loopback-only by default (`127.0.0.1:3000`)

Repository ownership for this stack is split:

- root ICP repo: governance docs, gateway assets, and root deployment guidance
- `eudaemon-alpha` submodule: Python worker, env templates, agent service unit, and agent bootstrap tooling

## Server Layout

Use one deployment root on the host:

```text
/srv/nostra/eudaemon-alpha/
├── config/
│   └── eudaemon-alpha.env
├── logs/
├── repo/
├── state/
│   ├── agent-memory/
│   ├── sandboxes/
│   └── cortex-workspace/
│       └── _spaces/
```

Recommended bindings:

- repo checkout: `/srv/nostra/eudaemon-alpha/repo`
- memory root: `/srv/nostra/eudaemon-alpha/state/agent-memory`
- sandbox root: `/srv/nostra/eudaemon-alpha/state/sandboxes`
- workspace root: `/srv/nostra/eudaemon-alpha/state/cortex-workspace`
- Python runtime: `/srv/nostra/eudaemon-alpha/.venv/bin/python`
- actor registry: `/srv/nostra/eudaemon-alpha/state/cortex-workspace/_spaces/actors.json`
- space registry: `/srv/nostra/eudaemon-alpha/state/cortex-workspace/_spaces/registry.json`

## Required Env Contract

Start from [`.env.hetzner.example`](/Users/xaoj/ICP/eudaemon-alpha/agent/.env.hetzner.example) and keep these values production-safe:

- `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
- `NOSTRA_GATEWAY_URL=http://127.0.0.1:3000/api/cortex/studio`
- `NOSTRA_AUTHZ_DEV_MODE=0`
- `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
- `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
- `NOSTRA_TEMPORAL_TARGET=127.0.0.1:7233`

## SSH Access

The expected local access pattern is a private key already present on the workstation plus a documented Hetzner host alias. Example:

```sshconfig
Host eudaemon-alpha-hetzner
  HostName <hetzner-ip-or-dns>
  User <server-user>
  IdentityFile ~/.ssh/id_ed25519
```

Do not leave the host identity implicit in local shell history or undocumented notes.

## Bring-Up Steps

1. Provision Ubuntu on Hetzner and confirm SSH access.
2. Clone the root repo with submodules under the deployment root:

```bash
git clone --recurse-submodules <root-repo-url> /srv/nostra/eudaemon-alpha/repo
```

3. Install the gateway service from the root repo assets:
   - [`cortex-gateway.service`](/Users/xaoj/ICP/ops/hetzner/systemd/cortex-gateway.service)
   - [`run_cortex_gateway_production.sh`](/Users/xaoj/ICP/scripts/run_cortex_gateway_production.sh)
4. Run the companion repo bootstrap script on the server as a privileged user:
   - [`bootstrap_eudaemon_alpha_hetzner.sh`](/Users/xaoj/ICP/eudaemon-alpha/scripts/bootstrap_eudaemon_alpha_hetzner.sh)
5. Fill in `/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env`.
6. Bootstrap governance state:
   - create or validate `agent:eudaemon-alpha-01` in the actor registry
   - create or validate the target Space entry in the space registry
   - ensure `members` includes the agent
   - ensure the target `archetype` is set
7. Enable and start:
   - `cortex-gateway.service`
   - `eudaemon-alpha-agent.service`
8. Validate the full loop:
   - gateway reachable on `127.0.0.1:3000`
   - identity enforcement enabled
   - solicitation block discoverable
   - `ConfigProposalBlock` emitted
   - local memory persisted

## Governance Bootstrap

Use the admin utility after sourcing the env file:

```bash
python3 /srv/nostra/eudaemon-alpha/repo/eudaemon-alpha/agent/src/admin.py \
  bootstrap-governance \
  --space-id <space-id> \
  --space-owner systems-steward \
  --space-archetype Research
```

This will:

- upsert `agent:eudaemon-alpha-01` in the actor registry
- create or update the Space entry in the space registry
- ensure the agent is a member
- ensure the intended archetype is recorded

## Quick Guide for the Next Setup Pass

After the server is bootstrapped, the remaining operational path is:

1. point local SSH to the Hetzner host alias
2. verify service health and auth posture
3. seed or create the target solicitation block
4. confirm the worker emits its `ConfigProposalBlock`
5. verify steward review and the next-cycle context ingestion path

## Gaps Still Outside This Bring-Up

- A2UI feedback binding remains a follow-up slice
- agent activity notification polish remains a follow-up slice
- chronicle promotion to Heap/DPub remains separate from the local draft path
- Rust-native worker migration remains a later parity-backed milestone
