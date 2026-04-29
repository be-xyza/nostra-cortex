#!/usr/bin/env python3
"""Validate the Cortex Web shell route/surface observability fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/shellSurfaceInventory.fixture.json"

VISIBLE_NAV_BLOCKED_STATUSES = {"under_construction", "failing_500", "missing", "inconsistent"}
REQUIRED_SCOPE_EXCLUDES = {
    "heap_block_capability_deep_validation",
    "block_action_mutation_contracts",
}
REQUIRED_SETTINGS_CATEGORIES = {
    "personal_preferences",
    "space_settings",
    "workbench_settings",
    "operator_settings",
    "design_theme_governance",
}
ALLOWED_CLASSES = {
    "governed_a2ui_surface",
    "native_cortex_host_surface",
    "execution_surface",
    "labs_experiment",
    "placeholder",
    "redirect_alias",
    "operator_only_admin",
    "settings_gap",
}


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text())
    except FileNotFoundError:
        fail(f"fixture not found: {path}")
    except json.JSONDecodeError as exc:
        fail(f"{path}: invalid JSON: {exc}")
    if not isinstance(value, dict):
        fail(f"{path}: expected JSON object")
    return value


def require_known_gap(route: dict[str, Any], context: str) -> None:
    gap = route.get("known_gap")
    if not isinstance(gap, dict):
        fail(f"{context}: requires known_gap metadata")
    severity = gap.get("severity")
    if severity not in {"low", "medium", "high"}:
        fail(f"{context}: known_gap.severity must be low, medium, or high")
    if not str(gap.get("summary", "")).strip():
        fail(f"{context}: known_gap.summary is required")
    if not str(gap.get("recommended_action", "")).strip():
        fail(f"{context}: known_gap.recommended_action is required")


def main() -> None:
    fixture_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FIXTURE
    fixture = load_json(fixture_path)

    if fixture.get("schema_version") != "CortexWebShellSurfaceInventoryV1":
        fail("schema_version must be CortexWebShellSurfaceInventoryV1")
    if fixture.get("authority_mode") != "recommendation_only":
        fail("authority_mode must remain recommendation_only")

    excludes = set(fixture.get("scope_boundaries", {}).get("excludes", []))
    missing_excludes = REQUIRED_SCOPE_EXCLUDES - excludes
    if missing_excludes:
        fail(f"scope_boundaries.excludes missing {sorted(missing_excludes)}")

    declared_classes = set(fixture.get("route_classes", []))
    unknown_declared = declared_classes - ALLOWED_CLASSES
    if unknown_declared:
        fail(f"route_classes contains unknown classes {sorted(unknown_declared)}")

    routes = fixture.get("routes")
    if not isinstance(routes, list) or not routes:
        fail("routes must be a non-empty list")

    by_route: dict[str, dict[str, Any]] = {}
    for route in routes:
        if not isinstance(route, dict):
            fail("each route entry must be an object")
        route_id = route.get("route")
        if not isinstance(route_id, str) or not route_id.startswith("/"):
            fail(f"invalid route id: {route_id!r}")
        if route_id in by_route:
            fail(f"duplicate route: {route_id}")
        by_route[route_id] = route
        route_class = route.get("class")
        if route_class not in ALLOWED_CLASSES:
            fail(f"{route_id}: unknown class {route_class!r}")
        if not str(route.get("status", "")).strip():
            fail(f"{route_id}: status is required")
        if route_class == "redirect_alias" and not str(route.get("target", "")).startswith("/"):
            fail(f"{route_id}: redirect_alias requires target")
        if route_class in {"native_cortex_host_surface", "governed_a2ui_surface", "execution_surface"}:
            if not str(route.get("contract_status", "")).strip():
                fail(f"{route_id}: contract_status is required for surface routes")
        if route.get("status") in VISIBLE_NAV_BLOCKED_STATUSES:
            require_known_gap(route, route_id)

    visible_nav = fixture.get("visible_navigation")
    if not isinstance(visible_nav, list) or not visible_nav:
        fail("visible_navigation must be a non-empty list")

    for item in visible_nav:
        if not isinstance(item, dict):
            fail("each visible_navigation item must be an object")
        label = str(item.get("label", "")).strip()
        route_id = item.get("route")
        if not label:
            fail("visible_navigation item missing label")
        if route_id not in by_route:
            fail(f"visible nav {label}: route {route_id!r} missing from routes")
        route = by_route[route_id]
        if route.get("status") in VISIBLE_NAV_BLOCKED_STATUSES:
            require_known_gap(route, f"visible nav {label} -> {route_id}")
        if not str(item.get("source", "")).strip():
            fail(f"visible nav {label}: source is required")
        if not str(item.get("slot", "")).strip():
            fail(f"visible nav {label}: slot is required")

    categories = {
        item.get("category")
        for item in fixture.get("settings_requirements", [])
        if isinstance(item, dict)
    }
    missing_categories = REQUIRED_SETTINGS_CATEGORIES - categories
    if missing_categories:
        fail(f"settings_requirements missing {sorted(missing_categories)}")

    for category in fixture.get("settings_requirements", []):
        items = category.get("items") if isinstance(category, dict) else None
        if not isinstance(items, list) or len(items) < 3:
            fail(f"settings category {category.get('category')!r} must include at least three items")

    print(
        "PASS: Cortex Web shell surface inventory validates "
        f"({len(routes)} routes, {len(visible_nav)} visible nav entries)"
    )


if __name__ == "__main__":
    main()
