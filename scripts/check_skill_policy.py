#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from fnmatch import fnmatch
from pathlib import Path
from typing import List, Optional, Tuple

LIB_DIR = Path(__file__).resolve().parent / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


VALID_TIERS = {"required", "forbidden", "advisory"}
VALID_STATUSES = {"active", "quarantined", "deprecated"}
NAME_PATTERN = re.compile(r"^[a-z0-9][a-z0-9-]{1,63}$")
FRONTMATTER_PATTERN = re.compile(r"^---\n(.*?)\n---", re.S)
USING_SKILL_REF_PATTERN = re.compile(r"using-skill:([a-z0-9-]+)")
MARKDOWN_REF_PATTERN = re.compile(r"`([^`]+\.md)`|([A-Za-z0-9._/-]+\.md)")


@dataclass
class Hit:
    rule_id: str
    tier: str
    failure_mode: str
    skill_id: str
    path: str
    line: int
    excerpt: str
    status: str
    disposition_id: Optional[str] = None


def now_utc() -> datetime:
    return datetime.now(timezone.utc)


def parse_iso_datetime(text: Optional[str]) -> Optional[datetime]:
    if not text:
        return None
    raw = text.strip()
    if not raw:
        return None
    try:
        if len(raw) == 10:
            return datetime.fromisoformat(raw + "T00:00:00+00:00")
        return datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError:
        return None


def repo_root() -> Path:
    return Path(os.environ.get("NOSTRA_WORKSPACE_ROOT", Path(__file__).resolve().parents[1]))


def resolve_path(root: Path, raw: str, env_override: Optional[str] = None) -> Path:
    if env_override:
        override = os.environ.get(env_override, "").strip()
        if override:
            raw = override
    expanded = os.path.expandvars(raw)
    expanded = os.path.expanduser(expanded)
    p = Path(expanded)
    return p if p.is_absolute() else root / p


def line_number(lines: list[str], needle: str, default: int = 1) -> int:
    for idx, line in enumerate(lines, start=1):
        if needle in line:
            return idx
    return default


def parse_frontmatter(text: str) -> dict[str, str]:
    match = FRONTMATTER_PATTERN.search(text)
    if not match:
        return {}
    frontmatter: dict[str, str] = {}
    for raw_line in match.group(1).splitlines():
        line = raw_line.strip()
        if not line or ":" not in line:
            continue
        key, value = line.split(":", 1)
        frontmatter[key.strip()] = value.strip().strip('"').strip("'")
    return frontmatter


def markdown_refs(text: str) -> set[str]:
    refs: set[str] = set()
    for match in MARKDOWN_REF_PATTERN.finditer(text):
        candidate = (match.group(1) or match.group(2) or "").strip()
        if not candidate:
            continue
        if candidate.startswith(("http://", "https://", "file://")):
            continue
        refs.add(candidate)
    return refs


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_contract(contract: dict) -> list[str]:
    failures: List[str] = []
    if not contract.get("rule"):
        failures.append("contract has no [[rule]] entries")

    for row in contract.get("rule", []):
        rule_id = str(row.get("id", "")).strip() or "<unknown>"
        tier = str(row.get("tier", "")).strip()
        check = str(row.get("check", "")).strip()
        if tier not in VALID_TIERS:
            failures.append(f"{rule_id}: invalid tier {tier}")
        if not check:
            failures.append(f"{rule_id}: missing check")
        if check == "regex":
            patterns = row.get("patterns", [])
            if not isinstance(patterns, list) or not patterns:
                failures.append(f"{rule_id}: regex check requires non-empty patterns")
            else:
                for patt in patterns:
                    try:
                        re.compile(str(patt))
                    except re.error as exc:
                        failures.append(f"{rule_id}: invalid regex pattern {patt!r}: {exc}")
        if check == "word_count_max":
            max_words = int(row.get("max_words", 0))
            if max_words <= 0:
                failures.append(f"{rule_id}: word_count_max requires max_words > 0")
    return failures


