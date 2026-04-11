from __future__ import annotations

import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import quick_validate_registry_asset as validator


def workflow_text(*sections: str) -> str:
    body = "\n\n".join(sections)
    return textwrap.dedent(
        """\
        ---
        id: sample-workflow
        title: Sample Workflow
        owner: Systems Steward
        updated_at: 2026-04-09
        ---

        # Sample Workflow

        {body}
        """
    ).format(body=body)


class QuickValidateWorkflowTests(unittest.TestCase):
    def write_workflow(self, text: str) -> Path:
        tmpdir = tempfile.TemporaryDirectory()
        self.addCleanup(tmpdir.cleanup)
        path = Path(tmpdir.name) / "sample-workflow"
        path.mkdir()
        (path / "WORKFLOW.md").write_text(text, encoding="utf-8")
        return path

    def test_minimal_grounded_workflow_passes_without_optional_sections(self) -> None:
        path = self.write_workflow(
            workflow_text(
                "## Purpose\nMinimal workflow.",
                "## Triggers\n- Trigger",
                "## Inputs\n- Input",
                "## Steps\n1. Step",
                "## Outputs\n- Output",
                "## Observability\n- Evidence",
                "## Required Checks\n- `bash scripts/check_example.sh`",
            )
        )
        self.assertEqual(validator.validate_workflow(path), [])

    def test_observability_and_required_checks_remain_mandatory(self) -> None:
        path = self.write_workflow(
            workflow_text(
                "## Purpose\nMinimal workflow.",
                "## Triggers\n- Trigger",
                "## Inputs\n- Input",
                "## Steps\n1. Step",
                "## Outputs\n- Output",
            )
        )
        failures = validator.validate_workflow(path)
        self.assertIn("WORKFLOW.md missing section: Observability", failures)
        self.assertIn("WORKFLOW.md missing section: Required Checks", failures)

    def test_legacy_self_improvement_section_is_rejected(self) -> None:
        path = self.write_workflow(
            workflow_text(
                "## Purpose\nMinimal workflow.",
                "## Triggers\n- Trigger",
                "## Inputs\n- Input",
                "## Steps\n1. Step",
                "## Outputs\n- Output",
                "## Observability\n- Evidence",
                "## Self-Improvement\n- Speculative loop",
                "## Required Checks\n- `bash scripts/check_example.sh`",
            )
        )
        failures = validator.validate_workflow(path)
        self.assertIn("WORKFLOW.md uses legacy speculative section: Self-Improvement", failures)

    def test_grounded_improvement_loop_is_allowed(self) -> None:
        path = self.write_workflow(
            workflow_text(
                "## Purpose\nMinimal workflow.",
                "## Triggers\n- Trigger",
                "## Inputs\n- Input",
                "## Lanes\n- `operator`",
                "## Analysis Focus\n- Contract drift",
                "## Steps\n1. Step",
                "## Outputs\n- Output",
                "## Observability\n- Evidence",
                "## Improvement Loop\n- Promotion review based on repeated evidence",
                "## Required Checks\n- `bash scripts/check_example.sh`",
            )
        )
        self.assertEqual(validator.validate_workflow(path), [])


class CanonicalWorkflowDocsTests(unittest.TestCase):
    def test_canonical_workflows_validate(self) -> None:
        workflow_root = ROOT / "nostra" / "commons" / "workflows"
        for workflow_dir in workflow_root.iterdir():
            if not workflow_dir.is_dir() or workflow_dir.name == "_archive":
                continue
            with self.subTest(workflow=workflow_dir.name):
                self.assertEqual(validator.validate_workflow(workflow_dir), [])

    def test_canonical_workflows_do_not_use_legacy_self_improvement(self) -> None:
        workflow_root = ROOT / "nostra" / "commons" / "workflows"
        for workflow_md in workflow_root.glob("*/WORKFLOW.md"):
            text = workflow_md.read_text(encoding="utf-8")
            with self.subTest(workflow=str(workflow_md)):
                self.assertNotIn("## Self-Improvement", text)


if __name__ == "__main__":
    unittest.main()
