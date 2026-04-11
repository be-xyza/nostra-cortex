#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


FRONTMATTER_PATTERN = re.compile(r"^---\n(.*?)\n---", re.S)
FIELD_PATTERN = re.compile(r"^([A-Za-z0-9_-]+):\s*(.+)$")


def parse_frontmatter(text: str) -> dict[str, str]:
    match = FRONTMATTER_PATTERN.search(text)
    if not match:
        return {}
    fields: dict[str, str] = {}
    for raw_line in match.group(1).splitlines():
        line = raw_line.strip()
        if not line:
            continue
        parsed = FIELD_PATTERN.match(line)
        if not parsed:
            continue
        key, value = parsed.groups()
        fields[key] = value.strip().strip('"').strip("'")
    return fields


def validate_skill(path: Path) -> list[str]:
    failures: list[str] = []
    skill_md = path / "SKILL.md"
    if not skill_md.exists():
        return [f"missing SKILL.md: {skill_md}"]
    text = skill_md.read_text(encoding="utf-8")
    frontmatter = parse_frontmatter(text)
    if not frontmatter:
        failures.append("SKILL.md missing frontmatter")
    name = frontmatter.get("name", "")
    if not name:
        failures.append("SKILL.md frontmatter missing name")
    elif name != path.name:
        failures.append(f"frontmatter name {name!r} does not match directory {path.name!r}")
    if "description" not in frontmatter:
        failures.append("SKILL.md frontmatter missing description")
    openai_yaml = path / "agents" / "openai.yaml"
    if not openai_yaml.exists():
        failures.append("missing agents/openai.yaml")
    else:
        yaml_text = openai_yaml.read_text(encoding="utf-8")
        if "display_name:" not in yaml_text:
            failures.append("agents/openai.yaml missing interface.display_name")
        if "short_description:" not in yaml_text:
            failures.append("agents/openai.yaml missing interface.short_description")
    return failures


def validate_workflow(path: Path) -> list[str]:
    failures: list[str] = []
    workflow_md = path / "WORKFLOW.md"
    if not workflow_md.exists():
        return [f"missing WORKFLOW.md: {workflow_md}"]
    text = workflow_md.read_text(encoding="utf-8")
    frontmatter = parse_frontmatter(text)
    if not frontmatter:
        failures.append("WORKFLOW.md missing frontmatter")
    workflow_id = frontmatter.get("id", "")
    if not workflow_id:
        failures.append("WORKFLOW.md frontmatter missing id")
    elif workflow_id != path.name:
        failures.append(f"frontmatter id {workflow_id!r} does not match directory {path.name!r}")
    if "title" not in frontmatter:
        failures.append("WORKFLOW.md frontmatter missing title")
    if "## Self-Improvement" in text:
        failures.append("WORKFLOW.md uses legacy speculative section: Self-Improvement")
    required_headers = [
        "Purpose",
        "Triggers",
        "Inputs",
        "Steps",
        "Outputs",
        "Observability",
        "Required Checks",
    ]
    for header in required_headers:
        if f"## {header}" not in text:
            failures.append(f"WORKFLOW.md missing section: {header}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description="Quick validation for repo-managed registry assets")
    parser.add_argument("--kind", choices=["skill", "workflow"], required=True)
    parser.add_argument("path")
    args = parser.parse_args()

    path = Path(args.path)
    if not path.exists():
        print(f"FAIL: path does not exist: {path}")
        return 1
    if not path.is_dir():
        print(f"FAIL: expected directory asset: {path}")
        return 1

    if args.kind == "skill":
        failures = validate_skill(path)
    else:
        failures = validate_workflow(path)

    if failures:
        print("FAIL: validation failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print(f"PASS: {args.kind} asset valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
