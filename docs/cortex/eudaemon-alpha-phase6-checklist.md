# Eudaemon Alpha Phase 6 Bring-Up Checklist

Use this checklist for the active VPS runtime authority model.

## Phase 0: Repository and Documentation Authority

1. Confirm the host repo mirror exists at `/srv/nostra/eudaemon-alpha/repo`.
2. Confirm the active runbook is [`docs/cortex/eudaemon-alpha-phase6-hetzner.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md).
3. Confirm the authority checker exists:

```bash
test -x /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

4. Confirm the authority manifest exists:

```bash
test -f /srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json
```

## Phase 1: VPS Runtime Sync

1. Run the authority check before any further validation:

```bash
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

2. Require all of these to pass:
   - repo mirror is a git checkout
   - manifest commit matches repo `HEAD`
   - gateway unit `ExecStart` is inside `/srv/nostra/eudaemon-alpha/repo`
   - worker unit `ExecStart` is inside `/srv/nostra/eudaemon-alpha/repo`
   - `cortex-web` deployment mode is explicitly declared
   - primary authority docs exist

3. If the authority check fails, stop and repair sync before continuing.

## Phase 2: Runtime Health

1. Confirm services are active:

```bash
sudo systemctl status cortex-gateway.service --no-pager
sudo systemctl status cortex-worker.service --no-pager
```

2. Confirm gateway health:

```bash
curl -sS http://127.0.0.1:3000/api/system/ready
curl -sS http://127.0.0.1:3000/api/system/status
```

3. Confirm the gateway and worker binaries resolve under the repo mirror rather than `/usr/local/bin` or another detached path.

## Phase 3: Operator Validation

1. Use the live gateway as the VPS runtime surface.
2. Treat `cortex-web` as a separate client, not a deployed VPS service.
3. If browser validation is needed, run it from an operator-local or separately hosted `cortex-web` instance pointed at the live gateway.

## Completion Gate

The VPS is aligned for agent analysis only when:

- repo mirror is current
- authority manifest matches the mirrored commit
- gateway and worker units use repo-local binaries
- authority docs match the actual runtime boundary
- `cortex-web` remains explicitly `not_deployed`
