#!/usr/bin/env python3
from pathlib import Path
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
taxonomy_path = ROOT / "docs/reference/knowledge_taxonomy.toml"
topics_path = ROOT / "docs/reference/topics.md"

if not taxonomy_path.exists():
    print(f"FAIL: missing taxonomy file {taxonomy_path}", file=sys.stderr)
    sys.exit(1)
if not topics_path.exists():
    print(f"FAIL: missing topics file {topics_path}", file=sys.stderr)
    sys.exit(1)

data = tomllib.loads(taxonomy_path.read_text())
required_artifacts = {"paper", "book", "standard", "legal_doc"}
allowed = set(data.get("artifact_types", {}).get("allowed", []))
missing = sorted(required_artifacts - allowed)
if missing:
    print(f"FAIL: taxonomy missing required artifact types: {', '.join(missing)}", file=sys.stderr)
    sys.exit(1)

allowed_topics = data.get("topics", {}).get("allowed", [])
if not allowed_topics:
    print("FAIL: taxonomy has no allowed topics", file=sys.stderr)
    sys.exit(1)

topic_doc = topics_path.read_text()
for topic in allowed_topics:
    if f"`{topic}`" not in topic_doc:
        print(f"FAIL: topic {topic} missing from docs/reference/topics.md", file=sys.stderr)
        sys.exit(1)

print("PASS: reference taxonomy integrity")
