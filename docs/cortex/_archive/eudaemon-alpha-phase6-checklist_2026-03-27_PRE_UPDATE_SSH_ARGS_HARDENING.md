# Eudaemon Alpha Phase 6 Bring-Up Checklist

Use this checklist for the active VPS runtime authority model.

## Phase 0: Operator Authority and Documentation

1. Confirm the local promotion path exists:

```bash
test -x /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh
```

2. Confirm the active runbook is [`docs/cortex/eudaemon-alpha-phase6-hetzner.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md).
3. Confirm the local SSH alias is configured for `eudaemon-alpha-hetzner` using [`docs/cortex/eudaemon-alpha-ssh-config.example`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-ssh-config.example).
4. Confirm GitHub has reported the target `main` commit as promotable before operator promotion.

## Phase 1: Governed Promotion

1. Promote the latest promotable `main` commit or a specific known-good commit from the operator machine:

```bash
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh <commit-sha>
```

2. Require the promotion command to finish with all of these true:
   - the host repo mirror is a git checkout
   - the manifest exists and parses
   - manifest commit matches host repo `HEAD`
   - gateway and worker unit `ExecStart` values stay inside `/srv/nostra/eudaemon-alpha/repo`
   - running gateway and worker processes match the manifest executable paths
   - authority docs exist and match the runtime boundary
   - `cortex-web` deployment mode is explicitly `not_deployed`

3. If the promotion command fails, stop and repair authority drift before any smoke checks.

## Phase 2: VPS Runtime Sync

1. Run the authority check directly on the host after promotion or rollback:

```bash
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

2. If the authority check fails, stop and repair sync before continuing.
3. Confirm the deployed commit reported by the promotion command matches the intended commit.

## Phase 3: Runtime Health

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

## Phase 4: Rollback

1. Choose a prior known-good commit.
2. Run the same local promotion command with that commit.
3. Rerun the host authority check before any additional validation.

## Phase 5: Operator Validation

1. Use the live gateway as the VPS runtime surface.
2. Treat `cortex-web` as a separate client, not a deployed VPS service.
3. If browser validation is needed, run it from an operator-local or separately hosted `cortex-web` instance pointed at the live gateway.

## Completion Gate

The VPS is aligned for agent analysis only when:

- repo mirror is current for the chosen promoted commit
- authority manifest matches the mirrored commit
- gateway and worker units use repo-local binaries
- gateway and worker processes match the manifest executable paths
- authority docs match the actual runtime boundary
- `cortex-web` remains explicitly `not_deployed`
