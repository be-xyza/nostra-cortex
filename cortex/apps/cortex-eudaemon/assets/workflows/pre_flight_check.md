# Pre-Flight Check (System Upgrade)

> [!IMPORTANT]
> This workflow validates the system state before applying an update.

- [ ] Check Disk Availability (Requires > 1GB)
> check_disk_space 1024

- [ ] Check Replica Connection
> dfx ping

- [ ] Create Safety Snapshot
> snapshot create nostra_backend local

- [ ] Verify Snapshot Integrity
> snapshot verify last
