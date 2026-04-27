#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SCHEMA = ROOT / "research/120-nostra-design-language/schemas/SpaceDesignProfileV1.schema.json"
DEFAULT_IMPORT_SCHEMA = ROOT / "research/120-nostra-design-language/schemas/DesignElementImportV1.schema.json"
DEFAULT_TEMPLATE_SCHEMA = ROOT / "research/120-nostra-design-language/schemas/SpaceTemplatePackV1.schema.json"
DEFAULT_PROFILE_GLOB = "research/120-nostra-design-language/prototypes/**/*.space-profile.v1.json"
DEFAULT_IMPORT_GLOB = "research/120-nostra-design-language/prototypes/**/*.design-import.v1.json"
DEFAULT_TEMPLATE_GLOB = "research/120-nostra-design-language/prototypes/**/*.template-pack.v1.json"

EXPECTED_SECTIONS = [
    "Overview",
    "Colors",
    "Typography",
    "Layout",
    "Elevation & Depth",
    "Shapes",
    "Components",
    "Do's and Don'ts",
]
REQUIRED_PROHIBITED_CLAIMS = {
    "ratified",
    "approved",
    "constitutional",
    "verified_identity",
    "steward_authorized",
}
PROHIBITED_TOKEN_TERMS = {
    "ratified",
    "approved",
    "constitutional",
    "verified",
    "steward-authorized",
    "steward_authorized",
}
REQUIRED_THEME_FIELDS = {
    "token_version",
    "safe_mode",
    "theme_allowlist_id",
    "motion_policy",
    "contrast_preference",
}
VALID_COMPONENT_PROPS = {
    "backgroundColor",
    "textColor",
    "typography",
    "rounded",
    "padding",
    "size",
    "height",
    "width",
}
TOKEN_REF_RE = re.compile(r"^\{([A-Za-z0-9._-]+)\}$")
HEX_RE = re.compile(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$")


class CheckFailure(Exception):
    pass


def fail(message: str) -> None:
    raise CheckFailure(message)


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"{path}: invalid JSON: {exc}")


def validate_with_schema(schema_path: Path, profile_path: Path, profile: dict[str, Any]) -> None:
    schema = load_json(schema_path)
    try:
        from jsonschema import Draft202012Validator
    except ImportError:
        required = schema.get("required", [])
        missing = [field for field in required if field not in profile]
        if missing:
            fail(f"{profile_path}: missing required schema fields without jsonschema installed: {missing}")
        return

    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(profile), key=lambda item: list(item.path))
    if errors:
        details = []
        for error in errors[:8]:
            location = "/".join(str(part) for part in error.path) or "<root>"
            details.append(f"{location}: {error.message}")
        fail(f"{profile_path}: schema validation failed: {'; '.join(details)}")


def read_front_matter(path: Path) -> dict[str, Any]:
    text = path.read_text()
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        fail(f"{path}: missing YAML front matter")
    end = None
    for idx, line in enumerate(lines[1:], start=1):
        if line.strip() == "---":
            end = idx
            break
    if end is None:
        fail(f"{path}: unterminated YAML front matter")
    return parse_simple_yaml(lines[1:end], path)


def parse_simple_yaml(lines: list[str], path: Path) -> dict[str, Any]:
    root: dict[str, Any] = {}
    stack: list[tuple[int, dict[str, Any]]] = [(-1, root)]

    for raw in lines:
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        if indent % 2 != 0:
            fail(f"{path}: unsupported YAML indentation: {raw}")
        line = raw.strip()
        if ":" not in line:
            fail(f"{path}: unsupported YAML line: {raw}")
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()

        while stack and indent <= stack[-1][0]:
            stack.pop()
        if not stack:
            fail(f"{path}: YAML nesting error near {raw}")
        parent = stack[-1][1]
        if value == "":
            child: dict[str, Any] = {}
            parent[key] = child
            stack.append((indent, child))
        else:
            parent[key] = parse_scalar(value)

    return root


def parse_scalar(value: str) -> Any:
    if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
        return value[1:-1]
    if value in {"true", "false"}:
        return value == "true"
    try:
        if "." in value:
            return float(value)
        return int(value)
    except ValueError:
        return value