def validate_dispositions(path: Path) -> Tuple[List[dict], List[str]]:
    failures: List[str] = []
    if not path.exists():
        return [], [f"missing dispositions file: {path}"]

    data = load_json(path)
    rows = data.get("entries", [])
    if not isinstance(rows, list):
        return [], ["dispositions entries must be a list"]

    now = now_utc()
    validated: List[dict] = []
    for row in rows:
        if not isinstance(row, dict):
            failures.append("disposition row must be an object")
            continue

        row_id = str(row.get("id", "")).strip()
        skill_id = str(row.get("skill_id", "")).strip()
        status = str(row.get("status", "")).strip()
        owner = str(row.get("owner", "")).strip()
        reason = str(row.get("reason", "")).strip()
        decision_ref = str(row.get("decision_ref", "")).strip()
        rule_ids = row.get("rule_ids", [])
        enabled = bool(row.get("enabled", False))

        if not row_id:
            failures.append("disposition missing id")
            continue
        if not skill_id:
            failures.append(f"{row_id}: missing skill_id")
            continue
        if status not in VALID_STATUSES:
            failures.append(f"{row_id}: invalid status {status}")
            continue
        if not owner:
            failures.append(f"{row_id}: missing owner")
            continue
        if not reason:
            failures.append(f"{row_id}: missing reason")
            continue
        if not isinstance(rule_ids, list) or not rule_ids:
            failures.append(f"{row_id}: rule_ids must be a non-empty list")
            continue
        if not decision_ref and status in {"quarantined", "deprecated"}:
            failures.append(f"{row_id}: {status} entry requires decision_ref")
            continue

        expiry = parse_iso_datetime(str(row.get("expires_at", "")))
        if expiry is None:
            failures.append(f"{row_id}: invalid expires_at")
            continue
        if enabled and expiry < now:
            failures.append(f"{row_id}: expired on {expiry.date().isoformat()}")
            continue

        if status == "deprecated":
            replacement = str(row.get("replacement_skill", "")).strip()
            if not replacement:
                failures.append(f"{row_id}: deprecated entry missing replacement_skill")
                continue

        validated.append(row)
    return validated, failures


def matching_disposition(dispositions: List[dict], skill_id: str, rule_id: str) -> Optional[dict]:
    for row in dispositions:
        if not bool(row.get("enabled", False)):
            continue
        if str(row.get("skill_id")) != skill_id:
            continue
        if rule_id not in [str(v) for v in row.get("rule_ids", [])]:
            continue
        return row
    return None


def make_hit(
    *,
    rule_id: str,
    tier: str,
    failure_mode: str,
    skill_id: str,
    rel_path: str,
    line: int,
    excerpt: str,
    disposition: Optional[dict],
) -> Hit:
    if disposition:
        status = str(disposition.get("status"))
        return Hit(
            rule_id=rule_id,
            tier=tier,
            failure_mode=failure_mode,
            skill_id=skill_id,
            path=rel_path,
            line=line,
            excerpt=excerpt.strip()[:240],
            status=status,
            disposition_id=str(disposition.get("id")),
        )
    if tier == "advisory":
        status = "advisory"
    else:
        status = "fail"
    return Hit(
        rule_id=rule_id,
        tier=tier,
        failure_mode=failure_mode,
        skill_id=skill_id,
        path=rel_path,
        line=line,
        excerpt=excerpt.strip()[:240],
        status=status,
    )


def rule_applies(rule: dict, rel_path: str) -> bool:
    scopes = rule.get("path_scopes", [])
    if not scopes:
        return True
    return any(fnmatch(rel_path, str(scope)) for scope in scopes)


