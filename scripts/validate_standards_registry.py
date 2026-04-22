#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

LIB_DIR = Path(__file__).resolve().parent / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


ROOT_SECTIONS = ("documents", "contracts", "groups", "schemas", "artifacts")
ENTRY_KEYS = {
    "id",
    "title",
    "path",
    "purpose",
    "owner",
    "aliases",
    "known_consumers",
    "initiative_refs",
    "status",
}
VALID_STATUS = {"active", "draft", "legacy"}


def load_toml(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def collect_delegate_entries(
    root: Path, delegate_path: Path
) -> tuple[set[str], set[str], set[str]]:
    delegate = load_toml(delegate_path)
    delegate_ids = {
        str(entry["id"]).strip()
        for section in ("schemas", "artifacts")
        for entry in delegate.get(section, [])
        if isinstance(entry, dict) and "id" in entry
    }
    schema_paths = {
        str(Path(entry["path"]))
        for entry in delegate.get("schemas", [])
        if isinstance(entry, dict) and "path" in entry
    }
    artifact_paths = {
        str(Path(entry["path"]))
        for entry in delegate.get("artifacts", [])
        if isinstance(entry, dict) and "path" in entry
    }
    return delegate_ids, schema_paths, artifact_paths


def resolve_root_and_registry(path: Path) -> tuple[Path, Path]:
    resolved = path.resolve()
    if resolved.is_file():
        if resolved.name != "standards_registry.toml":
            raise ValueError(f"unsupported registry path: {resolved}")
        return resolved.parents[2], resolved
    return resolved, resolved / "shared/standards/standards_registry.toml"


def validate_registry(root: Path) -> list[str]:
    try:
        root, registry_path = resolve_root_and_registry(root)
    except ValueError as exc:
        return [str(exc)]
    if not registry_path.exists():
        return [f"missing standards registry: {registry_path}"]

    registry = load_toml(registry_path)
    failures: list[str] = []
    ids: set[str] = set()
    registered_schema_paths: set[str] = set()
    registered_artifact_paths: set[str] = set()

    def validate_entries(entries: list[dict[str, Any]], section: str) -> None:
        nonlocal failures
        for entry in entries:
            missing = sorted(ENTRY_KEYS - entry.keys())
            if missing:
                failures.append(
                    f"{section} {entry.get('id', '<unknown>')}: missing keys {', '.join(missing)}"
                )
                continue
            entry_id = str(entry["id"]).strip()
            entry_path = str(entry["path"]).strip()
            status = str(entry["status"]).strip()
            if not entry_id:
                failures.append(f"{section}: empty id")
            elif entry_id in ids:
                failures.append(f"duplicate standards registry id: {entry_id}")
            else:
                ids.add(entry_id)

            if not entry_path:
                failures.append(f"{entry_id}: empty path")
            else:
                path = root / entry_path
                if not path.exists():
                    failures.append(f"{entry_id}: missing path on disk: {entry_path}")

            aliases = entry["aliases"]
            known_consumers = entry["known_consumers"]
            initiative_refs = entry["initiative_refs"]
            if not isinstance(aliases, list) or not aliases:
                failures.append(f"{entry_id}: aliases must be a non-empty list")
            if not isinstance(known_consumers, list):
                failures.append(f"{entry_id}: known_consumers must be a list")
            if not isinstance(initiative_refs, list) or not initiative_refs:
                failures.append(f"{entry_id}: initiative_refs must be a non-empty list")
            if status not in VALID_STATUS:
                failures.append(f"{entry_id}: invalid status {status}")

            if section == "schemas":
                registered_schema_paths.add(entry_path)
            if section == "artifacts":
                registered_artifact_paths.add(entry_path)

    for section in ROOT_SECTIONS:
        validate_entries(registry.get(section, []), section)

    for group in registry.get("groups", []):
        delegate = group.get("registry_delegate")
        if not delegate:
            continue
        delegate_path = root / delegate
        if not delegate_path.exists():
            failures.append(f"{group['id']}: missing registry delegate {delegate}")
            continue
        delegate_ids, delegate_schemas, delegate_artifacts = collect_delegate_entries(
            root, delegate_path
        )
        ids |= delegate_ids
        registered_schema_paths |= delegate_schemas
        registered_artifact_paths |= delegate_artifacts

    actual_schema_paths = {
        str(path.relative_to(root))
        for path in (root / "shared/standards").rglob("*.schema.json")
        if "_archive" not in path.parts
    }
    actual_artifact_paths = {
        str(path.relative_to(root))
        for path in (root / "shared/standards").rglob("*.json")
        if "_archive" not in path.parts and not path.name.endswith(".schema.json")
    }

    missing_schemas = sorted(actual_schema_paths - registered_schema_paths)
    missing_artifacts = sorted(actual_artifact_paths - registered_artifact_paths)
    if missing_schemas:
        failures.append(
            "unregistered schema paths: " + ", ".join(missing_schemas)
        )
    if missing_artifacts:
        failures.append(
            "unregistered artifact paths: " + ", ".join(missing_artifacts)
        )

    context_path = root / "shared/standards/schema_guided_extraction_context.json"
    if context_path.exists():
        context = json.loads(context_path.read_text(encoding="utf-8"))
        for item in context.get("contexts", []):
            for schema_id in item.get("schema_ids", []):
                if schema_id not in ids:
                    failures.append(
                        f"schema_guided_extraction_context references unknown schema id: {schema_id}"
                    )

    return failures


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    root = args.root.resolve()
    failures = validate_registry(root)
    if failures:
        print("FAIL: standards registry validation failed")
        for failure in failures:
            print(f" - {failure}")
        raise SystemExit(1)
    print("PASS: standards registry covers shared/standards schemas, artifacts, and delegated registries")


if __name__ == "__main__":
    main()
