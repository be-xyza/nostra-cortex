# Initiative 121 Integrity Dependencies

This contract defines blocking integrity dependencies for Initiative 121 (`cortex-memory-fs`).

## Blocking Rule Groups

- `siq_governance_execution_contract`
- `siq_host_parity_contract`
- `siq_graph_projection_contract`

## Blocking Dependencies

- `118-cortex-runtime-extraction`
  - Required evidence: runtime governance execution closure and host-neutrality integrity controls.
- `123-cortex-web-architecture`
  - Required evidence: dual-host parity baseline and locked `cortex:a2ui:event` semantics.
- `103-agent-client-protocol-alignment`
  - Required evidence: ACP contract alignment artifacts remain valid for memory-fs tool/event usage.
- `105-cortex-test-catalog`
  - Required evidence: catalog/gate contract remains healthy for CI evidence lineage.

## Advancement Policy

Initiative 121 phase/milestone advancement is blocked when SIQ governance or host parity gates are failing.

The blocking condition is machine-enforced through SIQ artifacts and checks, not manual interpretation.