def markdown_sections(path: Path) -> list[str]:
    headings = []
    for line in path.read_text().splitlines():
        if line.startswith("## "):
            headings.append(line[3:].strip())
    return headings


def build_symbol_table(tokens: dict[str, Any]) -> dict[str, Any]:
    symbols: dict[str, Any] = {}
    for group in ("colors", "typography", "rounded", "spacing"):
        values = tokens.get(group, {})
        if isinstance(values, dict):
            for name, value in values.items():
                symbols[f"{group}.{name}"] = value
    return symbols


def resolve_ref(value: Any, symbols: dict[str, Any]) -> Any:
    if not isinstance(value, str):
        return value
    match = TOKEN_REF_RE.match(value)
    if not match:
        return value
    ref = match.group(1)
    if ref not in symbols:
        fail(f"unresolved token reference {{{ref}}}")
    return symbols[ref]


def hex_to_rgb(hex_value: str) -> tuple[float, float, float]:
    if not HEX_RE.match(hex_value):
        fail(f"invalid color value {hex_value}")
    value = hex_value.lstrip("#")
    if len(value) in {3, 4}:
        value = "".join(ch * 2 for ch in value[:3])
    else:
        value = value[:6]
    return tuple(int(value[i : i + 2], 16) / 255 for i in (0, 2, 4))  # type: ignore[return-value]


def relative_luminance(hex_value: str) -> float:
    def channel(value: float) -> float:
        if value <= 0.03928:
            return value / 12.92
        return math.pow((value + 0.055) / 1.055, 2.4)

    r, g, b = hex_to_rgb(hex_value)
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)


def contrast_ratio(a: str, b: str) -> float:
    first = relative_luminance(a)
    second = relative_luminance(b)
    lighter = max(first, second)
    darker = min(first, second)
    return (lighter + 0.05) / (darker + 0.05)


def check_token_parity(profile_path: Path, profile_tokens: dict[str, Any], front: dict[str, Any]) -> None:
    for field in ("name", "description"):
        if profile_tokens.get(field) != front.get(field):
            fail(f"{profile_path}: front matter {field} does not match profile design_tokens.{field}")
    for group in ("colors", "typography", "rounded", "spacing", "components"):
        profile_keys = set((profile_tokens.get(group) or {}).keys())
        front_keys = set((front.get(group) or {}).keys())
        if profile_keys != front_keys:
            fail(f"{profile_path}: front matter {group} keys do not match profile design_tokens.{group}")


def check_design_tokens(profile_path: Path, tokens: dict[str, Any]) -> None:
    symbols = build_symbol_table(tokens)
    colors = tokens.get("colors", {})
    if "primary" not in colors:
        fail(f"{profile_path}: design_tokens.colors.primary is required")
    for color_name, color_value in colors.items():
        if not isinstance(color_value, str) or not HEX_RE.match(color_value):
            fail(f"{profile_path}: colors.{color_name} must be a hex color")

    components = tokens.get("components", {})
    if not isinstance(components, dict) or not components:
        fail(f"{profile_path}: design_tokens.components must not be empty")

    for component_name, component in components.items():
        normalized_name = component_name.lower()
        if any(term in normalized_name for term in PROHIBITED_TOKEN_TERMS):
            fail(f"{profile_path}: component name {component_name} implies governed authority")
        if not isinstance(component, dict):
            fail(f"{profile_path}: components.{component_name} must be an object")
        for prop_name, raw_value in component.items():
            if prop_name not in VALID_COMPONENT_PROPS:
                fail(f"{profile_path}: components.{component_name}.{prop_name} is not an allowed component property")
            try:
                resolve_ref(raw_value, symbols)
            except CheckFailure as exc:
                fail(f"{profile_path}: components.{component_name}.{prop_name}: {exc}")

        background = component.get("backgroundColor")
        text = component.get("textColor")
        if background and text:
            bg_value = resolve_ref(background, symbols)
            text_value = resolve_ref(text, symbols)
            if isinstance(bg_value, str) and isinstance(text_value, str) and HEX_RE.match(bg_value) and HEX_RE.match(text_value):
                ratio = contrast_ratio(bg_value, text_value)
                if ratio < 4.5:
                    fail(
                        f"{profile_path}: components.{component_name} contrast ratio {ratio:.2f}:1 is below WCAG AA 4.5:1"
                    )


