#!/usr/bin/env bash
set -euo pipefail

MODE="host"
if [[ "${1:-}" == "--repo-contract" ]]; then
  MODE="repo-contract"
  shift
fi

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
DEPLOY_ROOT="${NOSTRA_VPS_DEPLOY_ROOT:-/srv/nostra/eudaemon-alpha}"
REPO_ROOT="${NOSTRA_VPS_REPO_ROOT:-$DEPLOY_ROOT/repo}"
STATE_ROOT="${NOSTRA_VPS_STATE_ROOT:-$DEPLOY_ROOT/state}"
MANIFEST_PATH="${NOSTRA_VPS_AUTHORITY_MANIFEST:-$STATE_ROOT/cortex_runtime_authority.json}"
SYSTEMD_ROOT="${NOSTRA_VPS_SYSTEMD_ROOT:-/etc/systemd/system}"
SKIP_PROCESS_PROVENANCE="${NOSTRA_VPS_SKIP_PROCESS_PROVENANCE:-0}"

REPO_TEMPLATE_GATEWAY="$ROOT_DIR/ops/hetzner/systemd/cortex-gateway.service"
REPO_TEMPLATE_WORKER="$ROOT_DIR/ops/hetzner/systemd/cortex-worker.service"
DEPLOY_SCRIPT="$ROOT_DIR/ops/hetzner/deploy.sh"
PROMOTION_SCRIPT="$ROOT_DIR/scripts/promote_eudaemon_alpha_vps.sh"
DOCS_INDEX="$ROOT_DIR/docs/cortex/README.md"
PRIMARY_RUNBOOK="$ROOT_DIR/docs/cortex/eudaemon-alpha-phase6-hetzner.md"
CHECKLIST_DOC="$ROOT_DIR/docs/cortex/eudaemon-alpha-phase6-checklist.md"
SCHEMA_PATH="$ROOT_DIR/shared/standards/cortex_runtime_authority.schema.json"

errors=()
checks=()

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  printf '%s' "$value"
}

push_check() {
  checks+=("$1")
}

push_error() {
  errors+=("$1")
}

extract_json_string() {
  local key="$1"
  local path="$2"
  local index="${3:-1}"
  local match
  match="$(grep -Eo "\"${key}\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$path" | sed -n "${index}p" || true)"
  if [[ -z "$match" ]]; then
    return 1
  fi
  printf '%s\n' "$match" | sed -E "s/\"${key}\"[[:space:]]*:[[:space:]]*\"([^\"]*)\"/\\1/"
}

unit_value() {
  local key="$1"
  local path="$2"
  grep -E "^${key}=" "$path" | tail -n 1 | cut -d= -f2-
}

rendered_under_repo() {
  local value="$1"
  local repo_prefix="$2"
  [[ "$value" == "$repo_prefix"* ]]
}

check_file_exists() {
  local label="$1"
  local path="$2"
  if [[ -f "$path" ]]; then
    push_check "$label:$path"
  else
    push_error "$label missing: $path"
  fi
}

check_dir_exists() {
  local label="$1"
  local path="$2"
  if [[ -d "$path" ]]; then
    push_check "$label:$path"
  else
    push_error "$label missing: $path"
  fi
}

check_file_contains() {
  local label="$1"
  local path="$2"
  local pattern="$3"
  if grep -Fq "$pattern" "$path"; then
    push_check "$label:$pattern"
  else
    push_error "$label missing pattern '$pattern' in $path"
  fi
}

check_json_parse() {
  local label="$1"
  local path="$2"
  if python3 - "$path" <<'PY' >/dev/null 2>&1
import json
import pathlib
import sys

json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
PY
  then
    push_check "$label:$path"
  else
    push_error "$label failed to parse: $path"
  fi
}

