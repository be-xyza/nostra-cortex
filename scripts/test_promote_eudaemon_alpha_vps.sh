#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
PROMOTE_SCRIPT="$ROOT_DIR/scripts/promote_eudaemon_alpha_vps.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

tmpdir="$(mktemp -d /tmp/cortex-promote-vps.XXXXXX)"
origin_repo="$tmpdir/origin.git"
work_repo="$tmpdir/work"
ssh_log="$tmpdir/ssh.log"
ssh_stub="$tmpdir/fake-ssh.sh"

git init --bare -q "$origin_repo"
git clone -q "$origin_repo" "$work_repo"
git -C "$work_repo" config user.email test@example.com
git -C "$work_repo" config user.name test

printf 'main\n' >"$work_repo/README.md"
git -C "$work_repo" add README.md
git -C "$work_repo" commit -q -m 'main commit'
git -C "$work_repo" push -q origin HEAD:main
main_commit="$(git -C "$work_repo" rev-parse HEAD)"

git -C "$work_repo" checkout -q -b side
printf 'side\n' >"$work_repo/SIDE.md"
git -C "$work_repo" add SIDE.md
git -C "$work_repo" commit -q -m 'side commit'
side_commit="$(git -C "$work_repo" rev-parse HEAD)"
git -C "$work_repo" checkout -q --detach "$main_commit"

cat <<'STUB' >"$ssh_stub"
#!/usr/bin/env bash
set -euo pipefail

args=("$@")
option_value_expected=0
host=""
index=0

while (( index < ${#args[@]} )); do
  token="${args[$index]}"
  if (( option_value_expected == 1 )); then
    option_value_expected=0
    ((index += 1))
    continue
  fi
  case "$token" in
    -F|-i|-J|-l|-o|-p)
      option_value_expected=1
      ((index += 1))
      continue
      ;;
    -*)
      ((index += 1))
      continue
      ;;
    *)
      host="$token"
      ((index += 1))
      break
      ;;
  esac
done

if [[ -z "$host" ]]; then
  echo "missing ssh host" >&2
  exit 1
fi

remote_args=("${args[@]:$index}")
cat >/dev/null
printf 'host=%s\n' "$host" >>"$FAKE_SSH_LOG"
printf 'args=%s\n' "${args[*]}" >>"$FAKE_SSH_LOG"

if [[ "${FAKE_SSH_MODE:-success}" == "fail" ]]; then
  echo "remote failure" >&2
  exit 1
fi

target_commit="${remote_args[3]}"
manifest_path="${remote_args[7]}"
if [[ "${FAKE_SSH_MODE:-success}" == "mismatch" ]]; then
  target_commit="mismatch"
fi

echo "DEPLOYED_COMMIT=$target_commit"
echo "MANIFEST_PATH=$manifest_path"
STUB
chmod +x "$ssh_stub"

success_output="$({
  cd "$work_repo"
  FAKE_SSH_LOG="$ssh_log" \
  NOSTRA_WORKSPACE_ROOT="$work_repo" \
  SSH_BIN="$ssh_stub" \
  NOSTRA_EUDAEMON_PROMOTE_FETCH_REMOTE=0 \
  NOSTRA_EUDAEMON_VPS_HOST="fixture-host" \
  bash "$PROMOTE_SCRIPT" "$main_commit"
} 2>&1)"

if ! grep -Fq "Promoted commit $main_commit to fixture-host" <<<"$success_output"; then
  fail "promotion script did not report successful commit promotion"
fi

if ! grep -Fq "host=fixture-host" "$ssh_log"; then
  fail "promotion script did not invoke ssh on success path"
fi

rm -f "$ssh_log"

ssh_args_output="$({
  cd "$work_repo"
  FAKE_SSH_LOG="$ssh_log" \
  NOSTRA_WORKSPACE_ROOT="$work_repo" \
  SSH_BIN="$ssh_stub" \
  NOSTRA_EUDAEMON_PROMOTE_FETCH_REMOTE=0 \
  NOSTRA_EUDAEMON_VPS_HOST="fixture-host" \
  NOSTRA_EUDAEMON_VPS_SSH_ARGS="-F /tmp/fake-config -p 2222" \
  bash "$PROMOTE_SCRIPT" "$main_commit"
} 2>&1)"

if ! grep -Fq "Promoted commit $main_commit to fixture-host" <<<"$ssh_args_output"; then
  fail "promotion script did not report successful promotion with ssh args override"
fi

if ! grep -Fq "args=-F /tmp/fake-config -p 2222 fixture-host" "$ssh_log"; then
  fail "promotion script did not pass through ssh args override"
fi

rm -f "$ssh_log"

set +e
failure_output="$({
  cd "$work_repo"
  FAKE_SSH_LOG="$ssh_log" \
  NOSTRA_WORKSPACE_ROOT="$work_repo" \
  SSH_BIN="$ssh_stub" \
  NOSTRA_EUDAEMON_PROMOTE_FETCH_REMOTE=0 \
  NOSTRA_EUDAEMON_VPS_HOST="fixture-host" \
  bash "$PROMOTE_SCRIPT" "$side_commit"
} 2>&1)"
failure_rc=$?
set -e

if [[ "$failure_rc" -eq 0 ]]; then
  fail "promotion script unexpectedly accepted a commit outside origin/main"
fi

if ! grep -Fq "not reachable from" <<<"$failure_output"; then
  fail "promotion script did not explain origin/main reachability failure"
fi

if [[ -f "$ssh_log" ]]; then
  fail "promotion script should not reach ssh when commit is outside origin/main"
fi

set +e
remote_failure_output="$({
  cd "$work_repo"
  FAKE_SSH_LOG="$ssh_log" \
  NOSTRA_WORKSPACE_ROOT="$work_repo" \
  FAKE_SSH_MODE=fail \
  SSH_BIN="$ssh_stub" \
  NOSTRA_EUDAEMON_PROMOTE_FETCH_REMOTE=0 \
  NOSTRA_EUDAEMON_VPS_HOST="fixture-host" \
  bash "$PROMOTE_SCRIPT" "$main_commit"
} 2>&1)"
remote_failure_rc=$?
set -e

if [[ "$remote_failure_rc" -eq 0 ]]; then
  fail "promotion script unexpectedly passed when ssh failed"
fi

if ! grep -Fq "remote failure" <<<"$remote_failure_output"; then
  fail "promotion script did not surface remote ssh failure"
fi

echo "PASS: vps promotion script fixture coverage"
