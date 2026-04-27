#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts/check_ndl_design_profiles.py"
PROFILE = ROOT / "research/120-nostra-design-language/prototypes/space-design/SPACE_DESIGN.space-profile.v1.json"
A2UI_THEMES = ROOT / "shared/a2ui/themes"
A2UI_FIXTURES = ROOT / "shared/a2ui/fixtures"
WEB_SPACE_DESIGN_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/spaceDesignProfilePreview.fixture.json"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n")


def run_checker(
    profile_path: Path,
    fixture_dir: Path | None = None,
    theme_dir: Path | None = None,
    web_fixture_path: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    command = [
        sys.executable,
        str(CHECKER),
        str(profile_path),
        "--imports",
        "--templates",
    ]
    if theme_dir is not None:
        command.extend(["--a2ui-theme-dir", str(theme_dir)])
    if fixture_dir is not None:
        command.extend(["--a2ui-fixture-dir", str(fixture_dir)])
    if web_fixture_path is not None:
        command.extend(["--cortex-web-space-design-fixture", str(web_fixture_path)])
    return subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)


def copy_theme_dir(tmpdir: Path) -> Path:
    theme_dir = tmpdir / "themes"
    theme_dir.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(A2UI_THEMES, theme_dir)
    return theme_dir


def copy_fixture_dir(tmpdir: Path) -> Path:
    fixture_dir = tmpdir / "fixtures"
    fixture_dir.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(A2UI_FIXTURES, fixture_dir)
    return fixture_dir


def expect_pass(
    name: str,
    profile_path: Path,
    fixture_dir: Path | None = None,
    theme_dir: Path | None = None,
    web_fixture_path: Path | None = None,
) -> None:
    result = run_checker(profile_path, fixture_dir, theme_dir, web_fixture_path)
    if result.returncode != 0:
        print(f"FAIL: {name} unexpectedly failed", file=sys.stderr)
        print(result.stdout, file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        raise SystemExit(1)


def expect_failure(
    name: str,
    profile_path: Path,
    expected: str,
    fixture_dir: Path | None = None,
    theme_dir: Path | None = None,
    web_fixture_path: Path | None = None,
) -> None:
    result = run_checker(profile_path, fixture_dir, theme_dir, web_fixture_path)
    output = result.stdout + result.stderr
    if result.returncode == 0:
        print(f"FAIL: {name} unexpectedly passed", file=sys.stderr)
        print(output, file=sys.stderr)
        raise SystemExit(1)
    if expected not in output:
        print(f"FAIL: {name} did not report expected text: {expected}", file=sys.stderr)
        print(output, file=sys.stderr)
        raise SystemExit(1)


def profile_case(tmpdir: Path, name: str, mutate: Callable[[dict[str, Any]], None]) -> Path:
    profile = load_json(PROFILE)
    mutate(profile)
    path = tmpdir / f"{name}.space-profile.v1.json"
    write_json(path, profile)
    return path


def fixture_case(tmpdir: Path, name: str, mutate: Callable[[dict[str, Any]], None]) -> Path:
    fixture_dir = copy_fixture_dir(tmpdir / name)
    fixture_path = fixture_dir / "render_surface_golden.json"
    fixture = load_json(fixture_path)
    mutate(fixture)
    write_json(fixture_path, fixture)
    return fixture_dir


def theme_case(tmpdir: Path, name: str, mutate: Callable[[dict[str, Any]], None]) -> Path:
    theme_dir = copy_theme_dir(tmpdir / name)
    theme_path = theme_dir / "cortex.json"
    theme = load_json(theme_path)
    mutate(theme)
    write_json(theme_path, theme)
    return theme_dir


def web_fixture_case(tmpdir: Path, name: str, mutate: Callable[[dict[str, Any]], None]) -> Path:
    fixture_path = tmpdir / f"{name}.space-design-preview.fixture.json"
    fixture = load_json(WEB_SPACE_DESIGN_FIXTURE)
    mutate(fixture)
    write_json(fixture_path, fixture)
    return fixture_path


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="space-design-a2ui-fixtures-") as raw_tmpdir:
        tmpdir = Path(raw_tmpdir)

        expect_pass("canonical profile and fixtures", PROFILE)

        reused_runtime_allowlist = profile_case(
            tmpdir,
            "reused-runtime-allowlist",
            lambda profile: profile["a2ui_theme_policy"].update({"theme_allowlist_id": "host-default"}),
        )
        expect_failure(
            "profile reuses runtime allowlist",
            reused_runtime_allowlist,
            "reuses a runtime or fixture allowlist id",
        )

        reused_fixture_allowlist = profile_case(
            tmpdir,
            "reused-fixture-allowlist",
            lambda profile: profile["a2ui_theme_policy"].update({"theme_allowlist_id": "trusted-core"}),
        )
        expect_failure(
            "profile reuses fixture allowlist",
            reused_fixture_allowlist,
            "reuses a runtime or fixture allowlist id",
        )

        runtime_token_theme_dir = theme_case(
            tmpdir,
            "runtime-token-version",
            lambda theme: theme["policy"].update({"supported_token_version": "ndl-token-v1"}),
        )
        expect_failure(
            "profile claims runtime token version",
            PROFILE,
            "not a runtime A2UI token version",
            theme_dir=runtime_token_theme_dir,
        )

        tier1_token_claim = profile_case(
            tmpdir,
            "tier1-token-claim",
            lambda profile: profile["design_tokens"]["components"]["artifact-card"].update({"textColor": "tier1-authority"}),
        )
        expect_failure(
            "profile token claims Tier 1 authority",
            tier1_token_claim,
            "design_tokens must not claim theme allowlist, runtime, or Tier 1 governance authority",
        )

        unknown_theme_dir = fixture_case(
            tmpdir,
            "unknown-theme",
            lambda fixture: fixture["meta"].update({"theme": "unknown-space-theme"}),
        )
        expect_failure(
            "fixture references unknown theme",
            PROFILE,
            "is not present in shared/a2ui/themes",
            unknown_theme_dir,
        )

        unsafe_mode_dir = fixture_case(
            tmpdir,
            "unsafe-mode",
            lambda fixture: fixture["meta"].update({"safe_mode": False}),
        )
        expect_failure(
            "fixture disables safe mode",
            PROFILE,
            "themed A2UI fixtures must keep safe_mode true",
            unsafe_mode_dir,
        )

        runtime_bound_web_fixture = web_fixture_case(
            tmpdir,
            "web-runtime-binding",
            lambda fixture: fixture.update({"runtime_binding": "theme_selection"}),
        )
        expect_failure(
            "Cortex Web preview fixture claims runtime binding",
            PROFILE,
            "runtime_binding must remain none",
            web_fixture_path=runtime_bound_web_fixture,
        )

        token_carrying_web_fixture = web_fixture_case(
            tmpdir,
            "web-design-tokens",
            lambda fixture: fixture["profiles"][0].update({"design_tokens": {"colors": {"primary": "#000000"}}}),
        )
        expect_failure(
            "Cortex Web preview fixture carries profile tokens",
            PROFILE,
            "must not carry design_tokens",
            web_fixture_path=token_carrying_web_fixture,
        )

    print("PASS: Space design A2UI fixture validation regression coverage")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
