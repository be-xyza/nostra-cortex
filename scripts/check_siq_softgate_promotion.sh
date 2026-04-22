#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
EMIT_GITHUB_OUTPUT=false

if [[ "${1:-}" == "--emit-github-output" ]]; then
  EMIT_GITHUB_OUTPUT=true
fi

python3 - "$ROOT_DIR" "$EMIT_GITHUB_OUTPUT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
emit = sys.argv[2].lower() == "true"
runs_dir = root / "logs" / "siq" / "runs"

promotion_ready = False
reason = "insufficient_runs"
required = 5

runs = []
if runs_dir.exists():
    for path in runs_dir.glob("*.json"):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        generated = str(payload.get("generated_at", ""))
        if not generated:
            continue
        runs.append(payload)

runs.sort(key=lambda row: row.get("generated_at", ""), reverse=True)
observed = [row for row in runs if str(row.get("mode", "")).strip() == "observe"]

if len(observed) >= required:
    latest = observed[:required]
    verdicts_ready = all(str(row.get("overall_verdict", "")) == "ready" for row in latest)
    unresolved_p0 = False
    for row in latest:
        for failure in row.get("failures", []):
            if str(failure.get("severity", "")).upper() == "P0":
                unresolved_p0 = True
                break
        if unresolved_p0:
            break

    if verdicts_ready and not unresolved_p0:
        promotion_ready = True
        reason = "ready"
    elif not verdicts_ready:
        reason = "non_ready_verdict_in_latest_observe_window"
    else:
        reason = "unresolved_p0_in_latest_observe_window"
else:
    reason = f"insufficient_observe_runs:{len(observed)}/{required}"

print(f"SIQ softgate promotion: {'ready' if promotion_ready else 'not-ready'} ({reason})")
if emit:
    out = Path(__import__("os").environ.get("GITHUB_OUTPUT", ""))
    if out:
        with out.open("a", encoding="utf-8") as handle:
            handle.write(f"promote={'true' if promotion_ready else 'false'}\n")
            handle.write(f"reason={reason}\n")

raise SystemExit(0)
PY
