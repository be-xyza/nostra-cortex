#!/usr/bin/env python3
"""Generate a non-mutating D1 developer handoff from an approved WorkRouter bundle."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "research" / "132-eudaemon-alpha-initiative" / "schemas"
CODE_CHANGE_SCHEMA = SCHEMA_DIR / "CodeChangeRequestV1.schema.json"

REQUIRED_FORBIDDEN = {
    "repo_mutation",
    "runtime_mutation",
    "commit",
    "push",
    "deploy",
}


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def extract_code_change(bundle: dict) -> dict:
    code_change = bundle.get("codeChangeRequest")
    require(isinstance(code_change, dict), "bundle missing codeChangeRequest")
    jsonschema.validate(code_change, load_json(CODE_CHANGE_SCHEMA))
    require(code_change["mode"] == "patch_prep", "handoff generation only supports patch_prep mode")
    require(code_change["authorityLevel"] == "D1", "handoff generation only supports D1 authority")
    forbidden = set(code_change["forbiddenActions"])
    missing = sorted(REQUIRED_FORBIDDEN - forbidden)
    require(not missing, f"codeChangeRequest missing required forbidden actions: {', '.join(missing)}")
    return code_change


def bullet_list(items: list[str]) -> str:
    return "\n".join(f"- `{item}`" for item in items) if items else "- none"


def checkbox_list(items: list[str]) -> str:
    return "\n".join(f"- [ ] `{item}`" for item in items) if items else "- [ ] no checks declared"


def render_handoff(bundle: dict) -> str:
    code_change = extract_code_change(bundle)
    router = bundle.get("workRouterDecision", {})
    request = bundle.get("dispatchRequest", {})
    decision = bundle.get("dispatchDecision", {})

    task_ref = code_change["taskRef"]
    title = task_ref.split(":", 1)[-1]
    route = router.get("recommendedRoute", "patch_prep")
    decision_ref = code_change["dispatchDecisionRef"]
    allowed_paths = code_change["scope"]["allowedPaths"]
    required_checks = code_change["requiredChecks"]
    forbidden_actions = code_change["forbiddenActions"]

    lines = [
        f"# Developer Handoff: {title}",
        "",
        f"Task Ref: `{task_ref}`",
        f"Code Change Request: `{code_change['codeChangeRequestId']}`",
        f"Dispatch Decision: `{decision_ref}`",
        f"Authority Level: `{code_change['authorityLevel']}`",
        f"Risk Level: `{code_change['riskLevel']}`",
        f"Recommended Route: `{route}`",
        "",
        "## Summary",
        "",
        request.get("prompt", {}).get(
            "summary",
            "Prepare a bounded patch-prep handoff without source or runtime mutation.",
        ),
        "",
        "This handoff is advisory implementation preparation only. It does not authorize file edits, commits, pushes, deploys, provider execution, graph mutation, or runtime mutation.",
        "",
        "## Recommended Patch Plan",
        "",
        "1. Review the task reference and allowed scope.",
        "2. Inspect the likely files listed below before choosing an implementation path.",
        "3. Prepare an implementation plan with exact file targets and acceptance criteria.",
        "4. Run the required checks listed in this handoff before requesting any higher dispatch level.",
        "5. Return findings, blockers, and review notes through the dispatch surface.",
        "",
        "No source mutation is approved by this handoff.",
        "",
        "## Likely Files",
        "",
        bullet_list(allowed_paths),
        "",
        "## Verification Commands",
        "",
        checkbox_list(required_checks),
        "",
        "## Risk Notes",
        "",
        f"- Current risk classification is `{code_change['riskLevel']}` and authority is capped at `{code_change['authorityLevel']}`.",
        "- Any implementation request must be dispatched separately and must name an isolated worktree/write scope.",
        "- If the task touches auth, provider/runtime topology, workflow authority, schemas, canister interfaces, deploys, or graph mutation, escalate instead of continuing.",
        "",
        "## Acceptance Criteria",
        "",
        "- [ ] Handoff reviewed by Codex/operator before implementation.",
        "- [ ] Allowed path scope is still sufficient and accurate.",
        "- [ ] Required checks are still appropriate for the requested change.",
        "- [ ] Any implementation work is routed through a separate dispatch decision.",
        "- [ ] No forbidden action was taken during handoff generation.",
        "",
        "## Forbidden Actions Confirmed",
        "",
        bullet_list(forbidden_actions),
        "",
        "## Dispatch Context",
        "",
        f"- Request: `{request.get('requestId', 'unknown')}`",
        f"- Decision: `{decision.get('decision', 'unknown')}`",
        f"- Decider: `{decision.get('decider', {}).get('id', 'unknown')}`",
        f"- Conditions: {', '.join(f'`{item}`' for item in decision.get('conditions', [])) or '`none`'}",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("approved_bundle", help="Path to approved WorkRouter bundle JSON")
    parser.add_argument("--output", help="Optional output Markdown path. Omit to print to stdout.")
    args = parser.parse_args(argv[1:])

    bundle_path = Path(args.approved_bundle)
    if not bundle_path.is_absolute():
        bundle_path = ROOT / bundle_path
    bundle = load_json(bundle_path)
    require(isinstance(bundle, dict), f"{bundle_path}: bundle must be a JSON object")
    rendered = render_handoff(bundle)

    if args.output:
        output_path = Path(args.output)
        if not output_path.is_absolute():
            output_path = ROOT / output_path
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
        if not rendered.endswith("\n"):
            sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
