# System Rollback (Emergency)

> [!WARNING]
> This workflow restores the system to the previous consistent state.

- [ ] Identify Last Snapshot
> snapshot list --last

- [ ] Stop Running Canisters
> icp canister stop nostra_backend

- [ ] Restore Snapshot
> snapshot restore last local

- [ ] Restart Canisters
> icp canister start nostra_backend

- [ ] Verify System Health
> icp canister status nostra_backend
