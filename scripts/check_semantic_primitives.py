#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LIB_DIR = ROOT / "scripts" / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib

REGISTRY_PATH = ROOT / "shared" / "standards" / "semantic_primitives_registry.toml"
CANONICAL_DOC_GLOBS = [
    "AGENTS.md",
    "nostra/spec.md",
    "research/README.md",
    "docs/reference/README.md",
    "docs/architecture/*.md",
]
CANONICAL_DOC_EXCLUDES = {
    ROOT / "docs" / "architecture" / "semantic-primitives-audit.md",
    ROOT / "docs" / "architecture" / "promotion-migration-rubric.md",
}
REQUIRED_TERMS = {
    "space",
    "workspace",
    "workbench",
    "steward",
    "gallery",
    "catalogue",
    "labs",
    "layout family",
    "contribution graph",
    "notes",
    "tasks",
    "plans",
    "initiatives",
}
ALLOWED_SURFACE_SCOPES = {"labs", "internal", "user_facing", "developer_only"}
ALLOWED_STATUSES = {"experimental", "proposed", "canonical", "deprecated"}
ALLOWED_LAYERS = {"nostra", "cortex"}


def iter_canonical_docs() -> list[Path]:
    docs: list[Path] = []
    for pattern in CANONICAL_DOC_GLOBS:
        for path in ROOT.glob(pattern):
            if path.is_file() and path not in CANONICAL_DOC_EXCLUDES:
                docs.append(path)
    return sorted(set(docs))


def load_registry() -> dict:
    return tomllib.loads(REGISTRY_PATH.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true")
    _ = parser.parse_args()

    if not REGISTRY_PATH.exists():
        print(f"FAIL: missing registry {REGISTRY_PATH}")
        return 1

    registry = load_registry()
    rows = registry.get("term", [])
    failures: list[str] = []

    if not rows:
        failures.append("registry has no term entries")

    seen: set[str] = set()
    terms_by_name: dict[str, dict] = {}
    reserved_terms: list[str] = []
    deprecated_terms: list[str] = []

    for index, row in enumerate(rows, start=1):
        term = str(row.get("term", "")).strip()
        layer = str(row.get("layer", "")).strip()
        scope = str(row.get("surface_scope", "")).strip()
        status = str(row.get("semantic_status", "")).strip()
        owner = str(row.get("owner", "")).strip()
        definition = str(row.get("definition", "")).strip()
        non_definition = str(row.get("non_definition", "")).strip()
        decision_ref = str(row.get("decision_ref", "")).strip()
        replacement = str(row.get("replacement_term", "")).strip()
        reserved_for = str(row.get("reserved_for", "")).strip()

        label = term or f"<row {index}>"
        if not term:
            failures.append(f"row {index}: missing term")
            continue
        normalized = term.lower()
        if normalized in seen:
            failures.append(f"{label}: duplicate term entry")
            continue
        seen.add(normalized)
        terms_by_name[normalized] = row

        if layer not in ALLOWED_LAYERS:
            failures.append(f"{label}: invalid layer '{layer}'")
        if scope not in ALLOWED_SURFACE_SCOPES:
            failures.append(f"{label}: invalid surface_scope '{scope}'")
        if status not in ALLOWED_STATUSES:
            failures.append(f"{label}: invalid semantic_status '{status}'")
        if not owner:
            failures.append(f"{label}: missing owner")
        if not definition:
            failures.append(f"{label}: missing definition")
        if not non_definition:
            failures.append(f"{label}: missing non_definition")
        if not decision_ref:
            failures.append(f"{label}: missing decision_ref")
        if status == "canonical" and replacement:
            failures.append(f"{label}: canonical terms must not declare replacement_term")
        if reserved_for:
            reserved_terms.append(term)
        if status == "deprecated":
            deprecated_terms.append(term)

    missing_required = sorted(REQUIRED_TERMS - set(terms_by_name.keys()))
    for term in missing_required:
        failures.append(f"missing required registry term '{term}'")

    for name, row in terms_by_name.items():
        replacement = str(row.get("replacement_term", "")).strip().lower()
        if replacement and replacement not in terms_by_name:
            failures.append(f"{row['term']}: replacement_term '{replacement}' not found in registry")

    canonical_docs = iter_canonical_docs()
    blocked_terms = set(reserved_terms + deprecated_terms)
    for path in canonical_docs:
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(ROOT)
        for term in blocked_terms:
            pattern = re.compile(rf"\b{re.escape(term)}\b", re.IGNORECASE)
            if pattern.search(text):
                failures.append(f"{term}: reserved/deprecated term appears in canonical doc {rel}")

    if failures:
        print("FAIL: semantic primitive checks failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print(f"PASS: semantic primitive checks ({len(rows)} registered terms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
