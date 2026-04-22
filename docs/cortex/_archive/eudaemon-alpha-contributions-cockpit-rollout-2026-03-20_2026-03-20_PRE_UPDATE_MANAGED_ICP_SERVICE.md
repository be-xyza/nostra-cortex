# Eudaemon Alpha Contributions Cockpit Rollout

Date: 2026-03-20
Target: `root@204.168.175.150`
Space: `01KM4C04QY37V9RV9H2HH9J1NM`

## Scope

This rollout promoted the `/contributions` hardening slice to the live Eudaemon Alpha VPS. The goal was to verify that the steward-facing contributions cockpit renders from the live gateway, that canonical `icp-cli` status fields are exposed, and that the read-only cockpit smoke contract passes on-host before any mutation-path validation.

## Rollout Actions

1. Synced the updated gateway, contracts, web host, smoke verifier, and operator docs into `/srv/nostra/eudaemon-alpha/repo/`.
2. Installed a current Rust toolchain via `rustup` under `root` because the host `cargo 1.75.0` toolchain was too old for the current workspace.
3. Built the live gateway binary directly on-host with:

```bash
cd /srv/nostra/eudaemon-alpha/repo/cortex
~/.cargo/bin/cargo build --release -p cortex-gateway
```

4. Restarted `cortex-gateway.service`, which runs the prebuilt binary directly from:

```text
/srv/nostra/eudaemon-alpha/repo/cortex/target/release/cortex-gateway
```

5. Verified the live gateway state, the new contributions cockpit surface, and the read-only smoke contract against `http://127.0.0.1:3000`.

## Live Deployment Evidence

- Gateway restart timestamp: `Fri 2026-03-20 09:30:46 UTC`
- `systemctl` main PID: `47512`
- Gateway build payload:

```json
{"buildId":"unknown","buildTimeUtc":"unknown","gatewayDispatchMode":"in_process","gatewayPort":3000,"workspaceRoot":"/srv/nostra/eudaemon-alpha/state/cortex-workspace"}
```

- Readiness payload:

```json
{"ready":false,"gateway_port":3000,"icp_network_healthy":false,"dfx_port_healthy":false,"notes":["Local replica TCP probe failed on port 4943"]}
```

- Status payload:

```json
{"icp_cli_running":false,"dfx_running":false,"version":"Unknown","replica_port":4943}
```

- Read-only smoke result:

```text
PASS: contributions cockpit smoke
  space_id=01KM4C04QY37V9RV9H2HH9J1NM ready=False icp_network_healthy=False icp_cli_running=False
  surface_id=viewspec:workbench-contributions agent_runs=auth_locked graph_runs=0
  build=unknown dispatch=in_process
```

- Browser evidence screenshot:
  - [/Users/xaoj/ICP/logs/cortex/contributions-cockpit-vps-2026-03-20.png](/Users/xaoj/ICP/logs/cortex/contributions-cockpit-vps-2026-03-20.png)

## What Passed

- `/api/system/ready` now exposes canonical `icp_network_healthy` while preserving `dfx_port_healthy`.
- `/api/system/status` now exposes canonical `icp_cli_running` while preserving `dfx_running`.
- `/api/system/ux/workbench?route=/contributions&space_id=01KM4C04QY37V9RV9H2HH9J1NM` returns the real contributions cockpit surface instead of the old route-level placeholder.
- The on-host read-only smoke verifier passed and correctly treated `/api/system/agents/runs` as `auth_locked` under the current verified-principal posture.
- Browser validation against the live gateway confirms the steward-facing host is reachable and shows the new focus, steward tools, and live execution layout.

## Follow-Up Recovery On March 20, 2026

After the first rollout note, the live host still had three concrete runtime gaps:

1. The gateway binary was still serving the stale placeholder `Contributions View` surface on direct `/api/system/ux/workbench` requests.
2. The managed Space path for `01KM4C04QY37V9RV9H2HH9J1NM` did not exist, so blast-radius lookups failed because no managed graph artifact was present.
3. The decision-surface agent run directory did not exist, so verified reads returned `503` instead of a clean empty list.