check_repo_contract() {
  check_file_exists "deploy_script" "$DEPLOY_SCRIPT"
  check_file_exists "promotion_script" "$PROMOTION_SCRIPT"
  check_file_exists "gateway_template" "$REPO_TEMPLATE_GATEWAY"
  check_file_exists "worker_template" "$REPO_TEMPLATE_WORKER"
  check_file_exists "docs_index" "$DOCS_INDEX"
  check_file_exists "primary_runbook" "$PRIMARY_RUNBOOK"
  check_file_exists "checklist_doc" "$CHECKLIST_DOC"
  check_file_exists "manifest_schema" "$SCHEMA_PATH"

  if grep -Fq 'TARGET_COMMIT="${1:-}"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_target_commit_input:present"
  else
    push_error "deploy script missing explicit commit input"
  fi

  if grep -Fq 'git -C "$REPO_ROOT" fetch origin main' "$DEPLOY_SCRIPT"; then
    push_check "deploy_fetch_origin_main:present"
  else
    push_error "deploy script missing origin/main fetch"
  fi

  if grep -Fq 'cat-file -e "${TARGET_COMMIT}^{commit}"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_commit_verification:present"
  else
    push_error "deploy script missing explicit commit verification"
  fi

  if grep -Fq 'git -C "$REPO_ROOT" checkout --detach "$TARGET_COMMIT"' "$DEPLOY_SCRIPT" && \
     grep -Fq 'git -C "$REPO_ROOT" reset --hard "$TARGET_COMMIT"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_exact_checkout:present"
  else
    push_error "deploy script missing exact target checkout/reset behavior"
  fi

  if grep -Fq 'git reset --hard origin/main' "$DEPLOY_SCRIPT"; then
    push_error "deploy script still hard-resets directly to origin/main"
  else
    push_check "deploy_hidden_origin_main_reset:absent"
  fi

  if grep -Fq 'cd "$GATEWAY_WORKDIR"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_build_root:gateway"
  else
    push_error "deploy script missing gateway workspace build step"
  fi

  if grep -Fq 'cd "$WORKER_WORKDIR"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_build_root:worker"
  else
    push_error "deploy script missing worker workspace build step"
  fi

  if grep -Fq 'write_authority_manifest' "$DEPLOY_SCRIPT"; then
    push_check "deploy_manifest_generation:present"
  else
    push_error "deploy script missing authority manifest generation"
  fi

  if grep -Fq 'cortex_runtime_authority.json' "$DEPLOY_SCRIPT"; then
    push_check "deploy_manifest_path:present"
  else
    push_error "deploy script missing authority manifest path"
  fi

  if grep -Fq '"deploymentMode": "not_deployed"' "$DEPLOY_SCRIPT"; then
    push_check "deploy_cortex_web_mode:not_deployed"
  else
    push_error "deploy script missing cortex-web not_deployed declaration"
  fi

  local worker_exec gateway_exec
  worker_exec="$(unit_value "ExecStart" "$REPO_TEMPLATE_WORKER" || true)"
  gateway_exec="$(unit_value "ExecStart" "$REPO_TEMPLATE_GATEWAY" || true)"

  if [[ "$worker_exec" == "__DEPLOY_ROOT__/repo/nostra/worker/target/release/cortex_worker" ]]; then
    push_check "worker_exec_template:repo_local"
  else
    push_error "worker template ExecStart is not repo-local"
  fi

  if [[ "$gateway_exec" == "__DEPLOY_ROOT__/repo/cortex/target/release/cortex-gateway" ]]; then
    push_check "gateway_exec_template:repo_local"
  else
    push_error "gateway template ExecStart is not repo-local"
  fi

  if grep -Fq '`docs/cortex/eudaemon-alpha-phase6-hetzner.md`' "$DOCS_INDEX" && \
     grep -Fq '`docs/cortex/eudaemon-alpha-phase6-checklist.md`' "$DOCS_INDEX"; then
    push_check "docs_index:phase6_links_present"
  else
    push_error "docs index missing active phase6 authority docs"
  fi

  check_file_contains "runbook_promotion_path" "$PRIMARY_RUNBOOK" "scripts/promote_eudaemon_alpha_vps.sh"
  check_file_contains "runbook_manifest_path" "$PRIMARY_RUNBOOK" "cortex_runtime_authority.json"
  check_file_contains "runbook_repo_authority" "$PRIMARY_RUNBOOK" "/srv/nostra/eudaemon-alpha/repo"
  check_file_contains "runbook_operator_ssh" "$PRIMARY_RUNBOOK" "operator-initiated"
  check_file_contains "runbook_cortex_web_not_deployed" "$PRIMARY_RUNBOOK" "not_deployed"
  check_file_contains "checklist_promotion_path" "$CHECKLIST_DOC" "scripts/promote_eudaemon_alpha_vps.sh"
  check_file_contains "docs_index_promotion_path" "$DOCS_INDEX" "scripts/promote_eudaemon_alpha_vps.sh"

  while IFS= read -r rel_path; do
    [[ -z "$rel_path" ]] && continue
    if [[ -e "$ROOT_DIR/$rel_path" ]] && \
       (git -C "$ROOT_DIR" ls-files --error-unmatch "$rel_path" >/dev/null 2>&1 || \
        [[ -n "$(git -C "$ROOT_DIR" status --short -- "$rel_path")" ]]); then
      push_check "docs_index_target:$rel_path"
    else
      push_error "docs index target missing from repo: $rel_path"
    fi
  done < <(grep -Eo '`(docs|research)/[^`]+`' "$DOCS_INDEX" | tr -d '`')
}

