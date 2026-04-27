#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
LIB_DIR = ROOT / "scripts" / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib

REGISTRY_PATH = ROOT / "shared" / "standards" / "semantic_primitives_registry.toml"
BOUNDARY_DOC = ROOT / "docs" / "architecture" / "nostra-cortex-boundary.md"

DEFAULT_SURFACES = [
    "cortex/apps/cortex-web/src",
    "cortex/apps/cortex-web/tests",
    "research/120-nostra-design-language",
    "research/132-eudaemon-alpha-initiative",
    "shared/a2ui",
    "nostra/commons/skills",
    ".codex/skills/nostra-cortex-dev-core",
    ".claude/skills/nostra-cortex-dev-core",
]

TEXT_SUFFIXES = {
    ".css",
    ".html",
    ".js",
    ".json",
    ".jsx",
    ".md",
    ".mdx",
    ".rs",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".yaml",
    ".yml",
}

SKIP_PARTS = {
    ".git",
    ".next",
    "dist",
    "node_modules",
    "target",
    "_archive",
}

ALLOW_NEGATION_RE = re.compile(
    r"\b(no|not|never|without|avoid|forbid|forbidden|deprecated|historical|risk|do not|must not)\b",
    re.IGNORECASE,
)
STRING_RE = re.compile(
    r"""
    (?P<quote>["'`])
    (?P<body>
      (?:\\.|(?! (?P=quote) ).)*?
    )
    (?P=quote)
    """,
    re.VERBOSE | re.DOTALL,
)
AMBIGUOUS_PAIR_RE = re.compile(r"\bNostra/Cortex\b")
NOSTRA_RUNTIME_RE = re.compile(r"\bNostra\s+(runtime|execution|worker|daemon)\b", re.IGNORECASE)
CORTEX_AUTHORITY_RE = re.compile(
    r"\bCortex\s+(platform authority|protocol authority|constitutional|governance authority)\b",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class Issue:
    severity: str
    path: Path
    line: int
    term: str
    message: str


def iter_files(paths: Iterable[str]) -> Iterable[Path]:
    for raw in paths:
        base = resolve_scan_path(raw)
        if not base.exists():
            continue
        if base.is_file():
            if base.suffix in TEXT_SUFFIXES:
                yield base
            continue
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
                continue
            if any(part in SKIP_PARTS for part in path.parts):
                continue
            yield path


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def resolve_scan_path(raw: str) -> Path:
    candidate = Path(raw)
    if candidate.is_absolute():
        return candidate
    return ROOT / candidate


def load_registry() -> list[dict]:
    if not REGISTRY_PATH.exists():
        raise SystemExit(f"FAIL: missing semantic primitive registry {REGISTRY_PATH}")
    payload = tomllib.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    rows = payload.get("term", [])
    if not isinstance(rows, list):
        raise SystemExit(f"FAIL: invalid semantic primitive registry {REGISTRY_PATH}")
    return rows


def canonical_forbidden_terms(rows: list[dict]) -> set[str]:
    terms: set[str] = set()
    for row in rows:
        status = str(row.get("semantic_status", "")).strip()
        reserved_for = str(row.get("reserved_for", "")).strip()
        term = str(row.get("term", "")).strip()
        if term and (status == "deprecated" or reserved_for):
            terms.add(term)
    return terms


def developer_only_terms(rows: list[dict]) -> set[str]:
    terms: set[str] = set()
    for row in rows:
        if str(row.get("surface_scope", "")).strip() == "developer_only":
            term = str(row.get("term", "")).strip()
            if term:
                terms.add(term)
    return terms


def extract_string_literals(path: Path, text: str) -> Iterable[tuple[int, str]]:
    if path.suffix not in {".js", ".jsx", ".ts", ".tsx"}:
        return []
    results: list[tuple[int, str]] = []
    for match in STRING_RE.finditer(text):
        body = match.group("body")
        if "\n" in body and match.group("quote") != "`":
            continue
        results.append((line_number(text, match.start()), body))
    return results


def scan_full_text(path: Path, text: str, forbidden_terms: set[str]) -> list[Issue]:
    issues: list[Issue] = []
    for match in AMBIGUOUS_PAIR_RE.finditer(text):
        issues.append(
            Issue(
                "error",
                path,
                line_number(text, match.start()),
                "Nostra/Cortex",
                "Use explicit layer wording: Nostra platform, Cortex runtime, or Nostra Cortex umbrella.",
            )
        )
    for regex, term, message in (
        (NOSTRA_RUNTIME_RE, "Nostra runtime", "Nostra defines what exists; runtime/execution belongs to Cortex."),
        (
            CORTEX_AUTHORITY_RE,
            "Cortex authority",
            "Cortex executes work; platform/protocol/governance authority belongs to Nostra.",
        ),
    ):
        for match in regex.finditer(text):
            issues.append(Issue("error", path, line_number(text, match.start()), term, message))

    if path.suffix in {".md", ".mdx"}:
        for term in forbidden_terms:
            pattern = re.compile(rf"\b{re.escape(term)}\b", re.IGNORECASE)
            for match in pattern.finditer(text):
                line_start = text.rfind("\n", 0, match.start()) + 1
                line_end = text.find("\n", match.start())
                if line_end == -1:
                    line_end = len(text)
                line = text[line_start:line_end]
                if ALLOW_NEGATION_RE.search(line):
                    continue
                issues.append(
                    Issue(
                        "warn",
                        path,
                        line_number(text, match.start()),
                        term,
                        "Reserved or deprecated primitive appears in prose outside an explicit negation/migration context.",
                    )
                )
    return issues


def scan_string_literals(
    path: Path,
    text: str,
    forbidden_terms: set[str],
    developer_terms: set[str],
) -> list[Issue]:
    issues: list[Issue] = []
    for line, literal in extract_string_literals(path, text):
        normalized = " ".join(literal.split())
        if not normalized:
            continue
        if AMBIGUOUS_PAIR_RE.search(normalized):
            issues.append(
                Issue(
                    "error",
                    path,
                    line,
                    "Nostra/Cortex",
                    "User-facing string literal uses ambiguous slash naming.",
                )
            )
        for term in sorted(forbidden_terms):
            if re.search(rf"\b{re.escape(term)}\b", normalized, re.IGNORECASE):
                if ALLOW_NEGATION_RE.search(normalized):
                    continue
                issues.append(
                    Issue(
                        "warn",
                        path,
                        line,
                        term,
                        "String literal uses a reserved or deprecated semantic primitive.",
                    )
                )
        for term in sorted(developer_terms):
            if re.search(rf"\b{re.escape(term)}\b", normalized, re.IGNORECASE):
                if ALLOW_NEGATION_RE.search(normalized):
                    continue
                issues.append(
                    Issue(
                        "warn",
                        path,
                        line,
                        term,
                        "Developer-only primitive appears in a string literal; confirm this is not user-facing UI copy.",
                    )
                )
    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Check semantic alignment across user-facing and advisory surfaces.")
    parser.add_argument("--strict", action="store_true", help="Fail on warnings as well as errors.")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")
    parser.add_argument("paths", nargs="*", help="Optional repo-relative paths to scan.")
    args = parser.parse_args()

    if not BOUNDARY_DOC.exists():
        print(f"FAIL: missing boundary contract {BOUNDARY_DOC}", file=sys.stderr)
        return 1

    rows = load_registry()
    forbidden_terms = canonical_forbidden_terms(rows)
    developer_terms = developer_only_terms(rows)
    scan_paths = args.paths or DEFAULT_SURFACES

    if args.paths:
        missing_explicit_paths = [raw for raw in args.paths if not resolve_scan_path(raw).exists()]
        if missing_explicit_paths:
            payload = {
                "scanned_files": 0,
                "errors": len(missing_explicit_paths),
                "warnings": 0,
                "issues": [
                    {
                        "severity": "error",
                        "path": raw,
                        "line": 0,
                        "term": "scan_path",
                        "message": "Explicit scan path does not exist.",
                    }
                    for raw in missing_explicit_paths
                ],
            }
            if args.json:
                print(json.dumps(payload, indent=2, sort_keys=True))
            else:
                print("FAIL: semantic alignment surface check")
                for issue in payload["issues"]:
                    print(
                        f" - ERROR: {issue['path']}:0: "
                        "scan_path: Explicit scan path does not exist."
                    )
            return 1

    issues: list[Issue] = []
    scanned = 0
    for path in iter_files(scan_paths):
        scanned += 1
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        issues.extend(scan_full_text(path, text, forbidden_terms))
        issues.extend(scan_string_literals(path, text, forbidden_terms, developer_terms))

    errors = [issue for issue in issues if issue.severity == "error"]
    warnings = [issue for issue in issues if issue.severity == "warn"]
    should_fail = bool(args.strict and issues)

    if args.json:
        payload = {
            "scanned_files": scanned,
            "errors": len(errors),
            "warnings": len(warnings),
            "issues": [
                {
                    "severity": issue.severity,
                    "path": display_path(issue.path),
                    "line": issue.line,
                    "term": issue.term,
                    "message": issue.message,
                }
                for issue in issues
            ],
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        status = "FAIL" if should_fail else ("PASS" if args.strict else "PASS(observe)")
        print(
            f"{status}: semantic alignment surface check "
            f"({scanned} files, {len(errors)} error(s), {len(warnings)} warning(s))"
        )
        for issue in issues[:50]:
            print(
                f" - {issue.severity.upper()}: "
                f"{display_path(issue.path)}:{issue.line}: {issue.term}: {issue.message}"
            )
        if len(issues) > 50:
            print(f" - ... {len(issues) - 50} more issue(s); rerun with --json for full output")

    return 1 if should_fail else 0


if __name__ == "__main__":
    raise SystemExit(main())
