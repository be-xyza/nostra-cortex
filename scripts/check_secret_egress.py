#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


HIGH_CONFIDENCE_PATTERNS = [
    ("private_key_block", re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----")),
    ("openrouter_key", re.compile(r"\bsk-or-v1-[A-Za-z0-9_-]{24,}\b")),
    ("openai_key", re.compile(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}\b")),
    ("anthropic_key", re.compile(r"\bsk-ant-[A-Za-z0-9_-]{24,}\b")),
    ("github_token", re.compile(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{30,}\b")),
    ("github_pat", re.compile(r"\bgithub_pat_[A-Za-z0-9_]{40,}\b")),
    ("bearer_token", re.compile(r"(?i)\bAuthorization\s*[:=]\s*Bearer\s+[A-Za-z0-9._~+/=-]{24,}")),
    (
        "env_secret_assignment",
        re.compile(
            r"(?m)^\s*(?:export\s+)?[A-Z0-9_]*(?:API_KEY|TOKEN|SECRET|PRIVATE_KEY|PASSWORD|CREDENTIAL)[A-Z0-9_]*\s*=\s*[\"']?[A-Za-z0-9._~+/=-]{24,}"
        ),
    ),
]

WARNING_PATTERNS = [
    ("ssn_like", re.compile(r"\b\d{3}-\d{2}-\d{4}\b")),
]

ALLOW_VALUE_RE = re.compile(
    r"(?i)(example|placeholder|redacted|dummy|fixture|test|fake|not-a-secret|sha256|fingerprint)"
)

SKIP_DIR_PARTS = {
    ".git",
    ".cache",
    "target",
    "node_modules",
    ".vite",
    "dist",
    "playwright-report",
}

SKIP_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".webp",
    ".gif",
    ".pdf",
    ".wasm",
    ".zip",
    ".gz",
}


@dataclass
class Finding:
    severity: str
    kind: str
    path: str
    line: int
    fingerprint: str


def repo_root() -> Path:
    script_root = Path(__file__).resolve().parent.parent
    candidates = [Path.cwd(), script_root]
    for candidate in candidates:
        result = subprocess.run(
            ["git", "-C", str(candidate), "rev-parse", "--show-toplevel"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        if result.returncode == 0:
            return Path(result.stdout.strip())
    return script_root


def tracked_files(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return [root / line for line in result.stdout.splitlines() if line.strip()]


def path_files(paths: list[str], root: Path) -> list[Path]:
    files: list[Path] = []
    for value in paths:
        path = Path(value)
        if not path.is_absolute():
            path = root / path
        if path.is_file():
            files.append(path)
        elif path.is_dir():
            for child in path.rglob("*"):
                if child.is_file():
                    files.append(child)
    return files


def should_skip(path: Path) -> bool:
    if path.suffix.lower() in SKIP_SUFFIXES:
        return True
    if any(part in SKIP_DIR_PARTS for part in path.parts):
        return True
    return False


def fingerprint(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()[:12]


def scan_file(path: Path, root: Path) -> list[Finding]:
    if should_skip(path):
        return []
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    findings: list[Finding] = []
    rel = str(path.relative_to(root)) if path.is_relative_to(root) else str(path)
    lines = text.splitlines()
    for line_number, line in enumerate(lines, 1):
        if ALLOW_VALUE_RE.search(line):
            continue
        for kind, pattern in HIGH_CONFIDENCE_PATTERNS:
            match = pattern.search(line)
            if match:
                context = "\n".join(lines[max(0, line_number - 8) : min(len(lines), line_number + 4)])
                if kind == "private_key_block" and (
                    ALLOW_VALUE_RE.search(context)
                    or "valid_internet_identity_session_request" in context
                ):
                    continue
                findings.append(Finding("error", kind, rel, line_number, fingerprint(match.group(0))))
        for kind, pattern in WARNING_PATTERNS:
            match = pattern.search(line)
            if match:
                findings.append(Finding("warning", kind, rel, line_number, fingerprint(match.group(0))))
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Scan governed repo surfaces for secret egress.")
    parser.add_argument("--paths", nargs="*", help="Optional files/directories to scan instead of tracked files.")
    parser.add_argument("--json", action="store_true", help="Emit JSON report.")
    parser.add_argument("--fail-on-warn", action="store_true", help="Treat warning findings as blocking.")
    args = parser.parse_args()

    root = repo_root()
    candidates = path_files(args.paths, root) if args.paths else tracked_files(root)
    findings: list[Finding] = []
    for path in candidates:
        findings.extend(scan_file(path, root))

    errors = [finding for finding in findings if finding.severity == "error"]
    warnings = [finding for finding in findings if finding.severity == "warning"]
    report = {
        "schemaVersion": "1.0.0",
        "scannedFiles": len(candidates),
        "errorCount": len(errors),
        "warningCount": len(warnings),
        "findings": [finding.__dict__ for finding in findings],
    }

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        for finding in findings:
            print(
                f"{finding.severity.upper()}: {finding.kind} at {finding.path}:{finding.line} "
                f"fingerprint={finding.fingerprint}"
            )
        if not findings:
            print("PASS: no secret egress findings")

    if errors or (args.fail_on_warn and warnings):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