Those were corrected in a second pass:

- Synced the updated [workbench_ux.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/workbench_ux.rs) back to the VPS and rebuilt `cortex-gateway`.
- Restarted `cortex-gateway.service` at `Fri 2026-03-20 15:50:23 UTC`.
- Seeded `research/000-contribution-graph` into:
  - `/srv/nostra/eudaemon-alpha/repo/research/000-contribution-graph`
  - `/srv/nostra/eudaemon-alpha/state/cortex-workspace/_spaces/01KM4C04QY37V9RV9H2HH9J1NM/research/000-contribution-graph`
- Created the missing decision-surface log directory:
  - `/srv/nostra/eudaemon-alpha/state/cortex-workspace/logs/system/decision_surfaces/agent_runs`

Post-recovery live checks:

- Direct workbench surface now resolves correctly:

```json
{
  "title": "Steward-facing contribution lifecycle cockpit with graph history and live agent drill-ins.",
  "surfaceId": "viewspec:workbench-contributions",
  "texts": ["Contributions Cockpit"]
}
```

- Verified operator identity now works when the host is sent the configured bound principal:

```json
{
  "principal": "2vxsx-fae",
  "effectiveRole": "operator",
  "identityVerified": true,
  "identitySource": "principal_binding"
}
```

- Verified agent-run list now returns a clean empty list instead of `503`:

```json
[]
```

- Blast radius now returns `200 OK` for `proposal-alpha`:

```json
{
  "contributionId": "proposal-alpha",
  "dependsOn": [],
  "dependedBy": [],
  "invalidates": [],
  "invalidatedBy": [],
  "supersedes": [],
  "supersededBy": [],
  "references": [],
  "referencedBy": []
}
```

## Steward Auth and IC Tool Lane Follow-Up On March 20, 2026

This follow-up verified the live auth posture, restored the canonical `icp-cli` toolchain, and re-ran smoke plus browser/operator validation against the loopback-only gateway without weakening auth.

### Live Auth Posture at `2026-03-20T16:01:13Z`

Verified on-host service wiring:

```text
host=eudaemon-alpha-01
MainPID=54214
ExecStart=/srv/nostra/eudaemon-alpha/repo/cortex/target/release/cortex-gateway
EnvironmentFile=/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env
service_start=Fri 2026-03-20 15:50:23 UTC
```

Verified auth/config posture from the environment file and the running process:

```text
NOSTRA_AUTHZ_DEV_MODE=0
NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
NOSTRA_DECISION_PRINCIPAL_ROLE_BINDINGS={"2vxsx-fae":"operator"}
NOSTRA_IC_PRINCIPAL=2vxsx-fae
```

Important findings:

- No `NOSTRA_AUTHZ_PRINCIPAL_CLAIM_BINDINGS` were configured.
- No authz signing secret was configured for principal claim elevation.
- The actor registry only contains `agent:eudaemon-alpha-01` with role `operator`.
- No host-side identity PEM or steward principal artifact was present under `/srv/nostra/eudaemon-alpha`.

Result: there was no legitimate steward-bound principal path available on the VPS. The safest next change is to add a separate real steward principal to `NOSTRA_DECISION_PRINCIPAL_ROLE_BINDINGS` after the steward provides and proves control of that principal. The existing operator principal `2vxsx-fae` should not be silently repurposed to `steward`.

### Steward Packet Export Validation at `2026-03-20T16:06:02Z`

Live verified POST used:

```bash
curl -si -X POST \
  http://127.0.0.1:3000/api/kg/spaces/01KM4C04QY37V9RV9H2HH9J1NM/contribution-graph/steward-packet/export \
  -H "content-type: application/json" \
  -H "x-ic-principal: 2vxsx-fae" \
  -H "x-cortex-role: operator" \
  --data-binary @- <<'JSON'
{"goal":"stable-cortex-domain","approval":{"approvedBy":"systems-steward","approvedAt":"2026-03-20T16:00:00Z","rationale":"live validation probe","decisionRef":"decision:live-probe-2026-03-20T16:00:00Z"}}
JSON
```