def check_nostra_policy(profile_path: Path, profile: dict[str, Any], lineage_path: Path) -> None:
    scope = set(profile.get("surface_scope", []))
    if "constitutional_surface" in scope:
        fail(f"{profile_path}: surface_scope must not include constitutional_surface")

    tier_policy = profile.get("ndl_tier_policy", {})
    if tier_policy.get("tier1_components_allowed") is not False:
        fail(f"{profile_path}: tier1_components_allowed must be false")
    if tier_policy.get("verified_projection_required") is not True:
        fail(f"{profile_path}: verified_projection_required must be true")
    claims = set(tier_policy.get("prohibited_claims", []))
    missing_claims = sorted(REQUIRED_PROHIBITED_CLAIMS - claims)
    if missing_claims:
        fail(f"{profile_path}: prohibited_claims missing {missing_claims}")

    authority_mode = profile.get("authority_mode")
    approved_by = profile.get("approved_by", [])
    review_status = profile.get("stewardship", {}).get("review_status")
    if authority_mode == "recommendation_only" and approved_by:
        fail(f"{profile_path}: recommendation_only profiles must not list approved_by")
    if authority_mode in {"steward_approved", "runtime_enforced"} and not approved_by:
        fail(f"{profile_path}: approved profiles must list approved_by")
    if authority_mode == "runtime_enforced" and review_status != "approved":
        fail(f"{profile_path}: runtime_enforced requires stewardship.review_status approved")

    theme_policy = profile.get("a2ui_theme_policy", {})
    missing_theme_fields = sorted(REQUIRED_THEME_FIELDS - set(theme_policy.keys()))
    if missing_theme_fields:
        fail(f"{profile_path}: a2ui_theme_policy missing {missing_theme_fields}")
    if theme_policy.get("safe_mode") is not True and authority_mode == "recommendation_only":
        fail(f"{profile_path}: recommendation_only profiles must keep safe_mode true")

    if not lineage_path.exists():
        fail(f"{profile_path}: lineage_ref does not resolve: {profile.get('lineage_ref')}")


def check_sections(profile_path: Path, lineage_path: Path, profile: dict[str, Any]) -> None:
    actual = markdown_sections(lineage_path)
    known = [heading for heading in actual if heading in EXPECTED_SECTIONS]
    if known != EXPECTED_SECTIONS:
        fail(f"{profile_path}: SPACE_DESIGN.md sections out of order or incomplete: {known}")
    profile_sections = [section.get("heading") for section in profile.get("design_sections", [])]
    if profile_sections != EXPECTED_SECTIONS:
        fail(f"{profile_path}: design_sections headings out of order or incomplete: {profile_sections}")


def check_profile(profile_path: Path, schema_path: Path) -> None:
    profile = load_json(profile_path)
    validate_with_schema(schema_path, profile_path, profile)

    lineage_ref = profile.get("lineage_ref")
    if not isinstance(lineage_ref, str):
        fail(f"{profile_path}: lineage_ref must be a string")
    lineage_path = ROOT / lineage_ref

    check_nostra_policy(profile_path, profile, lineage_path)
    check_sections(profile_path, lineage_path, profile)

    front = read_front_matter(lineage_path)
    tokens = profile.get("design_tokens", {})
    check_token_parity(profile_path, tokens, front)
    check_design_tokens(profile_path, tokens)


def check_design_import(import_path: Path, schema_path: Path) -> None:
    design_import = load_json(import_path)
    validate_with_schema(schema_path, import_path, design_import)
    if design_import.get("authority_mode") != "recommendation_only":
        fail(f"{import_path}: design imports must remain recommendation_only")
    required_checks = set(design_import.get("required_checks", []))
    if design_import.get("adoption_status") == "candidate" and "license_or_lineage" not in required_checks:
        fail(f"{import_path}: candidate imports must require license_or_lineage")
    provenance_ref = design_import.get("provenance_ref")
    if not isinstance(provenance_ref, str):
        fail(f"{import_path}: provenance_ref must be a string")
    provenance_path = ROOT / provenance_ref
    if not provenance_path.exists():
        fail(f"{import_path}: provenance_ref does not resolve: {provenance_ref}")


