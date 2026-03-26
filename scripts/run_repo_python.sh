#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
LIB_DIR="$ROOT_DIR/scripts/lib"

declare -a CANDIDATES=()
declare -a DIAGNOSTICS=()
SELECTED_CANDIDATE=""
SELECTED_PATH=""
SELECTED_VERSION=""
SELECTED_EXECUTABLE=""

add_candidate() {
  local candidate="$1"
  [[ -n "$candidate" ]] || return 0
  CANDIDATES+=("$candidate")
}

resolve_command_path() {
  local candidate="$1"
  if [[ "$candidate" == */* ]]; then
    printf '%s\n' "$candidate"
  else
    command -v -- "$candidate" 2>/dev/null || return 1
  fi
}

signature_hint() {
  local resolved="$1"
  [[ -n "$resolved" ]] || return 0
  [[ -x "$resolved" ]] || return 0
  command -v codesign >/dev/null 2>&1 || return 0
  case "$resolved" in
    /usr/local/bin/python3*|/Library/Frameworks/Python.framework/*)
      local verify_output
      verify_output="$(codesign --verify --verbose=2 "$resolved" 2>&1 || true)"
      if [[ "$verify_output" == *"invalid signature"* ]]; then
        printf '%s' 'codesign reports an invalid signature on the framework-backed interpreter'
      fi
      ;;
  esac
}

probe_candidate() {
  local candidate="$1"
  local resolved output rc hint

  if ! resolved="$(resolve_command_path "$candidate")"; then
    DIAGNOSTICS+=(" - $candidate: not found in PATH")
    return 1
  fi

  if [[ ! -x "$resolved" ]]; then
    DIAGNOSTICS+=(" - $candidate -> $resolved: not executable")
    return 1
  fi

  if output="$(PYTHONPATH="$LIB_DIR${PYTHONPATH:+:$PYTHONPATH}" "$resolved" -c 'import sys; print("{}.{}.{}".format(*sys.version_info[:3])); print(sys.executable); raise SystemExit(0 if sys.version_info >= (3, 9) else 11)' 2>&1)"; then
    SELECTED_CANDIDATE="$candidate"
    SELECTED_PATH="$resolved"
    SELECTED_VERSION="$(printf '%s\n' "$output" | sed -n '1p')"
    SELECTED_EXECUTABLE="$(printf '%s\n' "$output" | sed -n '2p')"
    return 0
  fi

  rc=$?
  hint="$(signature_hint "$resolved")"
  local message=" - $candidate -> $resolved: launch probe failed (exit $rc)"
  if [[ -n "$output" ]]; then
    output="${output//$'\n'/; }"
    message+=" [$output]"
  fi
  if [[ -n "$hint" ]]; then
    message+="; $hint"
  fi
  DIAGNOSTICS+=("$message")
  return 1
}

select_exec_candidate() {
  local candidate resolved
  for candidate in "${CANDIDATES[@]}"; do
    if ! resolved="$(resolve_command_path "$candidate")"; then
      continue
    fi
    if [[ -x "$resolved" ]]; then
      SELECTED_CANDIDATE="$candidate"
      SELECTED_PATH="$resolved"
      return 0
    fi
  done
  return 1
}

print_diagnostics() {
  if [[ -n "$SELECTED_PATH" ]]; then
    echo "PASS: repo Python runtime available"
    echo "selected_candidate=$SELECTED_CANDIDATE"
    echo "selected_path=$SELECTED_PATH"
    echo "selected_version=$SELECTED_VERSION"
    echo "selected_executable=$SELECTED_EXECUTABLE"
    return 0
  fi

  echo "FAIL: unable to resolve a usable Python 3.9+ interpreter for repo governance scripts" >&2
  echo "checked_candidates:" >&2
  if [[ "${#DIAGNOSTICS[@]}" -eq 0 ]]; then
    echo " - no candidates were configured" >&2
  else
    printf '%s\n' "${DIAGNOSTICS[@]}" >&2
  fi
  echo "hint: set NOSTRA_PYTHON_BIN to a known-good Python 3.9+ interpreter, or repair the local Python install if the framework-backed path is invalidly signed" >&2
  return 1
}

if [[ -n "${NOSTRA_PYTHON_BIN:-}" ]]; then
  add_candidate "$NOSTRA_PYTHON_BIN"
fi
add_candidate "/usr/bin/python3"
add_candidate "/usr/local/bin/python3"
add_candidate "python3"

case "${1:-}" in
  --diagnose)
    for candidate in "${CANDIDATES[@]}"; do
      if probe_candidate "$candidate"; then
        break
      fi
    done
    print_diagnostics
    exit $?
    ;;
  --resolve)
    if select_exec_candidate; then
      printf '%s\n' "$SELECTED_PATH"
      exit 0
    fi
    for candidate in "${CANDIDATES[@]}"; do
      probe_candidate "$candidate" || true
    done
    print_diagnostics
    exit 1
    ;;
esac

if ! select_exec_candidate; then
  for candidate in "${CANDIDATES[@]}"; do
    probe_candidate "$candidate" || true
  done
  print_diagnostics
  exit 1
fi

export PYTHONPATH="$LIB_DIR${PYTHONPATH:+:$PYTHONPATH}"
exec "$SELECTED_PATH" "$@"