def run_checks(contract: dict, skills_root: Path, dispositions: list[dict]) -> tuple[list[Hit], list[str]]:
    failures: List[str] = []
    hits: list[Hit] = []

    source = contract.get("source", {})
    skill_glob = str(source.get("skill_glob", "*/SKILL.md"))
    skill_files = sorted(skills_root.glob(skill_glob))
    if not skill_files:
        failures.append(f"no skill files matched {skill_glob} under {skills_root}")
        return hits, failures

    skill_ids = {p.parent.name for p in skill_files}
    rules = contract.get("rule", [])

    for skill_file in skill_files:
        skill_id = skill_file.parent.name
        text = skill_file.read_text(encoding="utf-8")
        lines = text.splitlines()
        rel_path = f"{skill_id}/SKILL.md"
        frontmatter = parse_frontmatter(text)
        refs = sorted(set(USING_SKILL_REF_PATTERN.findall(text)))
        md_refs = sorted(markdown_refs(text))

        for rule in rules:
            rule_id = str(rule.get("id"))
            tier = str(rule.get("tier"))
            check = str(rule.get("check"))
            failure_mode = str(rule.get("failure_mode", "policy_violation"))

            if not rule_applies(rule, rel_path):
                continue

            if check == "skill_dependency_exists":
                for ref in refs:
                    if ref in skill_ids:
                        continue
                    disposition = matching_disposition(dispositions, skill_id, rule_id)
                    hits.append(
                        make_hit(
                            rule_id=rule_id,
                            tier=tier,
                            failure_mode=failure_mode,
                            skill_id=skill_id,
                            rel_path=rel_path,
                            line=line_number(lines, f"using-skill:{ref}"),
                            excerpt=f"missing using-skill dependency: {ref}",
                            disposition=disposition,
                        )
                    )

            elif check == "referenced_markdown_paths_exist":
                for ref in md_refs:
                    ref_path = Path(ref)
                    if ref_path.is_absolute():
                        exists = ref_path.exists()
                    else:
                        head = ref_path.parts[0] if ref_path.parts else ""
                        if ref.startswith(("./", "../")):
                            candidate = (skill_file.parent / ref_path).resolve()
                        elif head in skill_ids:
                            candidate = (skills_root / ref_path).resolve()
                        else:
                            # Ignore non-local markdown mentions to avoid noisy false positives.
                            continue
                        exists = candidate.exists()
                    if exists:
                        continue
                    disposition = matching_disposition(dispositions, skill_id, rule_id)
                    hits.append(
                        make_hit(
                            rule_id=rule_id,
                            tier=tier,
                            failure_mode=failure_mode,
                            skill_id=skill_id,
                            rel_path=rel_path,
                            line=line_number(lines, ref),
                            excerpt=f"missing markdown reference: {ref}",
                            disposition=disposition,
                        )
                    )

            elif check == "frontmatter_name_hyphen_case":
                name = str(frontmatter.get("name", "")).strip()
                if name and NAME_PATTERN.fullmatch(name):
                    continue
                disposition = matching_disposition(dispositions, skill_id, rule_id)
                hits.append(
                    make_hit(
                        rule_id=rule_id,
                        tier=tier,
                        failure_mode=failure_mode,
                        skill_id=skill_id,
                        rel_path=rel_path,
                        line=line_number(lines, "name:", default=1),
                        excerpt=f"invalid frontmatter name: {name or '<missing>'}",
                        disposition=disposition,
                    )
                )

            elif check == "regex":
                patterns = [str(p) for p in rule.get("patterns", [])]
                for patt in patterns:
                    reg = re.compile(patt)
                    for idx, line in enumerate(lines, start=1):
                        if not reg.search(line):
                            continue
                        disposition = matching_disposition(dispositions, skill_id, rule_id)
                        hits.append(
                            make_hit(
                                rule_id=rule_id,
                                tier=tier,
                                failure_mode=failure_mode,
                                skill_id=skill_id,
                                rel_path=rel_path,
                                line=idx,
                                excerpt=line,
                                disposition=disposition,
                            )
                        )

            elif check == "word_count_max":
                max_words = int(rule.get("max_words", 0))
                words = len(text.split())
                if words <= max_words:
                    continue
                disposition = matching_disposition(dispositions, skill_id, rule_id)
                hits.append(
                    make_hit(
                        rule_id=rule_id,
                        tier=tier,
                        failure_mode=failure_mode,
                        skill_id=skill_id,
                        rel_path=rel_path,
                        line=1,
                        excerpt=f"word_count={words} exceeds max_words={max_words}",
                        disposition=disposition,
                    )
                )
            else:
                failures.append(f"{rule_id}: unsupported check type {check}")

    return hits, failures


