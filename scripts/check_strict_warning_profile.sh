#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUDGET_PATH="$ROOT_DIR/scripts/strict_warning_budget.json"
MODE="auto"
TARGET_CRATE=""
FREEZE_DATE="${STRICT_WARNING_FREEZE_DATE:-2026-03-15}"

while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --mode)
      MODE="${2:-auto}"
      shift 2
      ;;
    --crate)
      TARGET_CRATE="${2:-}"
      shift 2
      ;;
    *)
      echo "FAIL: unsupported argument '${1:-}'" >&2
      exit 2
      ;;
  esac
done

case "$MODE" in
  auto|advisory|blocking) ;;
  *)
    echo "FAIL: unsupported mode '$MODE'" >&2
    exit 2
    ;;
esac

if [[ ! -f "$BUDGET_PATH" ]]; then
  echo "FAIL: missing warning budget file: $BUDGET_PATH" >&2
  exit 1
fi

TODAY="$(date -u +%F)"
EFFECTIVE_MODE="$MODE"
if [[ "$MODE" == "auto" ]]; then
  if [[ "$TODAY" < "$FREEZE_DATE" ]]; then
    EFFECTIVE_MODE="advisory"
  else
    EFFECTIVE_MODE="blocking"
  fi
fi

python3 - "$ROOT_DIR" "$BUDGET_PATH" "$EFFECTIVE_MODE" "$TARGET_CRATE" <<'PY'
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

root = Path(sys.argv[1])
budget_path = Path(sys.argv[2])
mode = sys.argv[3]
target_crate = sys.argv[4].strip()

budget = json.loads(budget_path.read_text(encoding="utf-8"))
crates = budget.get("crates", {})
if not isinstance(crates, dict) or not crates:
    print("FAIL: warning budget file missing crates")
    raise SystemExit(1)

if target_crate:
    if target_crate not in crates:
        print(f"FAIL: unknown crate '{target_crate}'")
        raise SystemExit(2)
    crates = {target_crate: crates[target_crate]}

failures: list[str] = []
for crate_name, config in crates.items():
    manifest_rel = str(config.get("manifest", "")).strip()
    max_warnings = int(config.get("max_warnings", 0))
    cargo_args = config.get("cargo_args", ["check"])
    if not manifest_rel:
        failures.append(f"{crate_name}: missing manifest path")
        continue
    if not isinstance(cargo_args, list) or not cargo_args or not all(
        isinstance(item, str) and item.strip() for item in cargo_args
    ):
        failures.append(f"{crate_name}: invalid cargo_args")
        continue

    manifest = root / manifest_rel
    if not manifest.exists():
        failures.append(f"{crate_name}: manifest not found at {manifest_rel}")
        continue

    cmd = ["cargo", *cargo_args, "--manifest-path", str(manifest)]
    proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    output = f"{proc.stdout}\n{proc.stderr}"
    warning_count = len(re.findall(r"(?m)^warning:", output))

    if proc.returncode != 0:
        failures.append(f"{crate_name}: cargo check failed ({manifest_rel})")
        continue

    if warning_count > max_warnings:
        failures.append(
            f"{crate_name}: warnings {warning_count} exceed budget {max_warnings} ({manifest_rel})"
        )

if failures:
    header = "FAIL" if mode == "blocking" else "WARN"
    print(f"{header}: strict warning profile violations")
    for issue in failures:
        print(f" - {issue}")
    raise SystemExit(1 if mode == "blocking" else 0)

print(f"PASS: strict warning profile ({mode})")
PY