check_running_process_provenance() {
  local unit_name="$1"
  local expected_exec="$2"
  local active_state main_pid running_exec

  if [[ "$SKIP_PROCESS_PROVENANCE" == "1" ]]; then
    push_check "process_provenance_skipped:$unit_name"
    return 0
  fi

  if ! command -v systemctl >/dev/null 2>&1; then
    push_error "systemctl unavailable for process provenance check: $unit_name"
    return 0
  fi

  active_state="$(systemctl show -p ActiveState --value "$unit_name" 2>/dev/null || true)"
  if [[ "$active_state" == "active" ]]; then
    push_check "unit_active:$unit_name"
  else
    push_error "unit is not active: $unit_name state=$active_state"
    return 0
  fi

  main_pid="$(systemctl show -p MainPID --value "$unit_name" 2>/dev/null || true)"
  if [[ -z "$main_pid" || "$main_pid" == "0" ]]; then
    push_error "unit has no running MainPID: $unit_name"
    return 0
  fi

  running_exec="$(readlink -f "/proc/$main_pid/exe" 2>/dev/null || true)"
  if [[ -z "$running_exec" ]]; then
    push_error "unable to resolve running executable for $unit_name pid=$main_pid"
    return 0
  fi

  if [[ "$running_exec" == "$expected_exec" ]]; then
    push_check "unit_process_matches_manifest:$unit_name"
  else
    push_error "running executable mismatch for $unit_name expected=$expected_exec actual=$running_exec"
  fi
}

