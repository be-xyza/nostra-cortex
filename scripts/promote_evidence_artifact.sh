#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

SOURCE_PATH=""
INITIATIVE_DIR=""
DEST_SUBDIR="evidence"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source)
      SOURCE_PATH="${2:?missing value for --source}"
      shift 2
      ;;
    --initiative)
      INITIATIVE_DIR="${2:?missing value for --initiative}"
      shift 2
      ;;
    --subdir)
      DEST_SUBDIR="${2:?missing value for --subdir}"
      shift 2
      ;;
    *)
      echo "usage: promote_evidence_artifact.sh --source <path> --initiative <research-dir> [--subdir <name>]" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$SOURCE_PATH" || -z "$INITIATIVE_DIR" ]]; then
  echo "FAIL: --source and --initiative are required" >&2
  exit 1
fi

SOURCE_ABS="$(cd "$(dirname "$SOURCE_PATH")" && pwd)/$(basename "$SOURCE_PATH")"
if [[ ! -f "$SOURCE_ABS" ]]; then
  echo "FAIL: source artifact not found: $SOURCE_ABS" >&2
  exit 1
fi

if ! python3 "$ROOT_DIR/scripts/check_secret_egress.py" --paths "$SOURCE_ABS"; then
  echo "FAIL: source artifact failed secret egress scan; promotion blocked" >&2
  exit 1
fi

DEST_DIR="$ROOT_DIR/research/$INITIATIVE_DIR/$DEST_SUBDIR"
if [[ ! -d "$ROOT_DIR/research/$INITIATIVE_DIR" ]]; then
  echo "FAIL: initiative directory not found: research/$INITIATIVE_DIR" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
target="$DEST_DIR/${timestamp}_$(basename "$SOURCE_ABS")"
cp "$SOURCE_ABS" "$target"

checksum="$(shasum -a 256 "$target" | awk '{print $1}')"
cat > "$target.meta.json" <<EOF
{
  "promoted_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source_path": "$SOURCE_ABS",
  "target_path": "$target",
  "sha256": "$checksum"
}
EOF

echo "PASS: evidence artifact promoted"
echo "target=$target"
echo "metadata=$target.meta.json"
