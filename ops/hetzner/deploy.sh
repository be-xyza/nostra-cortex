#!/usr/bin/env bash
set -euo pipefail

# This script is executed on the VPS by GitHub Actions via SSH.

DEPLOY_ROOT="/srv/nostra/eudaemon-alpha"
LOG="$DEPLOY_ROOT/logs/deploy.log"

# Ensure log directory exists
mkdir -p "$DEPLOY_ROOT/logs"

echo "[$(date -u +%FT%TZ)] Deploy started" | tee -a "$LOG"

# Sync with GitHub
cd "$DEPLOY_ROOT/repo"
git fetch origin main
git reset --hard origin/main

# Build Cortex components
# Using full path to cargo to avoid PATH issues in non-interactive SSH shells
CACO_BIN="$HOME/.cargo/bin/cargo"

if [ ! -f "$CACO_BIN" ]; then
    echo " ! Cargo not found at $CACO_BIN. Please install via rustup." | tee -a "$LOG"
    exit 1
fi

echo "   > Rebuilding cortex-gateway and cortex_worker..." | tee -a "$LOG"
"$CACO_BIN" build --release -p cortex-gateway -p cortex_worker 2>&1 | tee -a "$LOG"

# Restart services
# Assumes sudoers is configured for the deploying user for systemctl
echo "   > Restarting services..." | tee -a "$LOG"
sudo systemctl restart cortex-gateway.service
sudo systemctl restart cortex-worker.service

echo "[$(date -u +%FT%TZ)] Deploy complete" | tee -a "$LOG"
