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

## Residual Gaps

- The host remains degraded because the local IC probe is unhealthy:
  - `ready=false`
  - `icp_network_healthy=false`
  - `icp_cli_running=false`
- The VPS currently has no `icp`, `dfx`, `node`, or `npm` binary installed for the gateway userland, so there is no quick local path to restore the replica lane from the current host state without a broader tooling install slice.
- The managed Space now has a seeded contribution graph artifact, but it is only a baseline import. Live graph-run history is still empty because no contribution-graph pipeline runs have been executed on the VPS since the seed was copied in.
- Read-only smoke still reports `agent_runs=auth_locked` because it intentionally does not send a bound principal. Verified operator requests now work, but the smoke contract remains conservative by default.
- Steward packet export is still blocked because the only configured bound principal is `operator`, not `steward`:
  - live verified POST attempts now return `403 DPUB_PACKET_STEWARD_REQUIRED`
- Browser evidence shows one remaining UI integration gap inside the new host:
  - the embedded backend `Contributions View` summary surface still shows its own `under construction` copy even though the route-level cockpit host is now live
  - this should be cleaned up in the next slice so the inner summary surface matches the new steward host instead of reintroducing placeholder messaging

## Recommended Follow-Up

1. Restore or generate the contribution graph artifact for Space `01KM4C04QY37V9RV9H2HH9J1NM` so blast radius and graph-run data become live.
2. Validate agent-run detail and steward packet export with a verified principal in a controlled operator session.
3. Remove the remaining `under construction` copy from the embedded backend `Contributions View` surface so the browser experience is fully aligned with the completed cockpit host.
