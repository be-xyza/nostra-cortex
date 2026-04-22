#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
STRICT=false
if [[ "${1:-}" == "--strict" ]]; then
  STRICT=true
fi

python3 - "$ROOT_DIR" "$STRICT" <<'PY'
import json
import re
import sys
from datetime import date
from pathlib import Path

root = Path(sys.argv[1])
strict = sys.argv[2].lower() == "true"

sources = [
    root / "AGENTS.md",
    root / ".github" / "workflows" / "test-suite.yml",
]

exceptions_path = root / "shared" / "standards" / "alignment_contract_exceptions.json"
exception_map: dict[str, dict] = {}
if exceptions_path.exists():
    try:
        payload = json.loads(exceptions_path.read_text(encoding="utf-8"))
        for row in payload.get("exceptions", []):
            path = str(
                row.get("script_path")
                or row.get("command_target")
                or row.get("path")
                or ""
            ).strip()
            if path:
                exception_map[path] = row
    except Exception as exc:
        print(f"WARN: could not parse exceptions file: {exc}")

pattern = re.compile(r"(?:nostra/)?scripts/[A-Za-z0-9_./-]+")
references: set[str] = set()
for source in sources:
    if not source.exists():
        continue
    raw_refs = pattern.findall(source.read_text(encoding="utf-8"))
    for ref in raw_refs:
        references.add(ref.replace("\\.", ".").replace("\\", ""))

missing = []
for ref in sorted(references):
    candidates = [ref]
    if not ref.endswith(".sh"):
        candidates.append(f"{ref}.sh")
    if not ref.endswith(".py"):
        candidates.append(f"{ref}.py")
    resolved = any((root / candidate).exists() for candidate in candidates)
    if resolved:
        continue
    exception = exception_map.get(ref)
    if exception:
        expiry_raw = str(exception.get("expires_at") or exception.get("expires") or "").strip()
        try:
            expiry = date.fromisoformat(expiry_raw)
        except Exception:
            expiry = None
        if expiry is not None and expiry >= date.today():
            continue
    missing.append(ref)

if missing:
    level = "FAIL" if strict else "WARN"
    print(f"{level}: missing gate-surface script references")
    for ref in missing:
        print(f" - {ref}")
    raise SystemExit(1 if strict else 0)

print("PASS: gate-surface script references are resolvable (or exception-allowed)")
PY
