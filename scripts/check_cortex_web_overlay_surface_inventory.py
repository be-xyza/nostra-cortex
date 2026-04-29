#!/usr/bin/env python3
"""Validate the Cortex Web overlay/modal observability fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/overlaySurfaceInventory.fixture.json"

ALLOWED_AUTHORITY_CLASSES = {
    "local_ui_state",
    "runtime_read",
    "runtime_write",
    "steward_gated_write",
    "operator_only",
    "destructive_confirmation",
}
ALLOWED_SURFACE_KINDS = {"modal", "sheet", "drawer", "sidebar", "panel", "popover", "confirmation"}
REQUIRED_SCOPE_EXCLUDES = {"runtime_mutation_authority", "product_behavior_changes"}
REQUIRED_SURFACES = {
    "overlay.heap.detail_modal",
    "overlay.heap.chat_panel",
    "overlay.heap.comment_sidebar",
    "overlay.heap.steward_gate",
    "overlay.heap.aggregation_detail",
    "overlay.heap.saved_view_modal",
    "overlay.heap.history_modal",
    "overlay.heap.create_panel",
    "overlay.shell.space_selector",
    "overlay.shell.role_selector",
    "overlay.shell.workbench_naming",
    "overlay.shell.saved_view_confirmations",
    "overlay.system.provider_create_sheet",
    "overlay.system.provider_detail_sheet",
    "overlay.system.provider_discovery_sheet",
    "overlay.artifacts.workflow_inspector",
    "overlay.artifacts.trace_panel",
    "overlay.artifacts.checkpoint_panel",
    "overlay.artifacts.replay_panel",
    "overlay.shared.confirmation",
}
REQUIRED_GAPS = {
    "overlay.heap.chat_create_collision",
    "overlay.lifecycle.metadata_thin",
    "overlay.heap.chat_socket_under_observed",
    "overlay.system.provider_provenance",
    "overlay.artifacts.console_only_action",
}
REQUIRED_FIELDS = {
    "surface_id",
    "owner_route",
    "component",
    "surface_kind",
    "authority_class",
    "open_trigger",
    "close_mechanisms",
    "focus_policy",
    "escape_policy",
    "z_index_band",
    "can_stack_with",
    "known_collision",
    "state_source",
    "persistence",
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

    if fixture.get("schema_version") != "CortexWebOverlaySurfaceInventoryV1":
        fail("schema_version must be CortexWebOverlaySurfaceInventoryV1")
    if fixture.get("snapshot_id") != "system:ux:overlay-surface-inventory":
        fail("snapshot_id must be system:ux:overlay-surface-inventory")
    if fixture.get("authority_mode") != "recommendation_only":
        fail("authority_mode must remain recommendation_only")

    excludes = set(fixture.get("scope_boundaries", {}).get("excludes", []))
    missing_excludes = REQUIRED_SCOPE_EXCLUDES - excludes
    if missing_excludes:
        fail(f"scope_boundaries.excludes missing {sorted(missing_excludes)}")

    surfaces = fixture.get("overlay_surfaces")
    if not isinstance(surfaces, list) or not surfaces:
        fail("overlay_surfaces must be a non-empty list")

    surface_ids: set[str] = set()
    for surface in surfaces:
        if not isinstance(surface, dict):
            fail("each overlay surface must be an object")
        surface_id = surface.get("surface_id")
        require_non_empty_string(surface_id, "surface_id")
        if surface_id in surface_ids:
            fail(f"duplicate surface_id: {surface_id}")
        surface_ids.add(surface_id)

        missing_fields = REQUIRED_FIELDS - set(surface)
        if missing_fields:
            fail(f"{surface_id}: missing fields {sorted(missing_fields)}")
        if surface.get("surface_kind") not in ALLOWED_SURFACE_KINDS:
            fail(f"{surface_id}: unknown surface_kind {surface.get('surface_kind')!r}")
        if surface.get("authority_class") not in ALLOWED_AUTHORITY_CLASSES:
            fail(f"{surface_id}: unknown authority_class {surface.get('authority_class')!r}")
        for field in (
            "owner_route",
            "component",
            "open_trigger",
            "focus_policy",
            "escape_policy",
            "z_index_band",
            "known_collision",
            "state_source",
            "persistence",
            "recommended_action",
        ):
            require_non_empty_string(surface.get(field), f"{surface_id}.{field}")
        close_mechanisms = surface.get("close_mechanisms")
        if not isinstance(close_mechanisms, list) or not close_mechanisms:
            fail(f"{surface_id}.close_mechanisms must be a non-empty list")
        if not all(isinstance(item, str) and item.strip() for item in close_mechanisms):
            fail(f"{surface_id}.close_mechanisms must contain non-empty strings")
        can_stack_with = surface.get("can_stack_with")
        if not isinstance(can_stack_with, list):
            fail(f"{surface_id}.can_stack_with must be a list")
        if "known_gap" in surface:
            gap = surface.get("known_gap")
            if not isinstance(gap, dict):
                fail(f"{surface_id}.known_gap must be an object")
            require_gap(gap, f"{surface_id}.known_gap")

    missing_surfaces = REQUIRED_SURFACES - surface_ids
    if missing_surfaces:
        fail(f"missing required overlay surfaces {sorted(missing_surfaces)}")

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
        "PASS: Cortex Web overlay surface inventory validates "
        f"({len(surfaces)} surfaces, {len(gap_ids)} known gaps)"
    )


if __name__ == "__main__":
    main()
