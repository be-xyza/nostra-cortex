#!/usr/bin/env python3
"""Validate the Cortex Web graph surface observability fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "cortex/apps/cortex-web/src/store/graphSurfaceInventory.fixture.json"

ALLOWED_GRAPH_KINDS = {"contribution", "capability", "workflow", "spatial_plane", "relation"}
ALLOWED_SOURCE_AUTHORITIES = {"nostra", "cortex", "a2ui", "fixture", "local_cache"}
ALLOWED_LAYOUT_ENGINES = {"react_force_graph", "react_flow", "tldraw", "d3", "list", "json", "svg_fallback"}
ALLOWED_COUNT_SENTINELS = {"runtime_dependent", "not_exposed"}
REQUIRED_SCOPE_EXCLUDES = {"runtime_mutation_authority", "graph_data_correctness"}
REQUIRED_SURFACES = {
    "graph.contributions.runs",
    "graph.contributions.full",
    "graph.contributions.edge_quality",
    "graph.contributions.blast_radius",
    "graph.heap.ambient",
    "graph.a2ui.contribution_graph",
    "graph.capability.system",
    "graph.capability.space_overlay",
    "graph.workflow.flow_graph",
    "graph.workflow.normalized_graph",
    "graph.workflow.execution_topology",
    "graph.execution_canvas.spatial_plane",
    "graph.heap.detail_relations",
}
REQUIRED_GAPS = {
    "graph.surface.shared_health_contract",
    "graph.workflow.json_only_projections",
    "graph.a2ui.mock_fallback_visibility",
    "graph.execution_canvas.topology_drift",
    "graph.capability.renderer_visibility",
}
REQUIRED_SURFACE_FIELDS = {
    "surface_id",
    "route_id",
    "graph_kind",
    "source_authority",
    "source_endpoint",
    "schema_version",
    "projection_kind",
    "layout_engine",
    "renderer_mode",
    "render_status",
    "fetch_status",
    "node_count",
    "edge_count",
    "visible_node_count",
    "visible_edge_count",
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


def require_count(value: Any, context: str) -> None:
    if isinstance(value, int) and value >= 0:
        return
    if isinstance(value, str) and value in ALLOWED_COUNT_SENTINELS:
        return
    fail(f"{context} must be a non-negative integer or {sorted(ALLOWED_COUNT_SENTINELS)}")


def require_gap(value: dict[str, Any], context: str) -> None:
    severity = value.get("severity")
    if severity not in {"low", "medium", "high"}:
        fail(f"{context}: severity must be low, medium, or high")
    require_non_empty_string(value.get("summary"), f"{context}.summary")
    require_non_empty_string(value.get("recommended_action"), f"{context}.recommended_action")


def main() -> None:
    fixture_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FIXTURE
    fixture = load_json(fixture_path)

    if fixture.get("schema_version") != "CortexWebGraphSurfaceInventoryV1":
        fail("schema_version must be CortexWebGraphSurfaceInventoryV1")
    if fixture.get("snapshot_id") != "system:ux:graph-surface-inventory":
        fail("snapshot_id must be system:ux:graph-surface-inventory")
    if fixture.get("authority_mode") != "recommendation_only":
        fail("authority_mode must remain recommendation_only")

    excludes = set(fixture.get("scope_boundaries", {}).get("excludes", []))
    missing_excludes = REQUIRED_SCOPE_EXCLUDES - excludes
    if missing_excludes:
        fail(f"scope_boundaries.excludes missing {sorted(missing_excludes)}")

    surfaces = fixture.get("graph_surfaces")
    if not isinstance(surfaces, list) or not surfaces:
        fail("graph_surfaces must be a non-empty list")

    surface_ids: set[str] = set()
    for surface in surfaces:
        if not isinstance(surface, dict):
            fail("each graph surface must be an object")
        surface_id = surface.get("surface_id")
        require_non_empty_string(surface_id, "surface_id")
        if surface_id in surface_ids:
            fail(f"duplicate surface_id: {surface_id}")
        surface_ids.add(surface_id)

        missing_fields = REQUIRED_SURFACE_FIELDS - set(surface)
        if missing_fields:
            fail(f"{surface_id}: missing fields {sorted(missing_fields)}")
        if "graph_hash" not in surface and "graph_root_hash" not in surface:
            fail(f"{surface_id}: requires graph_hash or graph_root_hash")
        if surface.get("graph_kind") not in ALLOWED_GRAPH_KINDS:
            fail(f"{surface_id}: unknown graph_kind {surface.get('graph_kind')!r}")
        if surface.get("source_authority") not in ALLOWED_SOURCE_AUTHORITIES:
            fail(f"{surface_id}: unknown source_authority {surface.get('source_authority')!r}")
        if surface.get("layout_engine") not in ALLOWED_LAYOUT_ENGINES:
            fail(f"{surface_id}: unknown layout_engine {surface.get('layout_engine')!r}")

        for field in ("route_id", "source_endpoint", "schema_version", "projection_kind", "renderer_mode", "render_status", "fetch_status"):
            require_non_empty_string(surface.get(field), f"{surface_id}.{field}")
        for field in ("node_count", "edge_count", "visible_node_count", "visible_edge_count"):
            require_count(surface.get(field), f"{surface_id}.{field}")
        if "known_gap" in surface:
            gap = surface.get("known_gap")
            if not isinstance(gap, dict):
                fail(f"{surface_id}.known_gap must be an object")
            require_gap(gap, f"{surface_id}.known_gap")

    missing_surfaces = REQUIRED_SURFACES - surface_ids
    if missing_surfaces:
        fail(f"missing required graph surfaces {sorted(missing_surfaces)}")

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
        "PASS: Cortex Web graph surface inventory validates "
        f"({len(surfaces)} surfaces, {len(gap_ids)} known gaps)"
    )


if __name__ == "__main__":
    main()
