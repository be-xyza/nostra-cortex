#!/usr/bin/env python3
"""Validate the Cortex Web route/settings IA observability fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/routeIaInventory.fixture.json"

ALLOWED_ROUTE_CLASSES = {
    "native_cortex_host_surface",
    "governed_a2ui_surface",
    "operator_only_admin",
    "labs_experiment",
    "a2ui_candidate",
    "settings_absence_contract",
    "redirect_or_alias",
}
ALLOWED_READINESS = {
    "functional",
    "functional_thin",
    "degraded_empty",
    "inconsistent",
    "under_construction",
    "missing",
    "failing_500",
}
REQUIRED_SCOPE_EXCLUDES = {"runtime_mutation_authority", "new_settings_page"}
REQUIRED_ROUTES = {
    "/spaces",
    "/spaces/:id?tab=overview",
    "/contributions",
    "/artifacts",
    "/workflows",
    "/system",
    "/system/providers",
    "/labs",
    "/labs/space-studio",
    "/labs/execution-canvas",
    "/labs/layout-catalogue",
    "/settings",
    "/studio",
    "/notifications",
    "/metrics",
    "/vfs",
    "/siq",
    "/memory",
    "/simulation",
    "/institutions",
    "/testing",
    "/discovery",
}
REQUIRED_GAPS = {
    "route.settings.absent",
    "route.discovery.visible_placeholder",
    "route.a2ui.seeded_candidates_untyped",
    "route.operator_settings.boundary",
    "route.spaces.readiness_inconsistent",
}
REQUIRED_FIELDS = {
    "route_id",
    "declared_route_owner",
    "nav_source",
    "route_class",
    "typed_route",
    "a2ui_fallback_allowed",
    "visible_in_nav",
    "detail_tabs",
    "settings_affordances",
    "operator_boundary",
    "readiness_status",
    "recommended_action",
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


def require_non_empty_string(value: Any, context: str) -> None:
    if not isinstance(value, str) or not value.strip():
        fail(f"{context} must be a non-empty string")


def require_gap(value: dict[str, Any], context: str) -> None:
    severity = value.get("severity")
    if severity not in {"low", "medium", "high"}:
        fail(f"{context}: severity must be low, medium, or high")
    require_non_empty_string(value.get("summary"), f"{context}.summary")
    require_non_empty_string(value.get("recommended_action"), f"{context}.recommended_action")


def main() -> None:
    fixture_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FIXTURE
    fixture = load_json(fixture_path)

    if fixture.get("schema_version") != "CortexWebRouteIaInventoryV1":
        fail("schema_version must be CortexWebRouteIaInventoryV1")
    if fixture.get("snapshot_id") != "system:ux:route-ia-inventory":
        fail("snapshot_id must be system:ux:route-ia-inventory")
    if fixture.get("authority_mode") != "recommendation_only":
        fail("authority_mode must remain recommendation_only")

    excludes = set(fixture.get("scope_boundaries", {}).get("excludes", []))
    missing_excludes = REQUIRED_SCOPE_EXCLUDES - excludes
    if missing_excludes:
        fail(f"scope_boundaries.excludes missing {sorted(missing_excludes)}")

    absence = fixture.get("settings_absence_contract")
    if not isinstance(absence, dict):
        fail("settings_absence_contract is required")
    if absence.get("route_id") != "/settings":
        fail("settings_absence_contract.route_id must be /settings")
    if absence.get("readiness_status") != "missing":
        fail("settings_absence_contract.readiness_status must be missing")
    if absence.get("global_settings_page_allowed_this_stage") is not False:
        fail("global settings page must remain disallowed in this stage")
    require_non_empty_string(absence.get("current_owner_rule"), "settings_absence_contract.current_owner_rule")

    routes = fixture.get("routes")
    if not isinstance(routes, list) or not routes:
        fail("routes must be a non-empty list")

    route_ids: set[str] = set()
    by_route: dict[str, dict[str, Any]] = {}
    for route in routes:
        if not isinstance(route, dict):
            fail("each route entry must be an object")
        route_id = route.get("route_id")
        require_non_empty_string(route_id, "route_id")
        if route_id in route_ids:
            fail(f"duplicate route_id: {route_id}")
        route_ids.add(route_id)
        by_route[route_id] = route

        missing_fields = REQUIRED_FIELDS - set(route)
        if missing_fields:
            fail(f"{route_id}: missing fields {sorted(missing_fields)}")
        if route.get("route_class") not in ALLOWED_ROUTE_CLASSES:
            fail(f"{route_id}: unknown route_class {route.get('route_class')!r}")
        if route.get("readiness_status") not in ALLOWED_READINESS:
            fail(f"{route_id}: unknown readiness_status {route.get('readiness_status')!r}")
        for field in ("declared_route_owner", "nav_source", "operator_boundary", "recommended_action"):
            require_non_empty_string(route.get(field), f"{route_id}.{field}")
        for field in ("typed_route", "a2ui_fallback_allowed", "visible_in_nav"):
            if not isinstance(route.get(field), bool):
                fail(f"{route_id}.{field} must be a boolean")
        for field in ("detail_tabs", "settings_affordances"):
            if not isinstance(route.get(field), list):
                fail(f"{route_id}.{field} must be a list")
        if "known_gap" in route:
            gap = route.get("known_gap")
            if not isinstance(gap, dict):
                fail(f"{route_id}.known_gap must be an object")
            require_gap(gap, f"{route_id}.known_gap")

    missing_routes = REQUIRED_ROUTES - route_ids
    if missing_routes:
        fail(f"missing required routes {sorted(missing_routes)}")

    settings = by_route["/settings"]
    if settings.get("readiness_status") != "missing":
        fail("/settings must remain missing")
    if settings.get("typed_route") is not False or settings.get("a2ui_fallback_allowed") is not False:
        fail("/settings must not be typed or A2UI fallback-enabled in this stage")

    providers = by_route["/system/providers"]
    if providers.get("operator_boundary") != "operator_only":
        fail("/system/providers must remain operator_only")

    discovery = by_route["/discovery"]
    if discovery.get("visible_in_nav") is not True or discovery.get("readiness_status") != "under_construction":
        fail("/discovery must remain classified as visible under-construction nav gap")

    a2ui_candidates = [
        route for route in routes
        if route.get("route_class") == "a2ui_candidate"
    ]
    if len(a2ui_candidates) < 9:
        fail("expected A2UI-only candidate exemptions for seeded hidden routes")
    for route in a2ui_candidates:
        if route.get("a2ui_fallback_allowed") is not True:
            fail(f"{route.get('route_id')}: A2UI candidates must allow fallback")

    gap_ids = set()
    for gap in fixture.get("known_gaps", []):
        if not isinstance(gap, dict):
            fail("each known gap must be an object")
        gap_id = gap.get("id")
        require_non_empty_string(gap_id, "known_gap.id")
        gap_ids.add(gap_id)
        require_gap(gap, f"known gap {gap_id}")
    missing_gaps = REQUIRED_GAPS - gap_ids
    if missing_gaps:
        fail(f"missing required known gaps {sorted(missing_gaps)}")

    print(
        "PASS: Cortex Web route IA inventory validates "
        f"({len(routes)} routes, {len(a2ui_candidates)} A2UI candidates)"
    )


if __name__ == "__main__":
    main()
