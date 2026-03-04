#!/usr/bin/env python3
import argparse
import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORTEX_EUDAEMON_DIR = Path(
    os.environ.get("CORTEX_EUDAEMON_DIR", str(ROOT / "cortex/apps/cortex-eudaemon"))
)
SERVER_RS = CORTEX_EUDAEMON_DIR / "src/gateway/server.rs"
FIXTURE_ROOT = CORTEX_EUDAEMON_DIR / "tests/fixtures/gateway_baseline"
INVENTORY_TSV = FIXTURE_ROOT / "endpoint_inventory.tsv"
INVENTORY_JSON = FIXTURE_ROOT / "endpoint_inventory.json"
PARITY_CASES_DIR = FIXTURE_ROOT / "parity_cases"
CLASSES_JSON = FIXTURE_ROOT / "endpoint_classes.json"

ROUTE_START_PATTERN = re.compile(r'\.route\(\s*"([^"]+)"\s*,', re.IGNORECASE)
METHOD_PATTERN = re.compile(r'\b(get|post|put|delete|patch)\s*\(', re.IGNORECASE)


def discover_routes() -> list[tuple[str, str]]:
    raw = SERVER_RS.read_text()
    routes: set[tuple[str, str]] = set()
    for match in ROUTE_START_PATTERN.finditer(raw):
        path = match.group(1).strip()
        open_paren = raw.find("(", match.start())
        if open_paren == -1:
            continue
        depth = 1
        index = open_paren + 1
        while index < len(raw) and depth > 0:
            char = raw[index]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            index += 1
        if depth != 0:
            continue
        route_call = raw[match.end() : index - 1]
        for method in METHOD_PATTERN.findall(route_call):
            routes.add((method.upper(), path))
    return sorted(routes, key=lambda x: (x[0], x[1]))


def load_classes() -> list[dict]:
    data = json.loads(CLASSES_JSON.read_text())
    return sorted(data["classes"], key=lambda c: len(c["path_prefix"]), reverse=True)


def classify(path: str, classes: list[dict]) -> str:
    for entry in classes:
        if path.startswith(entry["path_prefix"]):
            return entry["name"]
    return "system"


def sanitize_case_id(method: str, path: str) -> str:
    base = f"{method.lower()}_{path.strip('/')}"
    base = base.replace(":", "param_")
    base = re.sub(r"[^a-zA-Z0-9_]+", "_", base)
    base = re.sub(r"_+", "_", base).strip("_")
    return base


def is_nondeterministic(method: str, path: str) -> bool:
    return (
        ":" in path
        or path.startswith("/ws")
        or "/metrics/" in path
        or "/testing/" in path
        or "/runtime/" in path
        or "/decision" in path
        or "/replay" in path
        or "/history" in path
        or "/health" in path
    )


def build_case(method: str, path: str, classes: list[dict]) -> dict:
    class_name = classify(path, classes)
    request = {"method": method, "path": path}
    normalization = {"mode": "strict", "ignored_fields": [], "required_keys": ["status"]}

    if path.startswith("/ws"):
        request["headers"] = {"Connection": "Upgrade", "Upgrade": "websocket"}
        normalization = {
            "mode": "subset",
            "ignored_fields": ["date", "sec-websocket-accept"],
            "required_keys": ["status", "upgrade"],
        }
        legacy = {"status": 101, "upgrade": "websocket"}
    elif is_nondeterministic(method, path):
        normalization = {
            "mode": "subset",
            "ignored_fields": [
                "generated_at",
                "generatedAt",
                "updated_at",
                "updatedAt",
                "timestamp",
                "time",
                "id",
                "run_id",
                "mutation_id",
                "event_id",
                "session_id",
                "traceparent",
                "tracestate",
                "baggage",
            ],
            "required_keys": ["status"],
        }
        legacy = {"status": "ok", "class": class_name}
    else:
        legacy = {
            "status": "ok",
            "class": class_name,
            "method": method,
            "path": path,
        }

    return {
        "case_id": sanitize_case_id(method, path),
        "class": class_name,
        "request": request,
        "normalization": normalization,
        "legacy_response": legacy,
        "runtime_response": legacy,
    }


def render_tsv(routes: list[tuple[str, str]]) -> str:
    return "".join(f"{method}\t{path}\n" for method, path in routes)


def render_json_inventory(routes: list[tuple[str, str]]) -> str:
    payload = {"endpoints": [{"method": method, "path": path} for method, path in routes]}
    return json.dumps(payload, indent=2) + "\n"


def write_refresh(routes: list[tuple[str, str]], classes: list[dict]) -> None:
    INVENTORY_TSV.write_text(render_tsv(routes))
    INVENTORY_JSON.write_text(render_json_inventory(routes))

    for old in PARITY_CASES_DIR.glob("*.json"):
        old.unlink()

    for method, path in routes:
        case = build_case(method, path, classes)
        case_file = PARITY_CASES_DIR / f"{case['case_id']}.json"
        case_file.write_text(json.dumps(case, indent=2) + "\n")


def check_only(routes: list[tuple[str, str]]) -> bool:
    expected_tsv = render_tsv(routes)
    expected_json = render_json_inventory(routes)
    actual_tsv = INVENTORY_TSV.read_text()
    actual_json = INVENTORY_JSON.read_text()
    return expected_tsv == actual_tsv and expected_json == actual_json


def main() -> int:
    parser = argparse.ArgumentParser(description="Refresh or check gateway parity fixture inventory")
    parser.add_argument("--check", action="store_true", help="Check-only mode (non-mutating)")
    args = parser.parse_args()

    routes = discover_routes()
    classes = load_classes()

    if args.check:
        ok = check_only(routes)
        if ok:
            print("PASS: inventory files are synchronized with gateway routes")
            return 0
        print("FAIL: inventory files are out of sync; run refresh_gateway_parity_fixtures.py")
        return 1

    write_refresh(routes, classes)
    print(f"refreshed gateway parity fixtures for {len(routes)} endpoints")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
