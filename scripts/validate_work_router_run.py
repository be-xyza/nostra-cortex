#!/usr/bin/env python3
"""Validate WorkRouterRunV1 artifacts."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = (
    ROOT
    / "research"
    / "132-eudaemon-alpha-initiative"
    / "schemas"
    / "WorkRouterRunV1.schema.json"
)


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def main(argv: list[str]) -> int:
    paths = [Path(arg) for arg in argv[1:]]
    if not paths:
        paths = sorted((ROOT / "logs" / "work_router" / "runs").glob("*/run.json"))
    if not paths:
        print("No WorkRouterRunV1 artifacts found.", file=sys.stderr)
        return 1
    schema = load_json(SCHEMA_PATH)
    for path in paths:
        resolved = path if path.is_absolute() else ROOT / path
        payload = load_json(resolved)
        jsonschema.validate(payload, schema)
        print(f"PASS: {resolved}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
