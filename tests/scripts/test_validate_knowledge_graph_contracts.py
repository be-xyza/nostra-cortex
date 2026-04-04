from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "validate_knowledge_graph_contracts.py"


def load_module():
    spec = importlib.util.spec_from_file_location(
        "validate_knowledge_graph_contracts", MODULE_PATH
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class KnowledgeGraphContractValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.validator = load_module()

    def test_positive_ontology_examples_validate_cleanly(self) -> None:
        ontology_schema = self.validator.load_json(
            ROOT / "shared/standards/knowledge_graphs/ontology_manifest.schema.json"
        )
        core_ontology = self.validator.load_json(ROOT / "shared/ontology/core_ontology_v1.json")
        for path_str in self.validator.POSITIVE_ONTOLOGY_EXAMPLES:
            ontology = self.validator.load_json(ROOT / path_str)
            self.assertEqual(
                self.validator.validate_schema(ontology, ontology_schema, path_str),
                [],
            )
            self.assertEqual(
                self.validator.validate_ontology_semantics(ontology, core_ontology),
                [],
            )

    def test_semantic_validator_rejects_invalid_ontology_examples(self) -> None:
        report = self.validator.build_validation_report(ROOT)
        failure_ids = {item["fixture"] for item in report["ontology"]["negative_examples"]}
        self.assertIn(
            "shared/ontology/examples/invalid_core_redefinition_ontology_v1.json",
            failure_ids,
        )
        self.assertIn(
            "shared/ontology/examples/invalid_relation_endpoint_ontology_v1.json",
            failure_ids,
        )
        self.assertIn(
            "shared/ontology/examples/invalid_provenance_extension_ontology_v1.json",
            failure_ids,
        )

    def test_bundle_normalization_is_idempotent(self) -> None:
        roundtrip_report = self.validator.validate_bundle_roundtrip(
            ROOT,
            ROOT
            / "shared/standards/knowledge_graphs/examples/roundtrip"
            / "research_space_export_bundle_roundtrip_source_v1.json",
            ROOT
            / "shared/standards/knowledge_graphs/examples/roundtrip"
            / "research_space_export_bundle_roundtrip_normalized_v1.json",
        )
        self.assertTrue(roundtrip_report["normalized_match"])
        self.assertTrue(roundtrip_report["idempotent"])

    def test_bundle_negative_fixtures_fail_for_expected_reasons(self) -> None:
        failures = self.validator.validate_negative_bundle_examples(ROOT)
        observed = {item["fixture"]: item["reason"] for item in failures}
        self.assertEqual(
            observed[
                "shared/standards/knowledge_graphs/examples/negative/missing_ref_bundle_v1.json"
            ],
            "missing_required_reference",
        )
        self.assertEqual(
            observed[
                "shared/standards/knowledge_graphs/examples/negative/incompatible_ontology_version_bundle_v1.json"
            ],
            "incompatible_ontology_version",
        )
        self.assertEqual(
            observed[
                "shared/standards/knowledge_graphs/examples/negative/incompatible_bundle_version_bundle_v1.json"
            ],
            "incompatible_bundle_version",
        )
        self.assertEqual(
            observed[
                "shared/standards/knowledge_graphs/examples/negative/non_portable_export_bundle_v1.json"
            ],
            "non_portable_export_ref",
        )

    def test_query_fixture_matrix_is_semantically_complete(self) -> None:
        matrix = self.validator.validate_query_fixture_matrix(ROOT)
        self.assertEqual(matrix["failures"], [])
        self.assertEqual(
            matrix["covered_cases"],
            {
                "actor",
                "system",
                "agent",
                "any",
                "zero_result",
                "provenance_disabled",
                "scope_isolation",
                "multi_hop_planning",
            },
        )
        self.assertEqual(matrix["missing_cases"], [])

    def test_reference_alignment_matrix_covers_required_comparators(self) -> None:
        matrix = self.validator.load_reference_alignment_matrix(ROOT)
        self.assertEqual(
            set(matrix["comparators"]),
            {
                "trustgraph",
                "json_ld_1_1",
                "shacl_core",
                "owlish",
                "horned_owl",
                "sparql_1_1",
            },
        )
        self.assertTrue(
            all(item["freeze_outcome"] == "pass" for item in matrix["entries"])
        )

    def test_topology_examples_are_registered_and_canonical(self) -> None:
        failures = self.validator.validate_topology_view(ROOT)
        self.assertEqual(failures, [])

    def test_benchmark_fixture_and_legacy_baseline_validate_cleanly(self) -> None:
        self.assertEqual(self.validator.validate_benchmark_fixture(ROOT), [])
        self.assertEqual(self.validator.validate_legacy_shared_evaluation(ROOT), [])

    def test_freeze_readiness_report_is_decision_complete(self) -> None:
        failures = self.validator.validate_freeze_report(ROOT)
        self.assertEqual(failures, [])


if __name__ == "__main__":
    unittest.main()
