# Cortex Documentation Entry

Use this index for Cortex execution-layer and runtime behavior.

## Active Authority Model

GitHub validates whether `main` is promotable, but production VPS mutation is operator-initiated over SSH. The authoritative host analysis surfaces are the repo mirror at `/srv/nostra/eudaemon-alpha/repo` and the runtime manifest at `/srv/nostra/eudaemon-alpha/state/cortex_runtime_authority.json`. The canonical promotion command is [`scripts/promote_eudaemon_alpha_vps.sh`](/Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh).

## Start Here

- [`docs/cortex/eudaemon-alpha-phase6-hetzner.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md)
- [`docs/cortex/eudaemon-alpha-phase6-checklist.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-checklist.md)
- [`docs/cortex/eudaemon-alpha-ssh-config.example`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-ssh-config.example)
- [`docs/architecture/nostra-cortex-boundary.md`](/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md)
- [`research/013-nostra-workflow-engine/`](/Users/xaoj/ICP/research/013-nostra-workflow-engine)

## Focus Areas

- workers, agents, and execution lifecycle
- durable workflows and outbox patterns
- runtime observability and operator surfaces
- VPS authority, promotion, and sync integrity
