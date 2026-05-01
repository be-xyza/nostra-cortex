#!/usr/bin/env bash
set -euo pipefail

# This script is executed on the VPS by an operator-initiated SSH promotion flow.

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
WORKROUTER_WORKDIR="$REPO_ROOT"
WORKROUTER_EXEC="$REPO_ROOT/scripts/work_router_service_stub.sh"
WORKER_KEYS_PATH="$STATE_ROOT/worker_keys.json"
OPERATIONS_INDEX="$REPO_ROOT/docs/cortex/README.md"
PRIMARY_RUNBOOK="$REPO_ROOT/docs/cortex/eudaemon-alpha-phase6-hetzner.md"
TARGET_COMMIT="${1:-}"

log() {
    echo "$1" | tee -a "$LOG"
}

resolve_target_commit() {
    git -C "$REPO_ROOT" fetch origin '+refs/heads/*:refs/remotes/origin/*'

    if [[ -z "$TARGET_COMMIT" ]]; then
        TARGET_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify "origin/main^{commit}")"
    elif ! git -C "$REPO_ROOT" cat-file -e "${TARGET_COMMIT}^{commit}" 2>/dev/null; then
        log " ! Commit does not exist on host mirror: $TARGET_COMMIT"
        exit 1
    else
        TARGET_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify "${TARGET_COMMIT}^{commit}")"
    fi
}

sync_repo_to_target() {
    git -C "$REPO_ROOT" checkout --detach "$TARGET_COMMIT"
    git -C "$REPO_ROOT" reset --hard "$TARGET_COMMIT"
}

mkdir -p "$DEPLOY_ROOT/logs"
mkdir -p "$STATE_ROOT"
sudo chown "$SERVICE_USER:$SERVICE_USER" "$DEPLOY_ROOT/logs"
sudo chown -R "$SERVICE_USER:$SERVICE_USER" "$STATE_ROOT"

render_systemd_unit() {
    local template_path="$1"
    local destination_path="$2"

    sed         -e "s|__DEPLOY_ROOT__|$DEPLOY_ROOT|g"         -e "s|__SERVICE_USER__|$SERVICE_USER|g"         "$template_path" | sudo tee "$destination_path" >/dev/null
}

render_worker_systemd_override() {
    sudo mkdir -p /etc/systemd/system/cortex-worker.service.d
    sudo tee /etc/systemd/system/cortex-worker.service.d/state.conf >/dev/null <<EOF
[Service]
Environment=NOSTRA_WORKER_KEYS_PATH=$WORKER_KEYS_PATH
EOF
}

redact_git_origin() {
    local origin="$1"

    if [[ "$origin" =~ ^https://[^/@]+@(.+)$ ]]; then
        printf 'https://REDACTED@%s\n' "${BASH_REMATCH[1]}"
    else
        printf '%s\n' "$origin"
    fi
}

write_authority_manifest() {
    local git_branch git_commit git_origin generated_at

    git_branch="$(git -C "$REPO_ROOT" symbolic-ref --short -q HEAD || echo detached)"
    git_commit="$(git -C "$REPO_ROOT" rev-parse HEAD)"
    git_origin="$(redact_git_origin "$(git -C "$REPO_ROOT" remote get-url origin)")"
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
    "workrouter": {
      "execPath": "$WORKROUTER_EXEC",
      "workingDirectory": "$WORKROUTER_WORKDIR",
      "mode": "observe",
      "maxDispatchLevel": "D1",
      "liveTransportEnabled": false
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

log "[$(date -u +%FT%TZ)] Deploy started"
resolve_target_commit
sync_repo_to_target

CARGO_BIN="$HOME/.cargo/bin/cargo"

if [[ ! -f "$CARGO_BIN" ]]; then
    log " ! Cargo not found at $CARGO_BIN. Please install via rustup."
    exit 1
fi

log "   > Rebuilding cortex-gateway from $GATEWAY_WORKDIR at $TARGET_COMMIT..."
(
    cd "$GATEWAY_WORKDIR"
    "$CARGO_BIN" build --release -p cortex-gateway
) 2>&1 | tee -a "$LOG"

log "   > Rebuilding cortex_worker from $WORKER_WORKDIR at $TARGET_COMMIT..."
(
    cd "$WORKER_WORKDIR"
    "$CARGO_BIN" build --release
) 2>&1 | tee -a "$LOG"

log "   > Rendering systemd units from repo templates..."
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-gateway.service" "/etc/systemd/system/cortex-gateway.service"
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-worker.service" "/etc/systemd/system/cortex-worker.service"
render_systemd_unit "$REPO_ROOT/ops/hetzner/systemd/cortex-workrouter.service" "/etc/systemd/system/cortex-workrouter.service"
render_worker_systemd_override
sudo systemctl daemon-reload

log "   > Restarting services..."
sudo systemctl restart cortex-gateway.service
sudo systemctl restart cortex-worker.service
sudo systemctl restart cortex-workrouter.service
sudo systemctl is-active --quiet cortex-gateway.service
sudo systemctl is-active --quiet cortex-worker.service
sudo systemctl is-active --quiet cortex-workrouter.service

if [[ "$(git -C "$REPO_ROOT" rev-parse HEAD)" != "$TARGET_COMMIT" ]]; then
    log " ! Host repo HEAD drifted away from target commit after service restart."
    exit 1
fi

log "   > Writing runtime authority manifest..."
write_authority_manifest

log "[$(date -u +%FT%TZ)] Deploy complete for $TARGET_COMMIT"
