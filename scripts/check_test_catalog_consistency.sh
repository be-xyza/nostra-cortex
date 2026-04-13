#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MODE="advisory"
ALLOW_SYNTHETIC=false
REQUIRE_PRESENT=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="${2:-advisory}"
      shift 2
      ;;
    --allow-synthetic-latest)
      ALLOW_SYNTHETIC=true
      shift
      ;;
    --require-present)
      REQUIRE_PRESENT=true
      shift
      ;;
    *)
      echo "FAIL: unknown arg '$1'" >&2
      exit 2
      ;;
  esac
done

case "$MODE" in
  advisory|blocking) ;;
  *)
    echo "FAIL: unsupported mode '$MODE'" >&2
    exit 2
    ;;
esac

python3 - "$ROOT_DIR" "$MODE" "$ALLOW_SYNTHETIC" "$REQUIRE_PRESENT" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
mode = sys.argv[2]
allow_synthetic = sys.argv[3].lower() == "true"
require_present = sys.argv[4].lower() == "true"

logs = root / "logs" / "testing"
catalog_path = logs / "test_catalog_latest.json"
summary_path = logs / "test_gate_summary_latest.json"
runs_dir = logs / "runs"

required = [catalog_path, summary_path, runs_dir]
missing = [str(path) for path in required if not path.exists()]
if missing:
    if require_present:
        print("FAIL: missing testing artifacts:")
        for item in missing:
            print(f" - {item}")
        raise SystemExit(1)
    if len(missing) == len(required):
        print(
            "PASS: test catalog artifacts absent "
            f"(mode={mode}, artifact_state=absent)"
        )
        raise SystemExit(0)
    print("FAIL: test catalog artifacts are partially present:")
    for item in missing:
        print(f" - {item}")
    raise SystemExit(1)


def load(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"FAIL: invalid json {path}: {exc}")
        raise SystemExit(1)

catalog = load(catalog_path)
summary = load(summary_path)

if summary.get("mode") != mode:
    print(f"FAIL: summary mode mismatch expected={mode} actual={summary.get('mode')}")
    raise SystemExit(1)

for field in ["schema_version", "generated_at", "tests"]:
    if field not in catalog:
        print(f"FAIL: catalog missing required field '{field}'")
        raise SystemExit(1)

for field in [
    "schema_version",
    "generated_at",
    "mode",
    "catalog_valid",
    "run_artifacts_valid",
    "required_blockers_pass",
    "overall_verdict",
    "latest_run_id",
    "failures",
    "counts",
]:
    if field not in summary:
        print(f"FAIL: summary missing required field '{field}'")
        raise SystemExit(1)

latest_run_id = str(summary.get("latest_run_id") or "").strip()
if not latest_run_id:
    print("FAIL: summary latest_run_id is empty")
    raise SystemExit(1)

run_path = runs_dir / f"{latest_run_id}.json"
if not run_path.exists():
    print(f"FAIL: latest run artifact missing: {run_path}")
    raise SystemExit(1)

run = load(run_path)
for field in [
    "schema_version",
    "run_id",
    "started_at",
    "finished_at",
    "agent_id",
    "environment",
    "git_commit",
    "results",
    "artifacts",
    "warnings",
]:
    if field not in run:
        print(f"FAIL: run artifact missing required field '{field}'")
        raise SystemExit(1)

if run.get("run_id") != latest_run_id:
    print("FAIL: latest_run_id does not match run artifact id")
    raise SystemExit(1)

if mode == "blocking":
    if summary.get("overall_verdict") != "ready":
        print("FAIL: blocking mode requires overall_verdict=ready")
        raise SystemExit(1)
    if not bool(summary.get("required_blockers_pass")):
        print("FAIL: blocking mode requires required_blockers_pass=true")
        raise SystemExit(1)

if not allow_synthetic:
    warnings = [str(item).lower() for item in run.get("warnings", [])]
    if any("synthetic" in item for item in warnings):
        print("FAIL: latest run is synthetic and --allow-synthetic-latest was not set")
        raise SystemExit(1)

print(
    "PASS: test catalog artifacts are consistent "
    f"(mode={mode}, run_id={latest_run_id}, verdict={summary.get('overall_verdict')})"
)
PY