Observed live response:

```json
{
  "details": {
    "authorization": {
      "action": "admin",
      "allowed": false,
      "engine": "legacy",
      "principal": "2vxsx-fae",
      "reason": "role 'operator' below required 'steward'",
      "requiredClaims": [],
      "requiredRole": "steward",
      "resource": "capability:dpub_steward_packet_export",
      "spaceId": "01KM4C04QY37V9RV9H2HH9J1NM"
    },
    "endpoint": "post_contribution_graph_steward_packet_export",
    "identitySource": "principal_binding",
    "identityVerified": true,
    "principal": "2vxsx-fae",
    "role": "operator"
  },
  "error": "Steward role is required for steward packet export.",
  "errorCode": "DPUB_PACKET_STEWARD_REQUIRED"
}
```

This confirms the live gateway is correctly enforcing steward-only packet export under the current production-safe auth posture.

### Canonical `icp-cli` Restoration Evidence

Expected current install path was confirmed from the official `dfinity/icp-cli` Quick Start:

- `npm install -g @icp-sdk/icp-cli @icp-sdk/ic-wasm`
- prerequisite: Node.js LTS (`>=18.0.0`)

Safe host-side install completed at `2026-03-20 16:07:43 UTC`:

```bash
apt-get update
apt-get install -y --no-install-recommends nodejs npm
npm install -g @icp-sdk/icp-cli@0.2.1 @icp-sdk/ic-wasm@0.9.10
```

Installed binaries:

```text
node v18.19.1
npm 9.2.0
icp 0.2.1
ic-wasm 0.9.10
```

Because `/srv/nostra/eudaemon-alpha/repo` still has no project `icp.yaml`, an isolated managed-network probe was created in:

```text
/srv/nostra/eudaemon-alpha/tmp/icp-replica-probe/icp.yaml
```

Probe config:

```yaml
networks:
  - name: local
    mode: managed
    gateway:
      host: 127.0.0.1
      port: 4943
```

Launch command:

```bash
cd /srv/nostra/eudaemon-alpha/tmp/icp-replica-probe
icp network start -d
```

Observed running process:

```text
Fri Mar 20 16:08:26 2026   60068 /root/.local/share/icp-cli/pkg/network-launcher/v12.0.0-2026-03-02-11-09.r1/pocket-ic --ttl 2592000 --port-file /tmp/.tmpr6KMZy/pocketic.port --log-levels error
```

Observed network status at `2026-03-20T16:14:05Z`:

```text
Api Url: http://localhost:4943/
Gateway Url: http://localhost:4943/
Root Key: 308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c050302010361008b52b4994f94c7ce4be1c1542d7c81dc79fea17d49efe8fa42e8566373581d4b969c4a59e96a0ef51b711fe5027ec01601182519d0a788f4bfe388e593b97cd1d7e44904de79422430bca686ac8c21305b3397b5ba4d7037d17877312fb7ee34
Candid UI Principal: tqzl2-p7777-77776-aaaaa-cai
Proxy Canister Principal: txyno-ch777-77776-aaaaq-cai
```

With that loopback-only network active, the gateway became healthy:

```json
{"ready":true,"gateway_port":3000,"icp_network_healthy":true,"dfx_port_healthy":true,"notes":[]}
```

```json
{"icp_cli_running":true,"dfx_running":true,"version":"icp 0.2.1","replica_port":4943}
```

Re-run smoke after the tool lane came up:

```text
PASS: contributions cockpit smoke
  space_id=01KM4C04QY37V9RV9H2HH9J1NM ready=True icp_network_healthy=True icp_cli_running=True
  surface_id=viewspec:workbench-contributions agent_runs=auth_locked graph_runs=0
  build=unknown dispatch=in_process
```

### Browser / Operator Validation

Operator browser validation used a local header-injecting bridge so the browser could hit the loopback-only live gateway with the verified bound principal without changing live auth flags:

