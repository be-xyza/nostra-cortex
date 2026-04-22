#!/usr/bin/env bash
set -euo pipefail

GATEWAY_URL="${CORTEX_GATEWAY_URL:-http://127.0.0.1:3000}"
WORKSPACE_ID="${CORTEX_HEAP_DEFAULT_WORKSPACE_ID:-01ARZ3NDEKTSV4RRFFQ69G5FAV}"

if ! command -v curl >/dev/null 2>&1; then
  echo "error: curl is required" >&2
  exit 1
fi

if ! curl -fsS "${GATEWAY_URL}/api/system/ready" >/dev/null; then
  echo "error: gateway is not reachable at ${GATEWAY_URL}" >&2
  exit 1
fi

emit_block() {
  local request_id="$1"
  local artifact_id="$2"
  local block_type="$3"
  local title="$4"
  local mention_label="$5"
  local mirror_mentions="$6"
  local file_hash="$7"
  local file_size="$8"
  local file_name="$9"

  local emitted_at
  emitted_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  local files_payload="[]"
  if [[ -n "${file_hash}" ]]; then
    files_payload="$(cat <<JSON
[{
  "hash": "${file_hash}",
  "file_size": ${file_size},
  "name": "${file_name}",
  "mime_type": "image/png"
}]
JSON
)"
  fi

  local payload
  payload="$(cat <<JSON
{
  "schema_version": "1.0.0",
  "mode": "heap",
  "workspace_id": "${WORKSPACE_ID}",
  "source": {
    "agent_id": "cortex-demo-seed",
    "request_id": "${request_id}",
    "session_id": "seed-session-1",
    "emitted_at": "${emitted_at}"
  },
  "block": {
    "type": "${block_type}",
    "title": "${title}"
  },
  "content": {
    "a2ui": {
      "surface_id": "surface:${request_id}",
      "protocol_version": "1.0.0",
      "renderer": "react",
      "tree": {
        "surfaceId": "surface:${request_id}",
        "title": "${title}",
        "root": "root",
        "components": [{
          "id": "root",
          "type": "Card",
          "props": {
            "title": "${title}",
            "subtitle": "Seeded heap demo block"
          },
          "children": []
        }]
      }
    },
    "plain_text": "${title} seeded content"
  },
  "relations": {
    "tags": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAX" }],
    "mentions": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAY", "label": "${mention_label}" }],
    "page_links": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ" }]
  },
  "files": ${files_payload},
  "projection_hints": {
    "mirror_mentions_to_relations": ${mirror_mentions},
    "files_key_format": "hash",
    "relation_map_version": "relations_v1"
  },
  "crdt_projection": {
    "artifact_id": "${artifact_id}"
  }
}
JSON
)"

  local response
  response="$(curl -fsS -X POST "${GATEWAY_URL}/api/cortex/studio/heap/emit" \
    -H "Content-Type: application/json" \
    -H "x-cortex-role: operator" \
    -H "x-cortex-actor: heap-seed-script" \
    --data "${payload}")"

  echo "${response}"
}

echo "[seed_heap_mode_demo] workspace_id=${WORKSPACE_ID} gateway=${GATEWAY_URL}"

emit_block "seed-heap-req-1" "heap-demo-1" "widget" "Heap Demo Card 1" "Project Alpha" true "seedhash001" 128 "demo-1.png" >/dev/null
emit_block "seed-heap-req-2" "heap-demo-2" "note" "Heap Demo Card 2" "Project Beta" true "seedhash002" 256 "demo-2.png" >/dev/null
emit_block "seed-heap-req-3" "heap-demo-3" "chart" "Heap Demo Card 3" "Project Gamma" false "" 0 "" >/dev/null

count="$(
  curl -fsS "${GATEWAY_URL}/api/cortex/studio/heap/blocks?spaceId=${WORKSPACE_ID}&limit=100" \
    | sed -n 's/.*"count":\([0-9][0-9]*\).*/\1/p' \
    | head -n 1
)"
if [[ -z "${count}" ]]; then
  count="unknown"
fi
echo "[seed_heap_mode_demo] complete count=${count}"
