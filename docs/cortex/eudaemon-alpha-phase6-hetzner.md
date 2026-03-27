# Eudaemon Alpha Phase 6 on Hetzner

This is the canonical VPS runtime runbook for the current Eudaemon Alpha host.

## Runtime Authority Model

- Canonical analysis authority: `/srv/nostra/eudaemon-alpha/repo`
- Host-local authority manifest: `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`
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

## Deploy Contract

The VPS deploy path must:

1. Sync `/srv/nostra/eudaemon-alpha/repo` to the intended committed revision.
2. Build the gateway from `/srv/nostra/eudaemon-alpha/repo/cortex`.
3. Build the worker from `/srv/nostra/eudaemon-alpha/repo/nostra/worker`.
4. Render and install the systemd units from the repo templates.
5. Write `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`.
6. Restart the gateway and worker.

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
- declared working directories exist
- `cortex-web` deployment mode is explicit
- authority docs referenced by the manifest exist

If this check fails, the VPS agent must treat the host as out of sync and stop before analysis or runtime mutation work.

## Operator Validation

Use these follow-up checks after a successful authority verification:

```bash
curl -sS http://127.0.0.1:3000/api/system/ready
curl -sS http://127.0.0.1:3000/api/system/status
systemctl status cortex-gateway.service --no-pager
systemctl status cortex-worker.service --no-pager
```

For browser-side validation, use an operator-local or separately hosted `cortex-web` instance pointed at the live gateway. Do not treat browser behavior as VPS-hosted runtime unless the authority manifest and this runbook are updated to declare `cortex-web` as deployed.
