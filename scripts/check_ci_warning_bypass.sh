#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MODE="advisory"

if [[ "${1:-}" == "--strict" ]]; then
  MODE="strict"
elif [[ "${1:-}" == "--advisory" || -z "${1:-}" ]]; then
  MODE="advisory"
else
  echo "FAIL: unsupported mode '${1:-}' (expected --strict or --advisory)" >&2
  exit 2
fi

python3 - "$ROOT_DIR" "$MODE" <<'PY'
from __future__ import annotations

import json
import re
import sys
from datetime import date, datetime, timezone
from pathlib import Path

root = Path(sys.argv[1])
mode = sys.argv[2]

workflow_root = root / ".github" / "workflows"
exceptions_path = root / "shared" / "standards" / "alignment_contract_exceptions.json"
report_path = root / "logs" / "alignment" / "ci_warning_bypass_latest.json"


def parse_expiry(raw: str | None) -> date | None:
    if raw is None:
        return None
    text = raw.strip()
    if not text:
        return None
    try:
        return date.fromisoformat(text)
    except ValueError:
        pass
    try:
        return datetime.fromisoformat(text.replace("Z", "+00:00")).date()
    except ValueError:
        return None


def load_exceptions() -> list[dict]:
    if not exceptions_path.exists():
        return []
    try:
        payload = json.loads(exceptions_path.read_text(encoding="utf-8"))
    except Exception:
        return []

    out: list[dict] = []
    for row in payload.get("exceptions", []):
        if not bool(row.get("enabled", True)):
            continue
        policy = str(row.get("policy", "")).strip()
        rule_id = str(row.get("rule_id", "")).strip()
        ref_id = str(row.get("id", "")).strip()
        if not (
            policy in {"ci_warning_bypass", "ci_warning_bypass_contract"}
            or rule_id == "ci_warning_bypass_contract"
            or ref_id.startswith("ci_warning_bypass_")
        ):
            continue
        expiry = parse_expiry(str(row.get("expires_at", "")).strip())
        if expiry is None or expiry < date.today():
            continue
        out.append(row)
    return out


def matches_exception(entry: dict, exc: dict) -> bool:
    workflow_path = str(exc.get("workflow_path", "")).strip()
    if workflow_path and workflow_path != entry["workflow_path"]:
        return False

    line = exc.get("line")
    if line is not None:
        try:
            if int(line) != int(entry["line"]):
                return False
        except Exception:
            return False

    pattern = str(exc.get("pattern", "")).strip()
    if pattern and pattern not in entry["match_text"] and pattern != entry["pattern_id"]:
        return False

    return True


patterns = [
    (
        "rustflags_allow_warnings",
        re.compile(r"(?i)\bRUSTFLAGS\b[^\n#]*-A\s*warnings\b"),
        "Remove warning suppression from RUSTFLAGS in active workflows.",
    ),
    (
        "rustdocflags_allow_warnings",
        re.compile(r"(?i)\bRUSTDOCFLAGS\b[^\n#]*-A\s*warnings\b"),
        "Remove warning suppression from RUSTDOCFLAGS in active workflows.",
    ),
    (
        "cargo_clippy_allow_warnings",
        re.compile(r"(?i)\bcargo\s+clippy\b[^\n#]*--[^\n#]*-A\s*warnings\b"),
        "Use blocking clippy policy; do not pass -A warnings.",
    ),
    (
        "rust_command_allow_warnings",
        re.compile(
            r"(?i)\bcargo\s+(?:test|check|build|run|doc|fmt|clippy)\b[^\n#]*-A\s*warnings\b"
        ),
        "Remove -A warnings from cargo commands in active workflows.",
    ),
]

workflow_files = sorted(workflow_root.glob("*.yml")) + sorted(workflow_root.glob("*.yaml"))
workflow_files = [path for path in workflow_files if "/_archive/" not in str(path)]

violations: list[dict] = []
waived: list[dict] = []
exceptions = load_exceptions()
scanned_files: list[str] = []

for path in workflow_files:
    if not path.is_file():
        continue
    rel = path.relative_to(root).as_posix()
    scanned_files.append(rel)
    lines = path.read_text(encoding="utf-8").splitlines()
    for idx, line in enumerate(lines, start=1):
        for pattern_id, regex, hint in patterns:
            if not regex.search(line):
                continue
            entry = {
                "workflow_path": rel,
                "line": idx,
                "pattern_id": pattern_id,
                "match_text": line.strip(),
                "remediation_hint": hint,
            }
            matched_exception = next(
                (exc for exc in exceptions if matches_exception(entry, exc)), None
            )
            if matched_exception is not None:
                waived.append(
                    {
                        **entry,
                        "exception_id": str(matched_exception.get("id", "")).strip() or None,
                    }
                )
            else:
                violations.append(entry)

if not scanned_files:
    violations.append(
        {
            "workflow_path": ".github/workflows",
            "line": 0,
            "pattern_id": "workflow_discovery_failure",
            "match_text": "no active workflow files discovered",
            "remediation_hint": "Ensure active workflow files exist under .github/workflows/*.yml.",
        }
    )

status = "pass"
if violations and mode == "strict":
    status = "fail"
elif violations:
    status = "warn"
elif waived:
    status = "pass_with_exceptions"

report = {
    "schema_version": "1.0.0",
    "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "status": status,
    "mode": mode,
    "scanned_files": scanned_files,
    "violations": violations,
    "waived": waived,
}

report_path.parent.mkdir(parents=True, exist_ok=True)
report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

if violations:
    header = "FAIL" if mode == "strict" else "WARN"
    print(f"{header}: CI warning-bypass integrity violations detected")
    for item in violations:
        print(
            f" - {item['workflow_path']}:{item['line']} [{item['pattern_id']}] {item['match_text']}"
        )
        print(f"   remediation: {item['remediation_hint']}")
else:
    print("PASS: CI warning-bypass integrity check")

if waived:
    print("INFO: warning-bypass exceptions applied")
    for item in waived:
        exception_id = item.get("exception_id") or "<unknown>"
        print(
            f" - {item['workflow_path']}:{item['line']} [{item['pattern_id']}] via exception '{exception_id}'"
        )

if violations and mode == "strict":
    raise SystemExit(1)
PY
