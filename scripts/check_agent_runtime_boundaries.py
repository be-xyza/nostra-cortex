#!/usr/bin/env python3
"""Validate live runtime/profile configs against declared agent boundaries."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BOUNDARIES = (
    ROOT
    / "research"
    / "132-eudaemon-alpha-initiative"
    / "examples"
    / "agent_runtime_boundaries.v1.json"
)


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def get_path(payload: dict[str, Any], path: str) -> Any:
    current: Any = payload
    for part in path.split("."):
        if not isinstance(current, dict) or part not in current:
            return None
        current = current[part]
    return current


def resolve_profile_dir(boundary: dict[str, Any]) -> Path:
    env_name = boundary.get("profileDirEnv")
    if env_name and os.environ.get(env_name):
        return Path(os.environ[env_name]).expanduser()
    return Path(boundary["defaultProfileDir"]).expanduser()


def validate_hermes_profile(boundary: dict[str, Any]) -> list[str]:
    agent_id = boundary["agentId"]
    profile_dir = resolve_profile_dir(boundary)
    config_path = profile_dir / boundary["configFile"]
    charter_path = profile_dir / boundary["charterFile"]
    if not config_path.exists() or not charter_path.exists():
        return [f"SKIP: {agent_id} profile not present at {profile_dir}"]

    config = yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
    charter = charter_path.read_text(encoding="utf-8").lower()

    disabled = set(get_path(config, "agent.disabled_toolsets") or [])
    required_disabled = set(boundary.get("requiredDisabledToolsets", []))
    missing_disabled = sorted(required_disabled - disabled)
    if missing_disabled:
        raise SystemExit(f"FAIL: {agent_id} must disable toolsets: {missing_disabled}")

    cli_tools = set(get_path(config, "platform_toolsets.cli") or [])
    active_forbidden = sorted(set(boundary.get("forbiddenCliToolsets", [])) & cli_tools)
    if active_forbidden:
        raise SystemExit(f"FAIL: {agent_id} cli toolsets include forbidden tools: {active_forbidden}")

    provider = str(get_path(config, "model.provider") or "").strip().lower()
    if provider in set(boundary.get("forbiddenModelProviders", [])):
        raise SystemExit(f"FAIL: {agent_id} model provider is forbidden: {provider}")

    for assertion in boundary.get("requiredConfigAssertions", []):
        actual = get_path(config, assertion["path"])
        if actual != assertion["equals"]:
            raise SystemExit(
                f"FAIL: {agent_id} config assertion failed for {assertion['path']}: "
                f"expected {assertion['equals']!r}, got {actual!r}"
            )

    missing_phrases = [phrase for phrase in boundary.get("requiredCharterPhrases", []) if phrase.lower() not in charter]
    if missing_phrases:
        raise SystemExit(f"FAIL: {agent_id} charter missing guardrail phrases: {missing_phrases}")

    return [f"PASS: {agent_id} runtime boundary"]


def validate_systemd_service(boundary: dict[str, Any]) -> list[str]:
    agent_id = boundary["agentId"]
    service_path = Path(boundary["servicePath"])
    if not service_path.is_absolute():
        service_path = ROOT / service_path
    if not service_path.exists():
        raise SystemExit(f"FAIL: {agent_id} systemd service missing: {service_path}")
    text = service_path.read_text(encoding="utf-8")
    for required in boundary.get("requiredContains", []):
        if required not in text:
            raise SystemExit(f"FAIL: {agent_id} systemd service missing required text: {required}")
    lowered = text.lower()
    for forbidden in boundary.get("forbiddenContains", []):
        if forbidden.lower() in lowered:
            raise SystemExit(f"FAIL: {agent_id} systemd service contains forbidden text: {forbidden}")
    return [f"PASS: {agent_id} service boundary"]


def validate_discovery(payload: dict[str, Any]) -> list[str]:
    discovery = payload.get("discovery", {})
    boundaries = payload.get("boundaries", [])
    boundary_agents = {boundary["agentId"] for boundary in boundaries}
    messages: list[str] = []

    profiles_root = Path(discovery.get("hermesProfilesRoot", "~/.hermes/profiles")).expanduser()
    prefixes = tuple(discovery.get("projectHermesProfilePrefixes", []))
    ignored_profiles = set(discovery.get("ignoredHermesProfiles", []))
    if profiles_root.exists() and prefixes:
        for profile_dir in sorted(path for path in profiles_root.iterdir() if path.is_dir()):
            name = profile_dir.name
            if name in ignored_profiles:
                continue
            if name.startswith(prefixes) and name not in boundary_agents:
                raise SystemExit(f"FAIL: discovered project Hermes profile without runtime boundary: {name}")
        messages.append("PASS: Hermes profile discovery")

    systemd_root = Path(discovery.get("systemdRoot", "ops/hetzner/systemd"))
    if not systemd_root.is_absolute():
        systemd_root = ROOT / systemd_root
    agent_like = set(discovery.get("agentLikeSystemdServices", []))
    ignored_services = set(discovery.get("ignoredSystemdServices", []))
    covered_services = {
        Path(boundary["servicePath"]).name
        for boundary in boundaries
        if boundary.get("runtimeKind") == "systemd_service"
    }
    if systemd_root.exists():
        for service in sorted(systemd_root.glob("cortex-*.service")):
            name = service.name
            if name in ignored_services:
                continue
            if name in agent_like and name not in covered_services:
                raise SystemExit(f"FAIL: discovered agent-like systemd service without runtime boundary: {name}")
            if name not in ignored_services and name not in agent_like:
                raise SystemExit(f"FAIL: unclassified Cortex systemd service: {name}")
        messages.append("PASS: Cortex service discovery")

    return messages


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_BOUNDARIES
    if not path.is_absolute():
        path = ROOT / path
    payload = load_json(path)
    if payload.get("schemaVersion") != "1.0.0":
        raise SystemExit("FAIL: boundary set must use schemaVersion 1.0.0")

    messages: list[str] = []
    for boundary in payload.get("boundaries", []):
        if boundary.get("runtimeKind") == "hermes_profile":
            messages.extend(validate_hermes_profile(boundary))
        elif boundary.get("runtimeKind") == "systemd_service":
            messages.extend(validate_systemd_service(boundary))
        else:
            raise SystemExit(f"FAIL: unsupported runtimeKind {boundary.get('runtimeKind')}")
    messages.extend(validate_discovery(payload))

    for message in messages:
        print(message)
    print("PASS: agent runtime boundary checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