1. SSH tunnel:

```bash
ssh -N -L 3300:127.0.0.1:3000 root@204.168.175.150
```

2. Local proxy added:
   - `x-ic-principal: 2vxsx-fae`

3. Local `cortex-web` dev host pointed to that proxy:

```bash
CORTEX_WEB_GATEWAY_URL=http://127.0.0.1:3301 \
VITE_CORTEX_GATEWAY_URL=http://127.0.0.1:3301 \
VITE_SPACE_ID=01KM4C04QY37V9RV9H2HH9J1NM \
VITE_ACTOR_ROLE=operator \
npm -C /Users/xaoj/ICP/cortex/apps/cortex-web run dev -- --host 127.0.0.1 --port 4174
```

Verified operator `whoami` through the browser bridge:

```json
{
  "principal": "2vxsx-fae",
  "requestedRole": "operator",
  "effectiveRole": "operator",
  "identityVerified": true,
  "identitySource": "principal_binding"
}
```

Observed browser evidence:

- The live `/contributions` route rendered the cockpit shell with `operator` as the active role.
- Focusing `proposal-alpha` updated the route to include:
  - `contribution_id=proposal-alpha`
  - `node_id=contribution:proposal-alpha`
- Blast radius displayed empty relation lanes exactly as the API reported.
- Exporting the steward packet surfaced the same live `403 DPUB_PACKET_STEWARD_REQUIRED` error in the UI.

Updated browser screenshot:

- [/Users/xaoj/ICP/logs/cortex/contributions-cockpit-vps-operator-2026-03-20T16-12-52Z.png](/Users/xaoj/ICP/logs/cortex/contributions-cockpit-vps-operator-2026-03-20T16-12-52Z.png)

### New Live Risk Discovered During Operator Validation

Focusing a contribution in the live browser session caused the `Live Execution` pane to auto-launch contribution runs through `A2UIRenderSpace` instead of only waiting for an explicit resume/select action.

Observed live outcome after shutting down the local browser bridge:

```text
verified operator agent-run count: 25
oldest startedAt: 2026-03-20T16:15:02.444950790+00:00
latest startedAt: 2026-03-20T16:15:17.016118294+00:00
status: waiting_approval
```

This appears to be driven by the frontend behavior that dispatches `startAgentContribution(...)` whenever `selectedContributionId` is present and no `selectedRunId` is active. In the temporary operator browser bridge used here, the websocket/spatial path was degraded, and that combination fanned out repeated launches instead of resuming a single governed run.

That fan-out should be treated as a real follow-up bug before future operator browser validation is repeated against production state.

## Residual Gaps

- Steward packet export remains correctly blocked because no steward-bound principal exists on the VPS.
- The current IC tool lane is functional but not yet durable:
  - the running `pocket-ic` process is loopback-only and safe enough for validation
  - it is still anchored to `/srv/nostra/eudaemon-alpha/tmp/icp-replica-probe`, not a repo-owned deployment manifest or managed service
- Graph-run history is still empty because no contribution-graph pipeline runs have been executed on-host since the graph seed was copied in.
- Read-only smoke still reports `agent_runs=auth_locked` by design because it does not send a verified principal.
- Operator browser validation uncovered a live repeat-launch risk in the `Live Execution` pane when contribution focus is used without an already-selected run.

## Recommended Follow-Up

1. Have the steward provide a separate real principal, append it to `NOSTRA_DECISION_PRINCIPAL_ROLE_BINDINGS`, and restart `cortex-gateway.service` before attempting end-to-end steward packet export.
2. Promote the scratch IC lane into a repo-owned and service-managed configuration so `icp_network_healthy=true` survives process exits and host reboots.
3. Fix the `A2UIRenderSpace` contribution-focus launch loop so browser validation resumes or deduplicates runs instead of spawning repeated `waiting_approval` executions.
4. Execute at least one controlled contribution-graph pipeline run on-host so graph-run history becomes non-empty and no longer relies only on the seeded baseline artifact.
