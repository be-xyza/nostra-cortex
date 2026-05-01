#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ENV_FILE="${NOSTRA_VPS_ENV_FILE:-/srv/nostra/eudaemon-alpha/config/eudaemon-alpha.env}"
OUTPUT_FORMAT="json"

usage() {
  cat <<'USAGE'
Usage: scripts/inspect_runtime_config_redacted.sh [--env-file PATH] [--format json|table]

Print redacted metadata for secret-bearing runtime config.
Raw values are never emitted. Output includes name, presence, source class,
value length, fingerprint, and policy state.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env-file)
      ENV_FILE="${2:-}"
      shift 2
      ;;
    --format)
      OUTPUT_FORMAT="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "FAIL: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$OUTPUT_FORMAT" != "json" && "$OUTPUT_FORMAT" != "table" ]]; then
  echo "FAIL: --format must be json or table" >&2
  exit 1
fi

if [[ -z "$ENV_FILE" || ! -f "$ENV_FILE" ]]; then
  echo "FAIL: env file not found" >&2
  exit 1
fi

OUTPUT_FORMAT="$OUTPUT_FORMAT" ENV_FILE="$ENV_FILE" python3 - <<'PY'
import hashlib
import json
import os
import re
from pathlib import Path

env_file = Path(os.environ["ENV_FILE"])
output_format = os.environ["OUTPUT_FORMAT"]

secret_name_re = re.compile(
    r"(API[_-]?KEY|TOKEN|SECRET|PRIVATE[_-]?KEY|PASSWORD|CREDENTIAL|AUTH[_-]?BINDING|BEARER)",
    re.IGNORECASE,
)
non_secret_name_re = re.compile(
    r"(COST[_-]?PER[_-]?1K[_-]?TOKENS|MAX[_-]?TOKENS|TOKEN[_-]?LIMIT|TOKEN[_-]?BUDGET|TOKENIZER)",
    re.IGNORECASE,
)

records = []
seen = set()

for raw_line in env_file.read_text(encoding="utf-8", errors="replace").splitlines():
    line = raw_line.strip()
    if not line or line.startswith("#") or "=" not in line:
        continue
    name, value = line.split("=", 1)
    name = name.strip()
    if not name or name in seen or not secret_name_re.search(name) or non_secret_name_re.search(name):
        continue
    seen.add(name)
    value = value.strip().strip('"').strip("'")
    fingerprint = "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()[:12]
    records.append(
        {
            "name": name,
            "present": bool(value),
            "source_class": "env_file",
            "value_length": len(value),
            "fingerprint": fingerprint if value else None,
            "policy": "raw_value_never_emit",
        }
    )

records.sort(key=lambda item: item["name"])

if output_format == "json":
    print(json.dumps({"schemaVersion": "1.0.0", "secrets": records}, indent=2))
else:
    print("name\tpresent\tsource_class\tvalue_length\tfingerprint\tpolicy")
    for item in records:
        print(
            "\t".join(
                [
                    item["name"],
                    str(item["present"]).lower(),
                    item["source_class"],
                    str(item["value_length"]),
                    item["fingerprint"] or "",
                    item["policy"],
                ]
            )
        )
PY