def main() -> int:
    parser = argparse.ArgumentParser(description="Check repo-managed skill governance policy")
    parser.add_argument("--mode", choices=["observe", "softgate", "hardgate"], default=None)
    parser.add_argument(
        "--policy-path",
        default="shared/standards/skill_policy.toml",
        help="policy contract path relative to workspace root",
    )
    args = parser.parse_args()

    root = repo_root()
    policy_path = resolve_path(root, args.policy_path)
    if not policy_path.exists():
        print(f"FAIL: missing policy contract: {policy_path}")
        return 1

    contract = tomllib.loads(policy_path.read_text(encoding="utf-8"))
    contract_failures = validate_contract(contract)

    source = contract.get("source", {})
    skills_root = resolve_path(
        root,
        str(source.get("skills_root", "nostra/commons/skills")),
        str(source.get("env_override", "")).strip() or None,
    )

    dispositions_path = resolve_path(root, str(contract.get("dispositions_path")))
    report_path = resolve_path(root, str(contract.get("report_path")))
    report_path.parent.mkdir(parents=True, exist_ok=True)

    dispositions, disposition_failures = validate_dispositions(dispositions_path)
    hits, runtime_failures = run_checks(contract, skills_root, dispositions)

    mode = args.mode or str(contract.get("policy", {}).get("default_mode", "observe")).strip()
    block_advisory = bool(contract.get("policy", {}).get("hardgate_blocks_advisory", False))

    required_fails = [h for h in hits if h.tier == "required" and h.status == "fail"]
    forbidden_fails = [h for h in hits if h.tier == "forbidden" and h.status == "fail"]
    advisory_fails = [h for h in hits if h.tier == "advisory" and h.status in {"advisory", "fail"}]

    governance_failures = contract_failures + disposition_failures + runtime_failures
    blocking_failures = len(required_fails) + len(forbidden_fails)
    if mode == "hardgate" and block_advisory:
        blocking_failures += len(advisory_fails)

    should_fail = bool(governance_failures) or (mode in {"softgate", "hardgate"} and blocking_failures > 0)
    if mode == "observe" and governance_failures:
        should_fail = True

    overall = "fail" if should_fail else ("warn" if (required_fails or forbidden_fails or advisory_fails) else "pass")
    payload = {
        "schema_version": "1.0.0",
        "generated_at": now_utc().replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "mode": mode,
        "policy_path": str(policy_path.relative_to(root)),
        "skills_root": str(skills_root),
        "dispositions_path": str(dispositions_path.relative_to(root)),
        "overall_verdict": overall,
        "counts": {
            "hits_total": len(hits),
            "required_failures": len(required_fails),
            "forbidden_failures": len(forbidden_fails),
            "advisory_hits": len(advisory_fails),
            "quarantined_hits": len([h for h in hits if h.status == "quarantined"]),
            "deprecated_hits": len([h for h in hits if h.status == "deprecated"]),
            "governance_failures": len(governance_failures),
        },
        "governance_failures": governance_failures,
        "hits": [asdict(h) for h in hits],
    }
    report_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if governance_failures:
        print("FAIL: skill policy governance errors")
        for msg in governance_failures:
            print(f" - {msg}")
    if required_fails or forbidden_fails:
        print("WARN: skill policy violations detected")
        for hit in required_fails + forbidden_fails:
            print(f" - {hit.rule_id}:{hit.failure_mode} {hit.path}:{hit.line} [{hit.skill_id}]")
    if advisory_fails:
        print(f"WARN: advisory bloat/quality findings: {len(advisory_fails)}")
    if not governance_failures and not hits:
        print("PASS: skill policy checks")

    print(f"report: {report_path}")
    return 1 if should_fail else 0


if __name__ == "__main__":
    raise SystemExit(main())
