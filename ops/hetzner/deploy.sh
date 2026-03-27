#!/usr/bin/env bash
set -euo pipefail

# This script is executed on the VPS by GitHub Actions via SSH.

DEPLOY_ROOT="/srv/nostra/eudaemon-alpha"
LOG="$DEPLOY_ROOT/logs/deploy.log"
SERVICE_USER="${SERVICE_USER:-nostra}"
REPO_ROOT="$DEPLOY_ROOT/repo"
STATE_ROOT="$DEPLOY_ROOT/state"
AUTHORITY_MANIFEST="$STATE_ROOT/cortex_runtime_authority.json"
GATEWAY_WORKDIR="$REPO_ROOT/cortex"
WORKER_WORKDIR="$REPO_ROOT/nostra/worker"
GATEWAY_EXEC="$GATEWAY_WORKDIR/target/release/cortex-gateway"
WORKER_EXEC="$WORKER_WORKDIR/target/release/cortex_worker"
OPERATIONS_INDEX="$REPO_ROOT/docs/cortex/README.md"
PRIMARY_RUNBOOK="$REPO_ROOT/docs/cortex/eudaemon-alpha-phase6-hetzner.md"

# Ensure log directory exists
mkdir -p "$DEPLOY_ROOT/logs"
mkdir -p "$STATE_ROOT"

render_systemd_unit() {
    local template_path="$1"
    local destination_path="$2"

    sed \
        -e "s|__DEPLOY_ROOT__|$DEPLOY_ROOT|g" \
        -e "s|__SERVICE_USER__|$SERVICE_USER|g" \
        "$template_path" | sudo tee "$destination_path" >/dev/null
}

write_authority_manifest() {
    local git_branch git_commit git_origin generated_at

    git_branch="$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)"
    git_commit="$(git -C "$REPO_ROOT" rev-parse HEAD)"
    git_origin="$(git -C "$REPO_ROOT" remote get-url origin)"
    generated_at="$(date -u +%FT%TZ)"

    cat >"$AUTHORITY_MANIFEST" <<EOF
{
  "schemaVersion": "1.0.0",
  "generatedAt": "$generated_at",
  "deployRoot": "$DEPLOY_ROOT",
  "repoRoot": "$REPO_ROOT",
  "git": {
    "branch": "$git_branch",
    "commit": "$git_commit",
    "origin": "$git_origin"
  },
  "runtime": {
    "gateway": {
      "execPath": "$GATEWAY_EXEC",
      "workingDirectory": "$GATEWAY_WORKDIR"
    },
    "worker": {
      "execPath": "$WORKER_EXEC",
      "workingDirectory": "$WORKER_WORKDIR"
    },
    "cortexWeb": {
      "deploymentMode": "not_deployed",
      "sourceRoot": "$REPO_ROOT/cortex/apps/cortex-web"
    }
  },
  "authorityDocs": {
    "primaryRunbook": "$PRIMARY_RUNBOOK",
    "operationsIndex": "$OPERATIONS_INDEX"
  }
}
EOF
}

echo "[$(date -u +%FT%TZ)] Deploy started" | tee -a "$LOG"

# Sync with GitHub
cd "$REPO_ROOT"
git fetch origin main
git reset --hard origin/main

# Build Cortex components
# Using full path to cargo to avoid PATH issues in non-interactive SSH shells
CACO_BIN="$HOME/.cargo/bin/cargo"

if [ ! -f "$CACO_BIN" ]; then
    echo " ! Cargo not found at $CACO_BIN. Please install via rustup." | tee -a "$LOG"
    exit 1
fi

echo "   > Rebuilding cortex-gateway from $GATEWAY_WORKDIR..." | tee -a "$LOG"
(
    cd "$GATEWAY_WORKDIR"
    "$CACO_BIN" build --release -p cortex-gateway
) 2>&1 | tee -a "$LOG"

echo "   > Rebuilding cortex_worker from $WORKER_WORKDIR..." | tee -a "$LOG"
(
    cd "$WORKER_WORKDIR"
    "$CACO_BIN" build --release
) 2>&1 | tee -a "$LOG"

echo "   > Rendering systemd units from repo templates..." | tee -a "$LOG"
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-gateway.service" "/etc/systemd/system/cortex-gateway.service"
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-worker.service" "/etc/systemd/system/cortex-worker.service"
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-icp-network.service" "/etc/systemd/system/cortex-icp-network.service"
sudo systemctl daemon-reload

# Restart services
# Assumes sudoers is configured for the deploying user for systemctl
echo "   > Restarting services..." | tee -a "$LOG"
sudo systemctl restart cortex-gateway.service
sudo systemctl restart cortex-worker.service
sudo systemctl is-active --quiet cortex-gateway.service
sudo systemctl is-active --quiet cortex-worker.service

echo "   > Writing runtime authority manifest..." | tee -a "$LOG"
write_authority_manifest

echo "[$(date -u +%FT%TZ)] Deploy complete" | tee -a "$LOG"
