# cortex-web

React/Vite host for the Cortex DPub Workbench.

## Purpose
- Active Cortex execution host (web surface) with parity goals against `cortex-desktop`.
- Consumes canonical runtime APIs from the host-neutral gateway surface.
- Does not implement independent graph/pathing logic.

## Run
```bash
/Users/xaoj/ICP/run_cortex_web dev
```

Optional gateway URL override:
```bash
VITE_CORTEX_GATEWAY_URL=http://127.0.0.1:3000 /Users/xaoj/ICP/run_cortex_web dev
```

## Operator Identity

The web host is viewer/read-only by default. Verified operator sessions are available only when a trusted local or private deployment explicitly enables Internet Identity operator auth.

```bash
VITE_II_OPERATOR_AUTH_ENABLED=true \
VITE_II_PROVIDER_URL=https://id.ai/authorize \
VITE_CORTEX_GATEWAY_URL=http://127.0.0.1:3000 \
/Users/xaoj/ICP/run_cortex_web dev
```

Convenience launcher:
```bash
/Users/xaoj/ICP/scripts/run_cortex_web_operator.sh
```

When enabled, read-only sessions show a compact `Sign in` shortcut beside the role/profile control and a `Verify operator` action inside the read-only details popover. The browser sends the Internet Identity principal and delegation proof to `POST /api/system/session/internet-identity`; the gateway must still verify the delegation and map the principal through role bindings before operator actions are granted.

Do not enable `VITE_II_OPERATOR_AUTH_ENABLED` on public/main Vercel unless that deployment is intended to accept verified operator sessions.

## A2UI Spatial Plane Experiment
- `VITE_A2UI_SPATIAL_PLANE=1`: enables `SpatialPlane` A2UI rendering in the web host.
- `VITE_A2UI_TLDRAW_EXPERIMENT=1`: attempts direct `tldraw` runtime binding. If `tldraw` is unavailable, renderer auto-falls back to SVG.

Example:
```bash
VITE_A2UI_SPATIAL_PLANE=1 VITE_A2UI_TLDRAW_EXPERIMENT=1 /Users/xaoj/ICP/run_cortex_web dev
```

## Heap Mode Quickstart (Desktop Source + Web Parity)

Start stack:
```bash
/Users/xaoj/ICP/run_cortex_heap_mode_stack up
```

Seed demo heap cards:
```bash
/Users/xaoj/ICP/run_cortex_heap_mode_stack seed
```

Open web host:
```text
http://127.0.0.1:5173
```

Optional: run desktop with heap adapter enabled for agent `RenderSurface` compatibility routing:
```bash
/Users/xaoj/ICP/run_cortex_heap_mode_stack desktop
```

Stop stack:
```bash
/Users/xaoj/ICP/run_cortex_heap_mode_stack down
```

### Evaluation Harness
The A2UI plane now supports `Linear`, `Spatial`, and `Compare` modes with on-screen metrics:
- total interaction events
- approval events
- button clicks
- spatial shape clicks
- adapter load/fallback counts
- time-to-first-interaction (ms)

## Heap Delta Polling (QA Toggle)
- Default behavior: heap delta polling is off unless enabled.
- Env enable (highest priority): `VITE_HEAP_CHANGED_BLOCKS_POLLING_ENABLED=true`
- Env interval: `VITE_HEAP_CHANGED_BLOCKS_POLLING_INTERVAL_MS=15000`
- In dev builds, the heap header includes a `Delta Poll` control (toggle + interval + apply).
- Local QA override (used when env enable is not `true`):
  - `localStorage["cortex.heap.deltaPolling"] = "1"`
  - `localStorage["cortex.heap.deltaPollingIntervalMs"] = "<ms>"`

Quick enable in browser console:
```js
localStorage.setItem("cortex.heap.deltaPolling", "1");
localStorage.setItem("cortex.heap.deltaPollingIntervalMs", "1000");
location.reload();
```

Disable and reset:
```js
localStorage.removeItem("cortex.heap.deltaPolling");
localStorage.removeItem("cortex.heap.deltaPollingIntervalMs");
location.reload();
```

Parity validation:
```bash
npm -C /Users/xaoj/ICP/cortex/apps/cortex-web run -s test:heap-parity
```
