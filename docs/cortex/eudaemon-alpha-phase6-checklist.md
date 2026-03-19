# Eudaemon Alpha Phase 6 Bring-Up Checklist

This is the operational checklist for the remaining Phase 6 work after the repo-boundary and CI alignment merge.

## Phase 0: Stabilize the Operator Environment

1. Use a clean root clone rather than `/Users/xaoj/ICP` for delivery work.

```bash
git clone --recurse-submodules https://github.com/be-xyza/nostra-cortex.git /Users/xaoj/nostra-cortex-clean
bash /Users/xaoj/nostra-cortex-clean/scripts/verify_phase6_clean_clone.sh /Users/xaoj/nostra-cortex-clean
```

2. Confirm the companion repo pin resolves correctly.

```bash
git -C /Users/xaoj/nostra-cortex-clean submodule status
```

3. Record the Hetzner SSH alias locally using the example config in [`eudaemon-alpha-ssh-config.example`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-ssh-config.example).

4. Branch protection note:
   The current GitHub plan for this private repo does not expose branch-protection APIs. Treat this as a manual platform follow-up unless the repo becomes public or the account tier changes.

## Phase 1: Hetzner Bring-Up

1. Verify SSH login using the documented alias.

```bash
ssh eudaemon-alpha-hetzner
```

2. Clone the merged root repo on the host.

```bash
git clone --recurse-submodules https://github.com/be-xyza/nostra-cortex.git /srv/nostra/eudaemon-alpha/repo
```

3. Install the gateway service from the root repo.

```bash
sudo install -m 0644 /srv/nostra/eudaemon-alpha/repo/ops/hetzner/systemd/cortex-gateway.service /etc/systemd/system/cortex-gateway.service
sudo systemctl daemon-reload
sudo systemctl enable cortex-gateway.service
```

4. Run the companion bootstrap script.

```bash
cd /srv/nostra/eudaemon-alpha/repo/eudaemon-alpha
sudo ./scripts/bootstrap_eudaemon_alpha_hetzner.sh
```

5. Fill `/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env` from the Hetzner env template and keep the production auth flags unchanged.

## Phase 2: Governance Bootstrap and First Live Cycle

1. Run governance bootstrap before starting the worker.

```bash
export NOSTRA_EUDAEMON_ENV_FILE=/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env
/srv/nostra/eudaemon-alpha/.venv/bin/python /srv/nostra/eudaemon-alpha/repo/eudaemon-alpha/agent/src/admin.py \
  bootstrap-governance \
  --space-id <space-id> \
  --space-owner systems-steward \
  --space-archetype Research
```

2. Require all of these before declaring the deployment healthy:
   - actor registry contains `agent:eudaemon-alpha-01`
   - target Space exists
   - target Space `members` includes the agent
   - target Space `archetype` is set correctly
   - gateway accepts the agent with identity enforcement enabled

3. Start services in this order.

```bash
sudo systemctl start cortex-gateway.service
sudo systemctl start eudaemon-alpha-agent.service
```

4. Validate the first pass:
   - gateway responds on `127.0.0.1:3000`
   - dev auth is off
   - a solicitation block is discoverable
   - the worker packages context with `POST /api/cortex/studio/heap/blocks/context`
   - the worker emits a `ConfigProposalBlock`
   - memory persists under `/srv/nostra/eudaemon-alpha/state/agent-memory`

5. Keep Temporal out of this pass until the loop above is healthy.

## Phase 3: Post-Go-Live Hardening

1. Promote Heap-driven production config refresh:
   - `execution_strategy.refresh_from_heap()`
   - `resource_governor.refresh_from_heap()`
2. Ship the `cortex-web` agent activity notification panel.
3. Bind the A2UI feedback projection form into the next-cycle context path.
4. Keep local chronicle drafting, then add Heap-backed chronicle promotion as the next persistence milestone.
5. Revalidate the operator loop after each slice instead of batching multiple hardening changes into one release.

## Phase 4: Temporal and Migration Readiness

1. Add live Temporal only after the first hosted cycle is routine and stable.
2. Treat Temporal bring-up as a separate validation phase.
3. Keep Rust-native migration gated on:
   - feature parity with the Python worker
   - matching governance/bootstrap behavior
   - matching heap/context/emit contracts
   - passing the same Hetzner end-to-end validation path
