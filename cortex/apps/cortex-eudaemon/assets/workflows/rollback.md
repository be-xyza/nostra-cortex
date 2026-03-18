# System Rollback (Emergency)

> [!WARNING]
> This workflow restores the system to the previous consistent state.

- [ ] Identify Last Snapshot
> snapshot list --last

- [ ] Stop Running Canisters
> dfx canister stop nostra_backend

- [ ] Restore Snapshot
> snapshot restore last local

- [ ] Restart Canisters
> dfx canister start nostra_backend

- [ ] Verify System Health
> dfx canister status nostra_backend
