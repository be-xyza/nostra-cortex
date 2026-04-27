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
DEFAULT_A2UI_THEME_DIR = ROOT / "shared/a2ui/themes"
DEFAULT_A2UI_FIXTURE_DIR = ROOT / "shared/a2ui/fixtures"
DEFAULT_DOMAIN_THEME_POLICY = ROOT / "cortex/libraries/cortex-domain/src/theme/policy.rs"
DEFAULT_EUDAEMON_THEME_POLICY = ROOT / "cortex/apps/cortex-eudaemon/src/services/theme_policy.rs"
DEFAULT_CORTEX_WEB_SPACE_DESIGN_FIXTURE = (
    ROOT / "cortex/apps/cortex-web/src/store/spaceDesignProfilePreview.fixture.json"
)

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
REQUIRED_ACCESSIBILITY_CHECKS = {
    "contrast meets WCAG AA",
    "reduced motion is governed",
    "focus visibility is preserved",
    "keyboard reachability is preserved",
    "text fit is bounded",
    "color is not sole state channel",
}
REQUIRED_IMPORT_CHECKS = {
    "locked_reality_snapshot",
    "brand_policy",
    "accessibility",
    "ndl_surface_boundary",
    "a2ui_theme_policy",
    "space_capability_alignment",
    "license_or_lineage",
}
REQUIRED_TEMPLATE_GATE_CHECKS = {
    "locked_reality_snapshot",
    "space_design_profile",
    "design_element_imports",
    "brand_policy",
    "accessibility",
    "a2ui_theme_policy",
    "space_capability_alignment",
    "hermes_advisory_only",
}
ALLOWED_IMPORT_STATUSES = {
    "candidate",
    "adapt_only",
    "rejected",
    "needs_steward_review",
}
PROHIBITED_ELEVATION_TERMS = {
    "runtime_contract",
    "runtime-enforced",
    "runtime_enforced",
    "approved",
    "steward_approved",
    "steward-approved",
}
PROHIBITED_PROFILE_TOKEN_CLAIM_TERMS = PROHIBITED_TOKEN_TERMS | {
    "allowlist",
    "theme_allowlist",
    "runtime_contract",
    "runtime-enforced",
    "runtime_enforced",
    "tier1",
    "tier_1",
}
STATUS_COLOR_KEYS = {
    "evidence",
    "warning",
    "boundary",
    "tertiary",
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
DIMENSION_RE = re.compile(r"^-?\d*\.?\d+(px|em|rem)$")


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


def parse_dimension(value: Any) -> tuple[float, str] | None:
    if not isinstance(value, str):
        return None
    match = DIMENSION_RE.match(value)
    if not match:
        return None
    return float(value[: -len(match.group(1))]), match.group(1)


def dimension_to_px(value: Any, base_px: float = 16.0) -> float | None:
    parsed = parse_dimension(value)
    if parsed is None:
        return None
    amount, unit = parsed
    if unit == "px":
        return amount
    return amount * base_px


def check_typography_accessibility(profile_path: Path, tokens: dict[str, Any]) -> None:
    typography = tokens.get("typography", {})
    if not isinstance(typography, dict) or not typography:
        fail(f"{profile_path}: design_tokens.typography must not be empty")

    for token_name, token in typography.items():
        if not isinstance(token, dict):
            fail(f"{profile_path}: typography.{token_name} must be an object")

        font_size_px = dimension_to_px(token.get("fontSize"))
        if font_size_px is None:
            fail(f"{profile_path}: typography.{token_name}.fontSize must use px, em, or rem")
        if font_size_px < 12:
            fail(f"{profile_path}: typography.{token_name}.fontSize {font_size_px:g}px is below the 12px readability floor")

        line_height = token.get("lineHeight")
        min_line_height = 1.05 if font_size_px >= 20 else 1.2
        if isinstance(line_height, (int, float)) and line_height < min_line_height:
            fail(
                f"{profile_path}: typography.{token_name}.lineHeight {line_height:g} "
                f"is below the {min_line_height:g} readability floor"
            )
        line_height_px = dimension_to_px(line_height)
        if line_height_px is not None and line_height_px < font_size_px * min_line_height:
            fail(f"{profile_path}: typography.{token_name}.lineHeight is below {min_line_height:g}x fontSize")

        letter_spacing = token.get("letterSpacing")
        parsed_spacing = parse_dimension(letter_spacing)
        if parsed_spacing is None:
            fail(f"{profile_path}: typography.{token_name}.letterSpacing must use px, em, or rem")
        if parsed_spacing[0] < 0:
            fail(f"{profile_path}: typography.{token_name}.letterSpacing must not be negative")


def check_status_color_accessibility(profile_path: Path, tokens: dict[str, Any]) -> None:
    colors = tokens.get("colors", {})
    if not isinstance(colors, dict):
        return
    backgrounds = [colors.get("surface"), colors.get("neutral")]
    backgrounds = [value for value in backgrounds if isinstance(value, str) and HEX_RE.match(value)]
    for color_name in sorted(STATUS_COLOR_KEYS & set(colors.keys())):
        color_value = colors[color_name]
        if not isinstance(color_value, str) or not HEX_RE.match(color_value):
            continue
        if not backgrounds:
            fail(f"{profile_path}: status color {color_name} requires surface or neutral background for contrast checks")
        best_ratio = max(contrast_ratio(color_value, background) for background in backgrounds)
        if best_ratio < 3.0:
            fail(f"{profile_path}: status color {color_name} contrast {best_ratio:.2f}:1 is below non-text 3.0:1")


def check_layout_accessibility(profile_path: Path, tokens: dict[str, Any]) -> None:
    spacing = tokens.get("spacing", {})
    if not isinstance(spacing, dict):
        fail(f"{profile_path}: design_tokens.spacing must be an object")
    measure = spacing.get("measure")
    if measure is None:
        fail(f"{profile_path}: design_tokens.spacing.measure is required for text-fit bounds")
    if not isinstance(measure, (int, float)):
        fail(f"{profile_path}: design_tokens.spacing.measure must be numeric")
    if measure > 80:
        fail(f"{profile_path}: design_tokens.spacing.measure {measure:g} exceeds the 80 character readability bound")


def collect_token_claim_terms(value: Any, path: str = "design_tokens") -> list[str]:
    violations = []
    if isinstance(value, dict):
        for key, child in value.items():
            key_path = f"{path}.{key}"
            lowered_key = str(key).lower()
            for term in PROHIBITED_PROFILE_TOKEN_CLAIM_TERMS:
                if term in lowered_key:
                    violations.append(f"{key_path} contains {term}")
            violations.extend(collect_token_claim_terms(child, key_path))
    elif isinstance(value, list):
        for idx, child in enumerate(value):
            violations.extend(collect_token_claim_terms(child, f"{path}[{idx}]"))
    elif isinstance(value, str):
        lowered_value = value.lower()
        for term in PROHIBITED_PROFILE_TOKEN_CLAIM_TERMS:
            if term in lowered_value:
                violations.append(f"{path} contains {term}")
    return violations


def check_profile_token_claims(profile_path: Path, tokens: dict[str, Any]) -> None:
    violations = collect_token_claim_terms(tokens)
    if violations:
        fail(
            f"{profile_path}: design_tokens must not claim theme allowlist, runtime, "
            f"or Tier 1 governance authority: {violations[:6]}"
        )


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
    check_profile_token_claims(profile_path, tokens)
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

        if any(term in normalized_name for term in ("evidence", "warning", "boundary")):
            if "typography" not in component or "textColor" not in component:
                fail(f"{profile_path}: components.{component_name} must not rely on color alone for state communication")

    check_typography_accessibility(profile_path, tokens)
    check_status_color_accessibility(profile_path, tokens)
    check_layout_accessibility(profile_path, tokens)


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
    if authority_mode == "recommendation_only" and theme_policy.get("motion_policy") == "full":
        fail(f"{profile_path}: recommendation_only profiles must not use full motion policy")

    lint_contract = profile.get("lint_contract", {})
    required_local_checks = set(lint_contract.get("required_local_checks", []))
    missing_accessibility = sorted(REQUIRED_ACCESSIBILITY_CHECKS - required_local_checks)
    if missing_accessibility:
        fail(f"{profile_path}: lint_contract.required_local_checks missing accessibility checks {missing_accessibility}")

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


def load_a2ui_theme_truth(theme_dir: Path) -> dict[str, set[str]]:
    if not theme_dir.exists():
        fail(f"missing A2UI theme directory {theme_dir}")
    theme_paths = sorted(theme_dir.glob("*.json"))
    if not theme_paths:
        fail(f"no A2UI theme files found under {theme_dir}")

    truth = {
        "theme_names": set(),
        "runtime_allowlist_ids": set(),
        "supported_token_versions": set(),
        "motion_policies": {"system", "reduced", "full"},
        "contrast_preferences": {"system", "more", "less"},
    }
    for theme_path in theme_paths:
        theme = load_json(theme_path)
        if not isinstance(theme, dict):
            fail(f"{theme_path}: A2UI theme must be an object")
        name = theme.get("name")
        if not isinstance(name, str) or not name:
            fail(f"{theme_path}: A2UI theme missing name")
        truth["theme_names"].add(name)

        policy = theme.get("policy", {})
        if not isinstance(policy, dict):
            fail(f"{theme_path}: A2UI theme policy must be an object")
        allowlist_id = policy.get("default_allowlist_id")
        if isinstance(allowlist_id, str) and allowlist_id:
            truth["runtime_allowlist_ids"].add(allowlist_id)
        supported_version = policy.get("supported_token_version")
        if isinstance(supported_version, str) and supported_version:
            truth["supported_token_versions"].add(supported_version)
        for field, allowed_key in (
            ("default_motion_policy", "motion_policies"),
            ("default_contrast_preference", "contrast_preferences"),
        ):
            value = policy.get(field)
            if isinstance(value, str) and value:
                truth[allowed_key].add(value)

    if not truth["supported_token_versions"]:
        fail(f"{theme_dir}: A2UI themes do not declare supported token versions")
    return truth


def check_a2ui_fixture_truth(fixture_dir: Path, truth: dict[str, set[str]]) -> int:
    if not fixture_dir.exists():
        fail(f"missing A2UI fixture directory {fixture_dir}")
    fixture_paths = sorted(fixture_dir.glob("*.json"))
    if not fixture_paths:
        fail(f"no A2UI fixtures found under {fixture_dir}")

    checked = 0
    for fixture_path in fixture_paths:
        fixture = load_json(fixture_path)
        if not isinstance(fixture, dict):
            fail(f"{fixture_path}: A2UI fixture must be an object")
        meta = fixture.get("meta", {})
        if not isinstance(meta, dict) or "theme" not in meta:
            continue
        checked += 1
        theme_name = meta.get("theme")
        if theme_name not in truth["theme_names"]:
            fail(f"{fixture_path}: fixture theme {theme_name!r} is not present in shared/a2ui/themes")
        token_version = meta.get("token_version")
        if token_version not in truth["supported_token_versions"]:
            fail(f"{fixture_path}: fixture token_version {token_version!r} is not supported by A2UI themes")
        if meta.get("safe_mode") is not True:
            fail(f"{fixture_path}: themed A2UI fixtures must keep safe_mode true")
        allowlist_id = meta.get("theme_allowlist_id")
        if isinstance(allowlist_id, str) and allowlist_id:
            truth["runtime_allowlist_ids"].add(allowlist_id)
        motion_policy = meta.get("motion_policy")
        if motion_policy not in truth["motion_policies"]:
            fail(f"{fixture_path}: fixture motion_policy {motion_policy!r} is not accepted by theme policy")
        contrast_preference = meta.get("contrast_preference")
        if contrast_preference not in truth["contrast_preferences"]:
            fail(f"{fixture_path}: fixture contrast_preference {contrast_preference!r} is not accepted by theme policy")

    if checked == 0:
        fail(f"{fixture_dir}: no themed A2UI fixture metadata found")
    return checked


def check_rust_theme_policy(domain_policy_path: Path, eudaemon_policy_path: Path) -> None:
    for path in (domain_policy_path, eudaemon_policy_path):
        if not path.exists():
            fail(f"missing theme policy source {path}")
    domain_text = domain_policy_path.read_text()
    eudaemon_text = eudaemon_policy_path.read_text()
    for value in ("system", "reduced", "full", "more", "less"):
        if f'"{value}"' not in domain_text:
            fail(f"{domain_policy_path}: theme policy normalization missing {value!r}")
    if "ThemeName::Cortex" not in eudaemon_text:
        fail(f"{eudaemon_policy_path}: desktop theme default must remain tied to the Cortex theme")
    if "domain_theme_policy::normalize_preferences" not in eudaemon_text:
        fail(f"{eudaemon_policy_path}: desktop theme policy must delegate normalization to cortex-domain")


def build_a2ui_truth(
    theme_dir: Path,
    fixture_dir: Path,
    domain_policy_path: Path,
    eudaemon_policy_path: Path,
) -> tuple[dict[str, set[str]], int]:
    truth = load_a2ui_theme_truth(theme_dir)
    fixture_count = check_a2ui_fixture_truth(fixture_dir, truth)
    check_rust_theme_policy(domain_policy_path, eudaemon_policy_path)
    return truth, fixture_count


def check_profile_a2ui_compatibility(profile_path: Path, profile: dict[str, Any], truth: dict[str, set[str]]) -> None:
    if profile.get("authority_mode") == "runtime_enforced":
        fail(f"{profile_path}: Space design profile fixture validation does not authorize runtime_enforced profiles")
    tier_policy = profile.get("ndl_tier_policy", {})
    if tier_policy.get("tier1_components_allowed") is not False:
        fail(f"{profile_path}: Space design profile fixture validation does not authorize Tier 1 components")

    theme_policy = profile.get("a2ui_theme_policy", {})
    token_version = theme_policy.get("token_version")
    if token_version in truth["supported_token_versions"]:
        fail(
            f"{profile_path}: a2ui_theme_policy.token_version must remain an NDL recommendation marker, "
            "not a runtime A2UI token version"
        )
    allowlist_id = theme_policy.get("theme_allowlist_id")
    if allowlist_id in truth["runtime_allowlist_ids"]:
        fail(
            f"{profile_path}: a2ui_theme_policy.theme_allowlist_id {allowlist_id!r} "
            "reuses a runtime or fixture allowlist id"
        )
    if theme_policy.get("motion_policy") not in truth["motion_policies"]:
        fail(f"{profile_path}: a2ui_theme_policy.motion_policy is not accepted by A2UI theme policy")
    if theme_policy.get("contrast_preference") not in truth["contrast_preferences"]:
        fail(f"{profile_path}: a2ui_theme_policy.contrast_preference is not accepted by A2UI theme policy")


def contains_key(value: Any, key: str) -> bool:
    if isinstance(value, dict):
        return key in value or any(contains_key(child, key) for child in value.values())
    if isinstance(value, list):
        return any(contains_key(child, key) for child in value)
    return False


def check_cortex_web_space_design_fixture(
    fixture_path: Path,
    profiles: dict[str, dict[str, Any]],
    truth: dict[str, set[str]],
) -> None:
    if not fixture_path.exists():
        fail(f"missing Cortex Web Space design preview fixture {fixture_path}")
    fixture = load_json(fixture_path)
    if not isinstance(fixture, dict):
        fail(f"{fixture_path}: preview fixture must be an object")
    if fixture.get("schema_version") != "CortexWebSpaceDesignProfilePreviewFixtureV1":
        fail(f"{fixture_path}: unexpected schema_version {fixture.get('schema_version')!r}")
    if fixture.get("snapshot_id") != "system:ux:space-design-profiles":
        fail(f"{fixture_path}: snapshot_id must remain system:ux:space-design-profiles")
    if fixture.get("source_mode") != "fixture":
        fail(f"{fixture_path}: source_mode must remain fixture")
    if fixture.get("runtime_binding") != "none":
        fail(f"{fixture_path}: runtime_binding must remain none")
    runtime_policy = fixture.get("runtime_policy", {})
    expected_runtime_policy = {
        "recommendation_only": True,
        "applies_tokens_to_cortex_web": False,
        "runtime_theme_selection": False,
        "requires_verified_projection_for_governance": True,
    }
    for key, expected in expected_runtime_policy.items():
        if runtime_policy.get(key) is not expected:
            fail(f"{fixture_path}: runtime_policy.{key} must be {expected!r}")
    if contains_key(fixture, "design_tokens"):
        fail(f"{fixture_path}: Cortex Web preview fixture must not carry design_tokens")

    profile_items = fixture.get("profiles", [])
    if not isinstance(profile_items, list) or not profile_items:
        fail(f"{fixture_path}: profiles must contain at least one preview profile")

    seen_profile_ids = set()
    for item in profile_items:
        if not isinstance(item, dict):
            fail(f"{fixture_path}: profile preview entries must be objects")
        profile_id = item.get("profile_id")
        if profile_id not in profiles:
            fail(f"{fixture_path}: profile preview {profile_id!r} does not match a Space design profile")
        seen_profile_ids.add(profile_id)
        source_profile = profiles[profile_id]
        if item.get("authority_mode") != "recommendation_only":
            fail(f"{fixture_path}: profile preview {profile_id} must remain recommendation_only")
        if item.get("authority_mode") != source_profile.get("authority_mode"):
            fail(f"{fixture_path}: profile preview {profile_id} authority_mode drifted from source profile")
        if item.get("review_status") != source_profile.get("stewardship", {}).get("review_status"):
            fail(f"{fixture_path}: profile preview {profile_id} review_status drifted from source profile")
        if item.get("approved_by_count") != len(source_profile.get("approved_by", [])):
            fail(f"{fixture_path}: profile preview {profile_id} approved_by_count drifted from source profile")
        if item.get("surface_scope") != source_profile.get("surface_scope"):
            fail(f"{fixture_path}: profile preview {profile_id} surface_scope drifted from source profile")
        if item.get("preview_status") != "metadata_only":
            fail(f"{fixture_path}: profile preview {profile_id} must remain metadata_only")
        theme_policy = item.get("a2ui_theme_policy", {})
        if theme_policy != source_profile.get("a2ui_theme_policy"):
            fail(f"{fixture_path}: profile preview {profile_id} a2ui_theme_policy drifted from source profile")
        if theme_policy.get("token_version") in truth["supported_token_versions"]:
            fail(f"{fixture_path}: profile preview {profile_id} reuses a runtime A2UI token_version")
        if theme_policy.get("theme_allowlist_id") in truth["runtime_allowlist_ids"]:
            fail(f"{fixture_path}: profile preview {profile_id} reuses a runtime A2UI allowlist id")

    missing_profile_ids = sorted(set(profiles) - seen_profile_ids)
    if missing_profile_ids:
        fail(f"{fixture_path}: missing Space design profile preview entries {missing_profile_ids}")


def check_profile(profile_path: Path, schema_path: Path, a2ui_truth: dict[str, set[str]]) -> None:
    profile = load_json(profile_path)
    validate_with_schema(schema_path, profile_path, profile)

    lineage_ref = profile.get("lineage_ref")
    if not isinstance(lineage_ref, str):
        fail(f"{profile_path}: lineage_ref must be a string")
    lineage_path = ROOT / lineage_ref

    check_nostra_policy(profile_path, profile, lineage_path)
    check_profile_a2ui_compatibility(profile_path, profile, a2ui_truth)
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
    if design_import.get("source_authority") == "steward_reviewed":
        fail(f"{import_path}: design imports must not claim steward_reviewed before a promotion gate")
    adoption_status = design_import.get("adoption_status")
    if adoption_status not in ALLOWED_IMPORT_STATUSES:
        fail(f"{import_path}: adoption_status must be one of {sorted(ALLOWED_IMPORT_STATUSES)}")
    required_checks = set(design_import.get("required_checks", []))
    missing_checks = sorted(REQUIRED_IMPORT_CHECKS - required_checks)
    if missing_checks:
        fail(f"{import_path}: required_checks missing promotion-gate checks {missing_checks}")
    provenance_ref = design_import.get("provenance_ref")
    if not isinstance(provenance_ref, str):
        fail(f"{import_path}: provenance_ref must be a string")
    provenance_path = ROOT / provenance_ref
    if not provenance_path.exists():
        fail(f"{import_path}: provenance_ref does not resolve: {provenance_ref}")
    candidate_materials = design_import.get("candidate_materials", [])
    for material in candidate_materials:
        text = " ".join(str(material.get(field, "")) for field in ("material_id", "material_kind", "description"))
        lowered = text.lower()
        if any(term in lowered for term in PROHIBITED_ELEVATION_TERMS):
            fail(f"{import_path}: candidate material {material.get('material_id')} implies promoted runtime authority")


def check_template_pack(template_path: Path, schema_path: Path) -> None:
    template = load_json(template_path)
    validate_with_schema(schema_path, template_path, template)
    if template.get("authority_mode") != "recommendation_only":
        fail(f"{template_path}: template packs must remain recommendation_only")
    promotion_gate = template.get("promotion_gate", {})
    if promotion_gate.get("required_recommendation") != "needs_steward_review":
        fail(f"{template_path}: promotion_gate.required_recommendation must be needs_steward_review for draft packs")
    gate_checks = set(promotion_gate.get("required_checks", []))
    missing_gate_checks = sorted(REQUIRED_TEMPLATE_GATE_CHECKS - gate_checks)
    if missing_gate_checks:
        fail(f"{template_path}: promotion_gate.required_checks missing {missing_gate_checks}")
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
        design_import = load_json(import_path)
        if design_import.get("authority_mode") != "recommendation_only":
            fail(f"{template_path}: included import {import_ref} is not recommendation_only")
        if design_import.get("adoption_status") not in {"candidate", "adapt_only", "needs_steward_review"}:
            fail(f"{template_path}: included import {import_ref} has incompatible adoption_status")


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
    parser.add_argument("--a2ui-theme-dir", default=str(DEFAULT_A2UI_THEME_DIR), help="A2UI theme directory used as effective theme truth.")
    parser.add_argument("--a2ui-fixture-dir", default=str(DEFAULT_A2UI_FIXTURE_DIR), help="A2UI render fixture directory used as effective render truth.")
    parser.add_argument(
        "--domain-theme-policy",
        default=str(DEFAULT_DOMAIN_THEME_POLICY),
        help="cortex-domain Rust theme policy source used for normalization truth.",
    )
    parser.add_argument(
        "--eudaemon-theme-policy",
        default=str(DEFAULT_EUDAEMON_THEME_POLICY),
        help="cortex-eudaemon Rust theme policy adapter source used for default policy truth.",
    )
    parser.add_argument(
        "--cortex-web-space-design-fixture",
        default=str(DEFAULT_CORTEX_WEB_SPACE_DESIGN_FIXTURE),
        help="Cortex Web preview fixture used to expose Space design profile metadata.",
    )
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
    a2ui_theme_dir = Path(args.a2ui_theme_dir)
    if not a2ui_theme_dir.is_absolute():
        a2ui_theme_dir = ROOT / a2ui_theme_dir
    a2ui_fixture_dir = Path(args.a2ui_fixture_dir)
    if not a2ui_fixture_dir.is_absolute():
        a2ui_fixture_dir = ROOT / a2ui_fixture_dir
    domain_theme_policy_path = Path(args.domain_theme_policy)
    if not domain_theme_policy_path.is_absolute():
        domain_theme_policy_path = ROOT / domain_theme_policy_path
    eudaemon_theme_policy_path = Path(args.eudaemon_theme_policy)
    if not eudaemon_theme_policy_path.is_absolute():
        eudaemon_theme_policy_path = ROOT / eudaemon_theme_policy_path
    cortex_web_space_design_fixture_path = Path(args.cortex_web_space_design_fixture)
    if not cortex_web_space_design_fixture_path.is_absolute():
        cortex_web_space_design_fixture_path = ROOT / cortex_web_space_design_fixture_path

    profiles = profile_paths(args)
    if not profiles:
        print("FAIL: no Space design profiles found", file=sys.stderr)
        return 1
    imports = sorted(ROOT.glob(DEFAULT_IMPORT_GLOB)) if args.imports is None else import_paths(args)
    templates = sorted(ROOT.glob(DEFAULT_TEMPLATE_GLOB)) if args.templates is None else template_paths(args)

    try:
        a2ui_truth, a2ui_fixture_count = build_a2ui_truth(
            a2ui_theme_dir,
            a2ui_fixture_dir,
            domain_theme_policy_path,
            eudaemon_theme_policy_path,
        )
        for path in profiles:
            if not path.exists():
                fail(f"missing profile {path}")
            check_profile(path, schema_path, a2ui_truth)
        profile_records = {}
        for path in profiles:
            profile = load_json(path)
            profile_records[profile["profile_id"]] = profile
        check_cortex_web_space_design_fixture(
            cortex_web_space_design_fixture_path,
            profile_records,
            a2ui_truth,
        )
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
        f"({len(profiles)} profile(s), {len(imports)} import(s), {len(templates)} template pack(s), "
        f"{len(a2ui_truth['theme_names'])} A2UI theme(s), {a2ui_fixture_count} themed fixture(s))"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
