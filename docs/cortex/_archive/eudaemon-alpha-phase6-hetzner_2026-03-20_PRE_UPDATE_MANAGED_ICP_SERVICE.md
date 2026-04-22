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

Supporting operator assets:

- bring-up checklist: [`eudaemon-alpha-phase6-checklist.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-checklist.md)
- SSH alias example: [`eudaemon-alpha-ssh-config.example`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-ssh-config.example)
- clean clone verifier: [`verify_phase6_clean_clone.sh`](/Users/xaoj/ICP/scripts/verify_phase6_clean_clone.sh)
- contributions cockpit smoke verifier: [`check_contributions_cockpit_smoke.sh`](/Users/xaoj/ICP/scripts/check_contributions_cockpit_smoke.sh)

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
- `NOSTRA_LLM_PROVIDER_KIND=api_key` for the default Hetzner live lane
- `NOSTRA_LLM_AUTH_MODE=api_key` unless an explicit subscription sidecar is adopted
- `NOSTRA_LLM_BASE_URL` must be set for the active live lane
- `NOSTRA_CODEX_SIDECAR_URL` and `NOSTRA_LLM_AUTH_PROFILE` are required only when `NOSTRA_LLM_PROVIDER_KIND=codex_subscription`
- `NOSTRA_BATCH_AUDIT_PROVIDER_KIND=doubleword_batch` remains advisory and does not gate first bring-up
- `NOSTRA_BATCH_AUDIT_BASE_URL`, `NOSTRA_BATCH_AUDIT_API_KEY`, and `NOSTRA_BATCH_AUDIT_MODEL` are optional and only needed when the advisory batch lane is explicitly enabled

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
   - Default the live cognition lane to `api_key`.
   - Only switch to `codex_subscription` when a sidecar/profile path is already provisioned and validated.
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
   - runtime preflight passes for the selected live cognition lane
   - solicitation block discoverable
   - `ConfigProposalBlock` emitted
   - local memory persisted
9. Treat live Temporal as a later validation phase, not as part of the first hosted pass.

## Contributions Cockpit Release Slice

Use this sequence when shipping the `/contributions` cockpit hardening slice to Hetzner:

1. Update only the targeted repo files on the host.
2. Restart `cortex-gateway.service` after the repo update lands.
3. Run the read-only smoke verifier before any steward mutation checks:

```bash
cd /srv/nostra/eudaemon-alpha/repo
bash scripts/check_contributions_cockpit_smoke.sh http://127.0.0.1:3000 <space-id>
```

4. Treat these smoke outcomes differently:
   - `ready=true` and `icp_network_healthy=false`: gateway is up, but the local IC tool lane is unhealthy. Fix the replica or `icp-cli` environment before trusting contribution operations.
   - smoke failure on `/api/system/ux/workbench?route=/contributions`: the cockpit surface is unavailable or has regressed to placeholder behavior. Fix the gateway/runtime slice before steward validation.
   - smoke failure on agent runs or graph runs: contribution data is unavailable even if the gateway is healthy. Check space selection, stored run artifacts, and contribution-graph state before release signoff.
5. After smoke passes, validate operator behavior through a browser session:
   - `/contributions` renders the real cockpit
   - graph runs and agent runs load
   - selecting an agent run shows the live approval surface
   - selecting a contribution shows blast radius
   - steward packet export works for a known contribution in a controlled session

Because the Hetzner host currently runs the gateway and agent only, treat browser validation as an operator-side session against the live gateway rather than as a separate `cortex-web` service restart.

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

Use the checklist in [`eudaemon-alpha-phase6-checklist.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-checklist.md) as the source of truth for completion gates.

## Gaps Still Outside This Bring-Up

- chronicle promotion to Heap/DPub remains separate from the local draft path
- Codex subscription sidecar validation remains a separate follow-up if that lane is adopted
- advisory batch audit adapter work remains separate from the first live deployment lane
- Rust-native worker migration remains a later parity-backed milestone
