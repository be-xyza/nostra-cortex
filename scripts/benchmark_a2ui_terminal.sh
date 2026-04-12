#!/usr/bin/env bash
set -euo pipefail

workspace_root="${NOSTRA_WORKSPACE_ROOT:-$(git rev-parse --show-toplevel)}"
experiment_dir="${workspace_root}/cortex/experiments/a2ui-terminal"
export NOSTRA_WORKSPACE_ROOT="${workspace_root}"

if [[ ! -d "${experiment_dir}" ]]; then
  echo "a2ui-terminal experiment directory not found at ${experiment_dir}" >&2
  exit 1
fi

npm --prefix "${experiment_dir}" run benchmark -- "$@"
