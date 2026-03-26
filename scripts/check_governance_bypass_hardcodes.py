#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from fnmatch import fnmatch
from pathlib import Path
from typing import List, Optional, Tuple

LIB_DIR = Path(__file__).resolve().parent / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


@dataclass
class Hit:
    rule_id: str
    failure_class: str
    path: str
    line: int
    pattern: str
    excerpt: str
    status: str
    exception_id: Optional[str] = None


def now_utc() -> datetime:
    return datetime.now(timezone.utc)


def parse_iso_date(text: Optional[str]) -> Optional[datetime]:
    if not text:
        return None
    t = text.strip()
    if not t:
        return None
    try:
        if len(t) == 10:
            return datetime.fromisoformat(t + "T00:00:00+00:00")
        return datetime.fromisoformat(t.replace("Z", "+00:00"))
    except ValueError:
        return None


def tracked_files(root: Path) -> list[Path]:
    try:
        out = subprocess.check_output(["git", "-C", str(root), "ls-files"], text=True)
        return [root / line.strip() for line in out.splitlines() if line.strip()]
    except Exception:
        files: list[Path] = []
        for p in root.rglob("*"):
            if p.is_file():
                files.append(p)
        return files


def should_exclude(path: str) -> bool:
    excluded_tokens = (
        "/_archive/",
        "/.git/",
        "/node_modules/",
        "/target/",
        "/logs/",
        "/research/",
    )
    return any(tok in f"/{path}" for tok in excluded_tokens)


def load_exceptions(path: Path) -> Tuple[List[dict], List[str]]:
    failures: List[str] = []
    if not path.exists():
        return [], [f"missing exceptions file: {path}"]

    data = json.loads(path.read_text(encoding="utf-8"))
    rows = data.get("exceptions", [])
    if not isinstance(rows, list):
        return [], ["exceptions payload must be a list"]

    required = ["id", "rule_id", "path", "pattern", "owner", "reason", "expires_at", "enabled"]
    validated: List[dict] = []
    now = now_utc()

    for row in rows:
        if not isinstance(row, dict):
            failures.append("exception row is not an object")
            continue
        missing = [k for k in required if not str(row.get(k, "")).strip()]
        if missing:
            failures.append(f"exception {row.get('id', '<unknown>')} missing required fields: {', '.join(missing)}")
            continue

        exp = parse_iso_date(str(row.get("expires_at")))
        if exp is None:
            failures.append(f"exception {row['id']} has invalid expires_at")
            continue
        if exp < now:
            failures.append(f"exception {row['id']} expired on {exp.date().isoformat()}")
            continue

        opened = parse_iso_date(str(row.get("opened_at", "")))
        if opened is not None and now - opened > timedelta(days=14):
            ref = str(row.get("follow_up_decision_ref", "")).strip()
            if not ref:
                failures.append(
                    f"exception {row['id']} older than 14 days must include follow_up_decision_ref"
                )
                continue

        validated.append(row)

    return validated, failures


def exception_for(exceptions: List[dict], rel_path: str, pattern: str, rule_id: str) -> Optional[dict]:
    for row in exceptions:
        if not bool(row.get("enabled", False)):
            continue
        if str(row.get("rule_id")) != rule_id:
            continue
        if str(row.get("path")) != rel_path:
            continue
        if str(row.get("pattern")) != pattern:
            continue
        return row
    return None


def main() -> int:
    root = Path(os.environ.get("NOSTRA_WORKSPACE_ROOT", Path(__file__).resolve().parents[1]))
    contract_path = root / "shared" / "standards" / "dynamic_source_contract.toml"
    if not contract_path.exists():
        print(f"FAIL: missing contract file: {contract_path}")
        return 1

    contract = tomllib.loads(contract_path.read_text(encoding="utf-8"))
    report_rel = contract.get("report_path", "logs/alignment/dynamic_config_contract_latest.json")
    report_path = root / str(report_rel)
    report_path.parent.mkdir(parents=True, exist_ok=True)

    exceptions_path = root / str(contract.get("exceptions_path", "shared/standards/dynamic_source_exceptions.json"))
    exceptions, exception_failures = load_exceptions(exceptions_path)

    files = tracked_files(root)
    governed_values = contract.get("governed_values", [])
    hits: List[Hit] = []

    for rule in governed_values:
        rule_id = str(rule.get("id", "")).strip()
        failure_class = str(rule.get("failure_class", "hardcoded_policy_override")).strip()
        patterns = [str(p) for p in rule.get("forbidden_literal_patterns", [])]
        scopes = [str(s) for s in rule.get("path_scopes", [])]
        if not rule_id or not patterns or not scopes:
            continue

        for fp in files:
            rel = fp.relative_to(root).as_posix()
            if should_exclude(rel):
                continue
            if not any(fnmatch(rel, scope) for scope in scopes):
                continue

            try:
                text = fp.read_text(encoding="utf-8")
            except Exception:
                continue

            lines = text.splitlines()
            for patt in patterns:
                try:
                    reg = re.compile(patt)
                except re.error:
                    continue

                for idx, line in enumerate(lines, start=1):
                    m = reg.search(line)
                    if not m:
                        continue
                    ex = exception_for(exceptions, rel, patt, rule_id)
                    hits.append(
                        Hit(
                            rule_id=rule_id,
                            failure_class=failure_class,
                            path=rel,
                            line=idx,
                            pattern=patt,
                            excerpt=line.strip()[:240],
                            status="exception" if ex else "fail",
                            exception_id=(str(ex.get("id")) if ex else None),
                        )
                    )

    failing_hits = [h for h in hits if h.status == "fail"]

    payload = {
        "schema_version": "1.0.0",
        "generated_at": now_utc().replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "contract_path": str(contract_path.relative_to(root)),
        "exceptions_path": str(exceptions_path.relative_to(root)),
        "overall_verdict": "fail" if (exception_failures or failing_hits) else "pass",
        "counts": {
            "total_hits": len(hits),
            "failing_hits": len(failing_hits),
            "excepted_hits": len([h for h in hits if h.status == "exception"]),
            "exception_validation_failures": len(exception_failures),
        },
        "exception_validation_failures": exception_failures,
        "hits": [h.__dict__ for h in hits],
    }

    report_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if exception_failures:
        print("FAIL: dynamic source exception validation failed")
        for msg in exception_failures:
            print(f" - {msg}")

    if failing_hits:
        print("FAIL: dynamic source contract violations detected")
        for hit in failing_hits[:80]:
            print(f" - {hit.rule_id}:{hit.failure_class} {hit.path}:{hit.line} pattern={hit.pattern}")

    if not exception_failures and not failing_hits:
        print("PASS: dynamic source contract checks")

    print(f"report: {report_path}")
    return 1 if (exception_failures or failing_hits) else 0


if __name__ == "__main__":
    raise SystemExit(main())
