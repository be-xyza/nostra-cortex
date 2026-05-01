# Eudaemon Alpha Phase 6 on Hetzner

This is the canonical VPS runtime runbook for the current Eudaemon Alpha host.

## Deployment Authority Model

- Source authority: committed `main`
- GitHub authority: validates promotability only; it does not mutate the VPS
- Deploy authority: operator-initiated SSH promotion from a governed machine
- Canonical analysis authority: `/srv/nostra/eudaemon-alpha/repo`
- Host-local runtime authority manifest: `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`
- Deployed runtime components:
  - `cortex-gateway`
  - `cortex_worker`
- Not deployed on the VPS:
  - `cortex-web`

`cortex-web` source remains in the mirrored repo for development and reference, but it is not part of live VPS runtime behavior. VPS analysis must start from the repo mirror plus the authority manifest, then compare the running services against those declared paths.

## Server Layout

```text
/srv/nostra/eudaemon-alpha/
├── config/
│   └── eudaemon-alpha.env
├── logs/
├── repo/
├── state/
│   ├── cortex_runtime_authority.json
│   └── cortex-workspace/
└── tmp/
```

Canonical roots:

- repo mirror: `/srv/nostra/eudaemon-alpha/repo`
- authority manifest: `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`
- gateway workspace root: `/srv/nostra/eudaemon-alpha/repo/cortex`
- worker workspace root: `/srv/nostra/eudaemon-alpha/repo/nostra/worker`
- `cortex-web` source root: `/srv/nostra/eudaemon-alpha/repo/cortex/apps/cortex-web`

## Runtime Components

### Gateway

- systemd unit: `cortex-gateway.service`
- working directory: `/srv/nostra/eudaemon-alpha/repo/cortex`
- executable: `/srv/nostra/eudaemon-alpha/repo/cortex/target/release/cortex-gateway`

### Worker

- systemd unit: `cortex-worker.service`
- working directory: `/srv/nostra/eudaemon-alpha/repo/nostra/worker`
- executable: `/srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker`

### `cortex-web`

- deployment mode: `not_deployed`
- source path: `/srv/nostra/eudaemon-alpha/repo/cortex/apps/cortex-web`
- operator use: local or separately hosted client against the live gateway

## Promotion Contract

The only supported production promotion path is the operator-local command [`scripts/promote_eudaemon_alpha_vps.sh`](/Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh). It is operator-initiated over SSH and must:

1. Resolve a target commit from `origin/main`.
2. SSH to the VPS using local operator SSH config.
3. Run [`ops/hetzner/deploy.sh`](/Users/xaoj/ICP/ops/hetzner/deploy.sh) on-host with that exact commit.
4. Run [`scripts/check_vps_runtime_authority.sh`](/Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh) on-host before any smoke validation.
5. Fail the promotion if repo, manifest, units, or running binaries drift from the intended commit.

The default operator expectation is a stable local SSH alias for `eudaemon-alpha-hetzner`. If an operator needs a temporary direct host token, SSH config file, or non-default port, they may set `NOSTRA_EUDAEMON_VPS_HOST` and `NOSTRA_EUDAEMON_VPS_SSH_ARGS` for that invocation without introducing a second deployment authority surface.

The on-host deploy path must:

1. Fetch `origin`.
2. Verify the chosen commit exists.
3. Checkout/reset the mirror to that exact committed revision.
4. Build the gateway from `/srv/nostra/eudaemon-alpha/repo/cortex`.
5. Build the worker from `/srv/nostra/eudaemon-alpha/repo/nostra/worker`.
6. Render and install the systemd units from repo templates.
7. Restart the gateway and worker.
8. Write `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`.

The authority manifest is the first source of truth for the VPS agent. Running services must be treated as derived state that is validated against that manifest, not as authority by themselves.

## Agent Loop Contract

The first command in the VPS agent development/testing loop is:

```bash
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

This check must confirm:

- the repo mirror is a valid git checkout
- the authority manifest exists and parses
- manifest commit matches repo `HEAD`
- installed gateway and worker units run binaries under the repo mirror
- running gateway and worker processes match the manifest executable paths
- declared working directories exist
- authority docs referenced by the manifest exist and match the runtime boundary
- `cortex-web` deployment mode is explicitly `not_deployed`

If this check fails, the VPS agent must treat the host as out of sync and stop before analysis or runtime mutation work.

## Operator Runbook

Promote the latest promotable `main` commit:

```bash
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh
```

Promote a specific known-good commit:

```bash
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh <commit-sha>
```

Promote with an explicit host token and temporary SSH options:

```bash
NOSTRA_EUDAEMON_VPS_HOST=root@203.0.113.10 \
NOSTRA_EUDAEMON_VPS_SSH_ARGS='-F /tmp/eudaemon-alpha-ssh.conf -p 2222' \
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh <commit-sha>
```

Rollback uses the same governed path:

```bash
bash /Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh <previous-known-good-commit>
```

After promotion or rollback, rerun the authority check and then continue to smoke validation.

## Operator Validation

Use these follow-up checks after a successful authority verification:

```bash
curl -sS http://127.0.0.1:3000/api/system/ready
curl -sS http://127.0.0.1:3000/api/system/status
systemctl status cortex-gateway.service --no-pager
systemctl status cortex-worker.service --no-pager
```

For browser-side validation, use an operator-local or separately hosted `cortex-web` instance pointed at the live gateway. Do not treat browser behavior as VPS-hosted runtime unless the authority manifest and this runbook are updated to declare `cortex-web` as deployed.
