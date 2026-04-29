#!/usr/bin/env python3
"""Validate the Cortex Web Heap / block capability observability fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/heapBlockCapabilityInventory.fixture.json"

ALLOWED_CLASSES = {
    "read_projection",
    "local_ui_state",
    "runtime_read",
    "runtime_write",
    "steward_gated_write",
    "destructive_write",
    "download",
    "placeholder_or_disabled",
    "overlay_interaction",
}
REQUIRED_ACTIONS = {
    "create",
    "regenerate",
    "refine_selection",
    "export",
    "history",
    "publish",
    "synthesize",
    "pin",
    "delete",
    "discussion",
    "relation_edit",
}
REQUIRED_ZONES = {
    "heap_page_bar",
    "heap_selection_bar",
    "heap_card_menu",
    "heap_detail_header",
    "heap_detail_footer",
}
REQUIRED_CREATE_MODES = {"create", "generate", "upload", "chat", "plan"}
REQUIRED_DETAIL_TABS = {"preview", "relations", "attributes", "code"}
REQUIRED_GAPS = {
    "heap.block.comments.persistence",
    "heap.block.overlay.layering",
    "heap.block.regenerate.handler",
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


def require_gap(value: dict[str, Any], context: str) -> None:
    severity = value.get("severity")
    if severity not in {"low", "medium", "high"}:
        fail(f"{context}: severity must be low, medium, or high")
    if not str(value.get("summary", "")).strip():
        fail(f"{context}: summary is required")
    if not str(value.get("recommended_action", "")).strip():
        fail(f"{context}: recommended_action is required")


def require_class(value: dict[str, Any], context: str) -> None:
    item_class = value.get("class")
    if item_class not in ALLOWED_CLASSES:
        fail(f"{context}: unknown class {item_class!r}")


def main() -> None:
    fixture_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FIXTURE
    fixture = load_json(fixture_path)

    if fixture.get("schema_version") != "CortexWebHeapBlockCapabilityInventoryV1":
        fail("schema_version must be CortexWebHeapBlockCapabilityInventoryV1")
    if fixture.get("snapshot_id") != "system:ux:heap-block-capability-inventory":
        fail("snapshot_id must be system:ux:heap-block-capability-inventory")
    if fixture.get("authority_mode") != "recommendation_only":
        fail("authority_mode must remain recommendation_only")

    excludes = set(fixture.get("scope_boundaries", {}).get("excludes", []))
    if "runtime_mutation_authority" not in excludes:
        fail("scope_boundaries.excludes must include runtime_mutation_authority")

    actions = fixture.get("actions")
    if not isinstance(actions, list) or not actions:
        fail("actions must be a non-empty list")
    action_ids = set()
    for action in actions:
        if not isinstance(action, dict):
            fail("each action must be an object")
        action_id = action.get("id")
        if not isinstance(action_id, str) or not action_id:
            fail("action id is required")
        if action_id in action_ids:
            fail(f"duplicate action id: {action_id}")
        action_ids.add(action_id)
        require_class(action, f"action {action_id}")
        zones = action.get("zones")
        if not isinstance(zones, list) or not zones:
            fail(f"action {action_id}: zones must be non-empty")
        unknown_zones = set(zones) - REQUIRED_ZONES
        if unknown_zones:
            fail(f"action {action_id}: unknown zones {sorted(unknown_zones)}")
        if not isinstance(action.get("required_observability"), list) or not action["required_observability"]:
            fail(f"action {action_id}: required_observability must be non-empty")
        if action.get("class") == "destructive_write":
            confirmation = action.get("confirmation_contract")
            if not isinstance(confirmation, dict):
                fail(f"action {action_id}: destructive actions require confirmation_contract")
            if confirmation.get("required") is not True:
                fail(f"action {action_id}: confirmation_contract.required must be true")
            if confirmation.get("style") != "danger":
                fail(f"action {action_id}: confirmation_contract.style must be danger")
            if confirmation.get("fallback_enforced") is not True:
                fail(f"action {action_id}: confirmation_contract.fallback_enforced must be true")
        if action.get("class") == "placeholder_or_disabled":
            gap = action.get("known_gap")
            if isinstance(gap, dict):
                require_gap(gap, f"action {action_id}.known_gap")

    missing_actions = REQUIRED_ACTIONS - action_ids
    if missing_actions:
        fail(f"missing required actions {sorted(missing_actions)}")

    zones = fixture.get("action_zones")
    if not isinstance(zones, list) or not zones:
        fail("action_zones must be a non-empty list")
    zone_ids = {zone.get("zone") for zone in zones if isinstance(zone, dict)}
    missing_zones = REQUIRED_ZONES - zone_ids
    if missing_zones:
        fail(f"missing action zones {sorted(missing_zones)}")

    create_modes = {
        mode.get("mode")
        for mode in fixture.get("create_modes", [])
        if isinstance(mode, dict)
    }
    missing_modes = REQUIRED_CREATE_MODES - create_modes
    if missing_modes:
        fail(f"missing create modes {sorted(missing_modes)}")
    for mode in fixture.get("create_modes", []):
        if isinstance(mode, dict):
            require_class(mode, f"create mode {mode.get('mode')}")

    detail_tabs = {
        tab.get("id")
        for tab in fixture.get("detail_tabs", [])
        if isinstance(tab, dict)
    }
    missing_tabs = REQUIRED_DETAIL_TABS - detail_tabs
    if missing_tabs:
        fail(f"missing detail tabs {sorted(missing_tabs)}")
    for tab in fixture.get("detail_tabs", []):
        if isinstance(tab, dict):
            require_class(tab, f"detail tab {tab.get('id')}")

    gap_ids = set()
    for gap in fixture.get("known_gaps", []):
        if not isinstance(gap, dict):
            fail("each known gap must be an object")
        gap_id = gap.get("id")
        if not isinstance(gap_id, str) or not gap_id:
            fail("known gap id is required")
        gap_ids.add(gap_id)
        require_gap(gap, f"known gap {gap_id}")
    missing_gaps = REQUIRED_GAPS - gap_ids
    if missing_gaps:
        fail(f"missing required known gaps {sorted(missing_gaps)}")

    print(
        "PASS: Cortex Web Heap block capability inventory validates "
        f"({len(actions)} actions, {len(zones)} zones, {len(create_modes)} create modes)"
    )


if __name__ == "__main__":
    main()
