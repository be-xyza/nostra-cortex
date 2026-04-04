#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SOURCE_PATH = ROOT / "shared" / "ontology" / "core_ontology_v1.json"
OUTPUT_PATH = ROOT / "shared" / "ontology" / "core_ontology_v1.experimental.jsonld"

NOSTRA_NS = "https://nostra.dev/ontology/core#"
RDFS_NS = "http://www.w3.org/2000/01/rdf-schema#"


def build_context() -> dict:
    return {
        "@version": 1.1,
        "nostra": NOSTRA_NS,
        "rdfs": RDFS_NS,
        "label": "rdfs:label",
        "description": "rdfs:comment",
        "Ontology": "nostra:Ontology",
        "Class": "nostra:Class",
        "Relation": "nostra:Relation",
        "Property": "nostra:Property",
        "CompatibilityPolicy": "nostra:CompatibilityPolicy",
        "id": "@id",
        "type": "@type",
        "ontologyId": "nostra:ontologyId",
        "schemaVersion": "nostra:schemaVersion",
        "version": "nostra:version",
        "classes": {"@id": "nostra:classes", "@container": "@set"},
        "relations": {"@id": "nostra:relations", "@container": "@set"},
        "properties": {"@id": "nostra:properties", "@container": "@set"},
        "extensionRules": {"@id": "nostra:extensionRules", "@container": "@set"},
        "compatibility": "nostra:compatibility",
        "source": {"@id": "nostra:source", "@type": "@id"},
        "target": {"@id": "nostra:target", "@type": "@id"},
        "appliesTo": {"@id": "nostra:appliesTo", "@container": "@set"},
        "valueType": "nostra:valueType",
        "cardinality": "nostra:cardinality",
        "mode": "nostra:mode",
        "breakingChangePolicy": "nostra:breakingChangePolicy",
        "spaceExtensionPolicy": "nostra:spaceExtensionPolicy",
    }


def term_id(local_id: str) -> str:
    return f"nostra:{local_id}"


def value_type_id(value_type: str) -> str:
    return f"nostra:{value_type}"


def build_projection(source: dict) -> dict:
    projection = {
        "@context": build_context(),
        "id": "https://nostra.dev/ontology/core/v1",
        "type": "Ontology",
        "ontologyId": source["ontology_id"],
        "schemaVersion": source["schema_version"],
        "label": source["title"],
        "version": source["version"],
        "description": (
            "Experimental JSON-LD projection of the canonical core_ontology_v1.json "
            "manifest. This file is a prototype interoperability artifact, not the "
            "canonical ontology contract."
        ),
        "classes": [],
        "relations": [],
        "properties": [],
        "extensionRules": list(source["extension_rules"]),
        "compatibility": {
            "type": "CompatibilityPolicy",
            "mode": source["compatibility"]["mode"],
            "breakingChangePolicy": source["compatibility"]["breaking_change_policy"],
            "spaceExtensionPolicy": source["compatibility"]["space_extension_policy"],
        },
    }

    for item in source["classes"]:
        projection["classes"].append(
            {
                "id": term_id(item["id"]),
                "type": "Class",
                "label": item["id"],
                "description": item["description"],
            }
        )

    for item in source["relations"]:
        relation = {
            "id": term_id(item["id"]),
            "type": "Relation",
            "label": item["id"],
            "description": item["description"],
            "source": term_id(item["source"]),
            "target": term_id(item["target"]),
        }
        if item.get("inverse"):
            relation["inverse"] = term_id(item["inverse"])
        projection["relations"].append(relation)

    for item in source["properties"]:
        prop = {
            "id": term_id(item["id"]),
            "type": "Property",
            "label": item["id"],
            "description": item["description"],
            "appliesTo": [term_id(class_id) for class_id in item["applies_to"]],
            "valueType": value_type_id(item["value_type"]),
        }
        if item.get("cardinality"):
            prop["cardinality"] = item["cardinality"]
        projection["properties"].append(prop)

    return projection


def serialize_document(document: dict) -> str:
    return json.dumps(document, indent=2, ensure_ascii=True) + "\n"


def load_source(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate the experimental JSON-LD projection for the core ontology manifest."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail if the generated projection differs from the checked-in output.",
    )
    args = parser.parse_args()

    source = load_source(SOURCE_PATH)
    rendered = serialize_document(build_projection(source))

    if args.check:
        current = OUTPUT_PATH.read_text(encoding="utf-8") if OUTPUT_PATH.exists() else ""
        if current != rendered:
            sys.stderr.write(
                "core ontology JSON-LD projection drift detected; run "
                "scripts/generate_core_ontology_jsonld.py to refresh it.\n"
            )
            return 1
        return 0

    OUTPUT_PATH.write_text(rendered, encoding="utf-8")
    sys.stdout.write(f"wrote {OUTPUT_PATH}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
