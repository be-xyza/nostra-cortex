#!/usr/bin/env python3
from pathlib import Path
import hashlib
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
index_path = ROOT / "research/reference/knowledge/index.toml"
taxonomy_path = ROOT / "docs/reference/knowledge_taxonomy.toml"

required_metadata_fields = [
    "schema_version:",
    "artifact_id:",
    "artifact_type:",
    "title:",
    "authors:",
    "year:",
    "publisher:",
    "upstream_url:",
    "topic:",
    "initiative_refs:",
    "primary_steward:",
    "authority_mode:",
    "lineage_record:",
    "review_date:",
    "confidence_score:",
    "source_reliability:",
    "validation_proof:",
    "standards_alignment:",
    "adoption_decision:",
    "known_risks:",
    "suggested_next_experiments:",
]
required_sections = [
    "# Placement",
    "## Intent",
    "## Possible Links To Nostra Platform and Cortex Runtime",
    "## Initiative Links",
    "## Pattern Extraction",
    "## Adoption Decision",
    "## Known Risks",
    "## Suggested Next Experiments",
]

if not index_path.exists():
    print(f"FAIL: missing knowledge index {index_path}", file=sys.stderr)
    sys.exit(1)

taxonomy = tomllib.loads(taxonomy_path.read_text())
allowed_topics = set(taxonomy.get("topics", {}).get("allowed", []))
index = tomllib.loads(index_path.read_text() or "")
artifacts = index.get("artifacts") or index.get("entry", [])

sha_re = re.compile(r"sha256:\s*\"([0-9a-f]{64})\"")
path_re = re.compile(r"- path:\s*\"?([^\n\"]+)\"?")

for entry in artifacts:
    meta_path = ROOT / entry.get("metadata_path", entry["path"])
    if meta_path.is_dir():
        meta_path = meta_path / "metadata.md"
    analysis_path = ROOT / entry["analysis_path"]
    if not meta_path.exists():
        print(f"FAIL: missing metadata file {meta_path}", file=sys.stderr)
        sys.exit(1)
    if not analysis_path.exists():
        print(f"FAIL: missing analysis file {analysis_path}", file=sys.stderr)
        sys.exit(1)

    meta_text = meta_path.read_text()
    for field in required_metadata_fields:
        if field not in meta_text:
            print(f"FAIL: missing metadata field {field} in {meta_path}", file=sys.stderr)
            sys.exit(1)

    topic_match = re.search(r"^topic:\s*\"?([A-Za-z0-9-]+)\"?$", meta_text, re.M)
    if not topic_match or topic_match.group(1) not in allowed_topics:
        print(f"FAIL: invalid or unregistered topic in {meta_path}", file=sys.stderr)
        sys.exit(1)

    sha_match = sha_re.search(meta_text)
    path_match = path_re.search(meta_text)
    if not sha_match or not path_match:
        print(f"FAIL: missing source file path or sha256 in {meta_path}", file=sys.stderr)
        sys.exit(1)
    source_ref = Path(path_match.group(1))
    source_path = source_ref if source_ref.is_absolute() else ROOT / source_ref
    if not source_path.exists():
        source_path = meta_path.parent / source_ref
    if not source_path.exists():
        print(f"FAIL: missing source artifact {source_path}", file=sys.stderr)
        sys.exit(1)
    actual_sha = hashlib.sha256(source_path.read_bytes()).hexdigest()
    if actual_sha != sha_match.group(1):
        print(f"FAIL: sha mismatch for {source_path}", file=sys.stderr)
        sys.exit(1)

    analysis_text = analysis_path.read_text()
    for section in required_sections:
        if section not in analysis_text:
            print(f"FAIL: missing analysis section {section} in {analysis_path}", file=sys.stderr)
            sys.exit(1)

print(f"PASS: reference metadata v2 ({len(artifacts)} artifact(s))")
