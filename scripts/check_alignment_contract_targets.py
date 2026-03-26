#!/usr/bin/env python3
from __future__ import annotations

import json
import shlex
import sys
from datetime import date, datetime
from pathlib import Path
from typing import Optional

ROOT = Path(__file__).resolve().parents[1]
LIB_DIR = ROOT / "scripts" / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


def parse_target(command: str) -> Optional[str]:
    parts = shlex.split(command)
    if len(parts) < 2:
        return None
    if parts[0] in {"python", "python3", "bash", "sh"}:
        return parts[1]
    return parts[0]


def parse_expiry(value: Optional[str]) -> Optional[date]:
    if not value:
        return None
    text = value.strip()
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


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: check_alignment_contract_targets.py <root> <contract_path>", file=sys.stderr)
        return 2

    root = Path(sys.argv[1])
    contract_path = Path(sys.argv[2])

    if not contract_path.exists():
        print(f"FAIL: missing contract file: {contract_path}")
        return 1

    contract = tomllib.loads(contract_path.read_text(encoding="utf-8"))
    rules = contract.get("rule", [])
    exceptions_path = root / str(contract.get("exceptions_path", "shared/standards/alignment_contract_exceptions.json"))

    exceptions_by_id: dict[str, dict] = {}
    if exceptions_path.exists():
        data = json.loads(exceptions_path.read_text(encoding="utf-8"))
        for row in data.get("exceptions", []):
            ref = str(row.get("id", "")).strip()
            if ref:
                exceptions_by_id[ref] = row

    failures: list[str] = []
    observed = 0
    for rule in rules:
        if not bool(rule.get("enabled", True)):
            continue

        observed += 1
        rule_id = str(rule.get("id", "")).strip() or "<unknown>"
        command = str(rule.get("command", "")).strip()
        if not command:
            failures.append(f"{rule_id}: missing command")
            continue

        expiry = parse_expiry(rule.get("expires_at"))
        if expiry and expiry < date.today():
            failures.append(f"{rule_id}: rule expires_at is in the past ({expiry.isoformat()})")

        target = parse_target(command)
        if target is None:
            failures.append(f"{rule_id}: cannot parse executable target from command '{command}'")
            continue

        target_path = root / target
        if target_path.exists():
            continue

        exception_ref = str(rule.get("exception_ref", "")).strip()
        if not exception_ref:
            failures.append(f"{rule_id}: missing command target '{target}' and no exception_ref")
            continue

        exception = exceptions_by_id.get(exception_ref)
        if exception is None:
            failures.append(f"{rule_id}: exception_ref '{exception_ref}' not found in {exceptions_path}")
            continue

        if not bool(exception.get("enabled", True)):
            failures.append(f"{rule_id}: exception '{exception_ref}' is disabled")
            continue

        exception_target = str(exception.get("command_target", "")).strip()
        if exception_target and exception_target != target:
            failures.append(
                f"{rule_id}: exception '{exception_ref}' target mismatch expected={target} actual={exception_target}"
            )
            continue

        expires_at = parse_expiry(exception.get("expires_at"))
        if expires_at is None:
            failures.append(f"{rule_id}: exception '{exception_ref}' missing/invalid expires_at")
            continue
        if expires_at < date.today():
            failures.append(
                f"{rule_id}: exception '{exception_ref}' expired on {expires_at.isoformat()} for missing target '{target}'"
            )

    if failures:
        print("FAIL: alignment contract target check failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print(f"PASS: alignment contract target check ({observed} enabled rules)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
