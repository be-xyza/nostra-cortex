#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Optional, Set

LIB_DIR = Path(__file__).resolve().parent / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


def repo_root() -> Path:
    return Path(os.environ.get("NOSTRA_WORKSPACE_ROOT", Path(__file__).resolve().parents[1]))


def sha256_dir(path: Path) -> str:
    hasher = hashlib.sha256()
    for fp in sorted(p for p in path.rglob("*") if p.is_file()):
        rel = fp.relative_to(path).as_posix()
        if rel.startswith(".git/"):
            continue
        hasher.update(rel.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(fp.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()


def load_registry(path: Path) -> dict:
    if not path.exists():
        raise SystemExit(f"FAIL: missing registry: {path}")
    reg = tomllib.loads(path.read_text(encoding="utf-8"))

    local_path = path.with_name("registry.local.toml")
    if local_path.exists():
        try:
            local_reg = tomllib.loads(local_path.read_text(encoding="utf-8"))
            for key in ["source", "ide", "skill"]:
                if key in local_reg:
                    reg.setdefault(key, []).extend(local_reg[key])
            print(f"INFO: Loaded local registry overlay from {local_path.name}")
        except Exception as exc:
            print(f"WARN: Failed to parse {local_path.name}: {exc}")

    return reg


@dataclass
class SyncRow:
    skill_id: str
    source: Path
    target_ide: str
    target: Path
    action: str
    source_digest: str
    target_digest: Optional[str]


def resolve_path(root: Path, raw: str) -> Path:
    expanded = os.path.expandvars(raw)
    expanded = os.path.expanduser(expanded)
    p = Path(expanded)
    return p if p.is_absolute() else root / p


def resolve_with_override(root: Path, raw: str, env_override: Optional[str]) -> Path:
    if env_override:
        override = os.environ.get(env_override, "").strip()
        if override:
            return resolve_path(root, override)
    return resolve_path(root, raw)


def validate_registry(reg: dict, root: Path) -> list[str]:
    failures: list[str] = []

    src_rows = reg.get("source", [])
    ide_rows = reg.get("ide", [])
    skill_rows = reg.get("skill", [])

    if not src_rows:
        failures.append("registry missing [[source]] entries")
    if not ide_rows:
        failures.append("registry missing [[ide]] entries")
    if not skill_rows:
        failures.append("registry missing [[skill]] entries")

    source_by_id = {str(s.get("id", "")).strip(): s for s in src_rows if str(s.get("id", "")).strip()}
    ide_by_id = {str(i.get("id", "")).strip(): i for i in ide_rows if str(i.get("id", "")).strip()}

    for sid, src in source_by_id.items():
        base = str(src.get("base_path", "")).strip()
        env_override = str(src.get("env_override", "")).strip() or None
        if not base:
            failures.append(f"source {sid} missing base_path")
            continue
        bp = resolve_with_override(root, base, env_override)
        if not bp.exists():
            failures.append(f"source {sid} base_path does not exist: {bp}")

    for ide_id, ide in ide_by_id.items():
        inst = str(ide.get("install_root", "")).strip()
        if not inst:
            failures.append(f"ide {ide_id} missing install_root")

    for sk in skill_rows:
        skill_id = str(sk.get("id", "")).strip()
        source_id = str(sk.get("source_id", "")).strip()
        source_rel = str(sk.get("source_rel_path", "")).strip()
        targets = sk.get("targets", [])

        if not skill_id:
            failures.append("skill missing id")
            continue
        if source_id not in source_by_id:
            failures.append(f"skill {skill_id} references unknown source_id: {source_id}")
            continue
        if not source_rel:
            failures.append(f"skill {skill_id} missing source_rel_path")
            continue

        src_env_override = str(source_by_id[source_id].get("env_override", "")).strip() or None
        base = resolve_with_override(root, str(source_by_id[source_id]["base_path"]), src_env_override)
        src_dir = base / source_rel
        if not src_dir.exists() or not src_dir.is_dir():
            failures.append(f"skill {skill_id} missing source dir: {src_dir}")
            continue
        if not (src_dir / "SKILL.md").exists():
            failures.append(f"skill {skill_id} missing SKILL.md at source: {src_dir / 'SKILL.md'}")

        if not isinstance(targets, list) or not targets:
            failures.append(f"skill {skill_id} must declare non-empty targets[]")
            continue
        for t in targets:
            if str(t) not in ide_by_id:
                failures.append(f"skill {skill_id} references unknown ide target: {t}")

    return failures


def build_rows(reg: dict, root: Path, ide_filter: Optional[Set[str]]) -> list[SyncRow]:
    source_by_id = {str(s["id"]): s for s in reg.get("source", [])}
    ide_by_id = {str(i["id"]): i for i in reg.get("ide", [])}

    rows: list[SyncRow] = []

    for sk in reg.get("skill", []):
        skill_id = str(sk["id"])
        source = source_by_id[str(sk["source_id"])]
        src_env_override = str(source.get("env_override", "")).strip() or None
        source_dir = resolve_with_override(root, str(source["base_path"]), src_env_override) / str(sk["source_rel_path"])
        source_digest = sha256_dir(source_dir)

        for target_ide in sk.get("targets", []):
            target_ide = str(target_ide)
            if ide_filter and target_ide not in ide_filter:
                continue
            ide = ide_by_id[target_ide]
            ide_env_override = str(ide.get("env_override", "")).strip() or None
            target_root = resolve_with_override(root, str(ide["install_root"]), ide_env_override)
            target_dir = target_root / skill_id

            if target_dir.resolve() == source_dir.resolve():
                action = "skip_same_path"
                target_digest = source_digest
            elif not target_dir.exists():
                action = "install"
                target_digest = None
            else:
                target_digest = sha256_dir(target_dir)
                action = "noop" if target_digest == source_digest else "update"

            rows.append(
                SyncRow(
                    skill_id=skill_id,
                    source=source_dir,
                    target_ide=target_ide,
                    target=target_dir,
                    action=action,
                    source_digest=source_digest,
                    target_digest=target_digest,
                )
            )

    return rows


def copy_skill(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def run_quick_validate(root: Path, skill_dir: Path) -> tuple[bool, str]:
    override = os.environ.get("NOSTRA_SKILL_VALIDATOR", "").strip()
    if override:
        validator = resolve_path(root, override)
    else:
        validator = root / "scripts/quick_validate_registry_asset.py"
    if not validator.exists():
        return True, "validator not found; skipped"
    try:
        out = subprocess.check_output(
            [
                str(root / "scripts" / "run_repo_python.sh"),
                str(validator),
                "--kind",
                "skill",
                str(skill_dir),
            ],
            text=True,
            stderr=subprocess.STDOUT,
        )
        return True, out.strip()
    except subprocess.CalledProcessError as exc:
        return False, (exc.output or "validation failed").strip()


def print_rows(rows: Iterable[SyncRow]) -> None:
    for r in rows:
        print(f"{r.skill_id} [{r.target_ide}] {r.action} -> {r.target}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Sync skills to IDE install roots from canonical registry")
    parser.add_argument("--mode", choices=["validate", "plan", "check", "install"], default="plan")
    parser.add_argument("--ide", action="append", help="target IDE id (repeatable), default: all")
    parser.add_argument("--registry", default="nostra/commons/skills/registry.toml")
    parser.add_argument("--json", action="store_true", help="print summary JSON")
    args = parser.parse_args()

    root = repo_root()
    registry_path = resolve_path(root, args.registry)
    reg = load_registry(registry_path)

    failures = validate_registry(reg, root)
    if failures:
        print("FAIL: skills registry validation failed")
        for f in failures:
            print(f" - {f}")
        return 1

    if args.mode == "validate":
        print("PASS: skills registry validation")
        return 0

    ide_filter = set(args.ide) if args.ide else None
    rows = build_rows(reg, root, ide_filter)

    if args.mode in {"plan", "check"}:
        print_rows(rows)
        if args.mode == "check":
            drift = [r for r in rows if r.action in {"install", "update"}]
            if drift:
                print(f"FAIL: skill drift detected ({len(drift)} targets)")
                return 1
            print("PASS: all targeted skills are synchronized")
            return 0
        return 0

    # install mode
    validation_failures: list[str] = []
    for r in rows:
        if r.action in {"install", "update"}:
            copy_skill(r.source, r.target)
            ok, msg = run_quick_validate(root, r.target)
            if not ok:
                validation_failures.append(f"{r.skill_id}[{r.target_ide}] {msg}")

    print_rows(rows)
    if validation_failures:
        print("FAIL: post-install validation failures")
        for f in validation_failures:
            print(f" - {f}")
        return 1

    print("PASS: skill sync install complete")
    if args.json:
        payload = {
            "registry": str(registry_path),
            "rows": [r.__dict__ for r in rows],
        }
        print(json.dumps(payload, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