check_host_contract() {
  check_dir_exists "deploy_root" "$DEPLOY_ROOT"
  check_dir_exists "repo_root" "$REPO_ROOT"
  check_dir_exists "state_root" "$STATE_ROOT"
  check_file_exists "manifest" "$MANIFEST_PATH"
  check_json_parse "manifest_json" "$MANIFEST_PATH"

  if git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    push_check "repo_git_checkout:true"
  else
    push_error "repo root is not a git checkout: $REPO_ROOT"
  fi

  local manifest_commit manifest_repo_root manifest_gateway_exec manifest_gateway_workdir
  local manifest_worker_exec manifest_worker_workdir manifest_web_mode manifest_web_root
  local manifest_runbook manifest_index actual_commit gateway_unit worker_unit
  local gateway_exec gateway_workdir worker_exec worker_workdir

  manifest_commit="$(extract_json_string "commit" "$MANIFEST_PATH" 1 || true)"
  manifest_repo_root="$(extract_json_string "repoRoot" "$MANIFEST_PATH" 1 || true)"
  manifest_gateway_exec="$(extract_json_string "execPath" "$MANIFEST_PATH" 1 || true)"
  manifest_worker_exec="$(extract_json_string "execPath" "$MANIFEST_PATH" 2 || true)"
  manifest_gateway_workdir="$(extract_json_string "workingDirectory" "$MANIFEST_PATH" 1 || true)"
  manifest_worker_workdir="$(extract_json_string "workingDirectory" "$MANIFEST_PATH" 2 || true)"
  manifest_web_mode="$(extract_json_string "deploymentMode" "$MANIFEST_PATH" 1 || true)"
  manifest_web_root="$(extract_json_string "sourceRoot" "$MANIFEST_PATH" 1 || true)"
  manifest_runbook="$(extract_json_string "primaryRunbook" "$MANIFEST_PATH" 1 || true)"
  manifest_index="$(extract_json_string "operationsIndex" "$MANIFEST_PATH" 1 || true)"

  if [[ -n "$manifest_commit" ]]; then
    push_check "manifest_commit:present"
  else
    push_error "manifest missing git.commit"
  fi

  if [[ -n "$manifest_web_mode" ]]; then
    push_check "manifest_cortex_web_mode:$manifest_web_mode"
  else
    push_error "manifest missing runtime.cortexWeb.deploymentMode"
  fi

  if [[ -n "$manifest_runbook" && -f "$manifest_runbook" ]]; then
    push_check "manifest_primary_runbook:$manifest_runbook"
  else
    push_error "manifest primaryRunbook missing or unreadable: $manifest_runbook"
  fi

  if [[ -n "$manifest_index" && -f "$manifest_index" ]]; then
    push_check "manifest_operations_index:$manifest_index"
  else
    push_error "manifest operationsIndex missing or unreadable: $manifest_index"
  fi

  actual_commit="$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || true)"
  if [[ -n "$manifest_commit" && "$manifest_commit" == "$actual_commit" ]]; then
    push_check "manifest_commit_matches_repo_head:true"
  else
    push_error "manifest git.commit does not match repo HEAD"
  fi

  if [[ -n "$manifest_repo_root" && "$manifest_repo_root" == "$REPO_ROOT" ]]; then
    push_check "manifest_repo_root_matches_expected:true"
  else
    push_error "manifest repoRoot does not match expected repo root"
  fi

  gateway_unit="$SYSTEMD_ROOT/cortex-gateway.service"
  worker_unit="$SYSTEMD_ROOT/cortex-worker.service"
  check_file_exists "gateway_unit" "$gateway_unit"
  check_file_exists "worker_unit" "$worker_unit"

  if [[ -f "$gateway_unit" ]]; then
    gateway_exec="$(unit_value "ExecStart" "$gateway_unit" || true)"
    gateway_workdir="$(unit_value "WorkingDirectory" "$gateway_unit" || true)"
    if rendered_under_repo "$gateway_exec" "$REPO_ROOT/"; then
      push_check "gateway_exec_under_repo:true"
    else
      push_error "gateway ExecStart is outside repo mirror: $gateway_exec"
    fi
    if [[ -d "$gateway_workdir" ]]; then
      push_check "gateway_workdir_exists:true"
    else
      push_error "gateway WorkingDirectory missing: $gateway_workdir"
    fi
    if [[ -n "$manifest_gateway_exec" && "$manifest_gateway_exec" == "$gateway_exec" ]]; then
      push_check "gateway_exec_matches_manifest:true"
    else
      push_error "gateway ExecStart does not match manifest"
    fi
    if [[ -n "$manifest_gateway_workdir" && "$manifest_gateway_workdir" == "$gateway_workdir" ]]; then
      push_check "gateway_workdir_matches_manifest:true"
    else
      push_error "gateway WorkingDirectory does not match manifest"
    fi
    check_running_process_provenance "cortex-gateway.service" "$manifest_gateway_exec"
  fi

  if [[ -f "$worker_unit" ]]; then
    worker_exec="$(unit_value "ExecStart" "$worker_unit" || true)"
    worker_workdir="$(unit_value "WorkingDirectory" "$worker_unit" || true)"
    if rendered_under_repo "$worker_exec" "$REPO_ROOT/"; then
      push_check "worker_exec_under_repo:true"
    else
      push_error "worker ExecStart is outside repo mirror: $worker_exec"
    fi
    if [[ -d "$worker_workdir" ]]; then
      push_check "worker_workdir_exists:true"
    else
      push_error "worker WorkingDirectory missing: $worker_workdir"
    fi
    if [[ -n "$manifest_worker_exec" && "$manifest_worker_exec" == "$worker_exec" ]]; then
      push_check "worker_exec_matches_manifest:true"
    else
      push_error "worker ExecStart does not match manifest"
    fi
    if [[ -n "$manifest_worker_workdir" && "$manifest_worker_workdir" == "$worker_workdir" ]]; then
      push_check "worker_workdir_matches_manifest:true"
    else
      push_error "worker WorkingDirectory does not match manifest"
    fi
    check_running_process_provenance "cortex-worker.service" "$manifest_worker_exec"
  fi

  if [[ "$manifest_web_mode" == "not_deployed" ]]; then
    push_check "cortex_web_mode_matches_boundary:true"
  else
    push_error "manifest cortexWeb deploymentMode must equal not_deployed"
  fi

  if [[ -n "$manifest_web_root" && -d "$manifest_web_root" ]]; then
    push_check "cortex_web_source_root_exists:true"
  else
    push_error "manifest cortexWeb sourceRoot missing or unreadable: $manifest_web_root"
  fi

  if [[ -n "$manifest_runbook" && -f "$manifest_runbook" ]]; then
    check_file_contains "host_runbook_promotion_path" "$manifest_runbook" "scripts/promote_eudaemon_alpha_vps.sh"
    check_file_contains "host_runbook_repo_authority" "$manifest_runbook" "/srv/nostra/eudaemon-alpha/repo"
    check_file_contains "host_runbook_manifest_authority" "$manifest_runbook" "cortex_runtime_authority.json"
    check_file_contains "host_runbook_not_deployed" "$manifest_runbook" "not_deployed"
  fi

  if [[ -f "$CHECKLIST_DOC" ]]; then
    check_file_contains "host_checklist_promotion_path" "$CHECKLIST_DOC" "scripts/promote_eudaemon_alpha_vps.sh"
    check_file_contains "host_checklist_authority_check" "$CHECKLIST_DOC" "check_vps_runtime_authority.sh"
  fi
}

if [[ "$MODE" == "repo-contract" ]]; then
  check_repo_contract
else
  check_host_contract
fi

status="pass"
if [[ ${#errors[@]} -gt 0 ]]; then
  status="fail"
fi

printf '{\n'
printf '  "schemaVersion": "1.0.0",\n'
printf '  "mode": "%s",\n' "$(json_escape "$MODE")"
printf '  "status": "%s",\n' "$status"
printf '  "checks": [\n'
for i in "${!checks[@]}"; do
  suffix=","
  if [[ "$i" -eq $((${#checks[@]} - 1)) ]]; then
    suffix=""
  fi
  printf '    "%s"%s\n' "$(json_escape "${checks[$i]}")" "$suffix"
done
printf '  ],\n'
printf '  "errors": [\n'
for i in "${!errors[@]}"; do
  suffix=","
  if [[ "$i" -eq $((${#errors[@]} - 1)) ]]; then
    suffix=""
  fi
  printf '    "%s"%s\n' "$(json_escape "${errors[$i]}")" "$suffix"
done
printf '  ]\n'
printf '}\n'

if [[ "$status" != "pass" ]]; then
  exit 1
fi