def check_template_pack(template_path: Path, schema_path: Path) -> None:
    template = load_json(template_path)
    validate_with_schema(schema_path, template_path, template)
    if template.get("authority_mode") != "recommendation_only":
        fail(f"{template_path}: template packs must remain recommendation_only")
    profile_defaults_ref = template.get("profile_defaults_ref")
    if not isinstance(profile_defaults_ref, str):
        fail(f"{template_path}: profile_defaults_ref must be a string")
    profile_defaults_path = ROOT / profile_defaults_ref
    if not profile_defaults_path.exists():
        fail(f"{template_path}: profile_defaults_ref does not resolve: {profile_defaults_ref}")
    if "constitutional_surface" in set(template.get("allowed_surface_scope", [])):
        fail(f"{template_path}: allowed_surface_scope must not include constitutional_surface")
    for import_ref in template.get("included_import_refs", []):
        import_path = ROOT / import_ref
        if not import_path.exists():
            fail(f"{template_path}: included_import_ref does not resolve: {import_ref}")


def profile_paths(args: argparse.Namespace) -> list[Path]:
    if args.profiles:
        return [Path(path) if Path(path).is_absolute() else ROOT / path for path in args.profiles]
    return sorted(ROOT.glob(DEFAULT_PROFILE_GLOB))


def import_paths(args: argparse.Namespace) -> list[Path]:
    return [Path(path) if Path(path).is_absolute() else ROOT / path for path in args.imports]


def template_paths(args: argparse.Namespace) -> list[Path]:
    return [Path(path) if Path(path).is_absolute() else ROOT / path for path in args.templates]


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate Space design contract prototypes.")
    parser.add_argument("profiles", nargs="*", help="Profile JSON files. Defaults to Initiative 120 prototypes.")
    parser.add_argument("--schema", default=str(DEFAULT_SCHEMA), help="SpaceDesignProfileV1 JSON schema path.")
    parser.add_argument("--import-schema", default=str(DEFAULT_IMPORT_SCHEMA), help="DesignElementImportV1 schema path.")
    parser.add_argument("--template-schema", default=str(DEFAULT_TEMPLATE_SCHEMA), help="SpaceTemplatePackV1 schema path.")
    parser.add_argument("--imports", nargs="*", default=None, help="Design import JSON files. Defaults to prototypes.")
    parser.add_argument("--templates", nargs="*", default=None, help="Template pack JSON files. Defaults to prototypes.")
    args = parser.parse_args()

    schema_path = Path(args.schema)
    if not schema_path.is_absolute():
        schema_path = ROOT / schema_path
    if not schema_path.exists():
        print(f"FAIL: missing schema {schema_path}", file=sys.stderr)
        return 1
    import_schema_path = Path(args.import_schema)
    if not import_schema_path.is_absolute():
        import_schema_path = ROOT / import_schema_path
    if not import_schema_path.exists():
        print(f"FAIL: missing schema {import_schema_path}", file=sys.stderr)
        return 1
    template_schema_path = Path(args.template_schema)
    if not template_schema_path.is_absolute():
        template_schema_path = ROOT / template_schema_path
    if not template_schema_path.exists():
        print(f"FAIL: missing schema {template_schema_path}", file=sys.stderr)
        return 1

    profiles = profile_paths(args)
    if not profiles:
        print("FAIL: no Space design profiles found", file=sys.stderr)
        return 1
    imports = sorted(ROOT.glob(DEFAULT_IMPORT_GLOB)) if args.imports is None else import_paths(args)
    templates = sorted(ROOT.glob(DEFAULT_TEMPLATE_GLOB)) if args.templates is None else template_paths(args)

    try:
        for path in profiles:
            if not path.exists():
                fail(f"missing profile {path}")
            check_profile(path, schema_path)
        for path in imports:
            if not path.exists():
                fail(f"missing design import {path}")
            check_design_import(path, import_schema_path)
        for path in templates:
            if not path.exists():
                fail(f"missing template pack {path}")
            check_template_pack(path, template_schema_path)
    except CheckFailure as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    print(
        "PASS: Space design contract checks "
        f"({len(profiles)} profile(s), {len(imports)} import(s), {len(templates)} template pack(s))"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
