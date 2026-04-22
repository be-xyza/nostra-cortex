#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
TARGET_DIR="$CORTEX_DESKTOP_DIR/src/components"
OUT_DIR="$ROOT_DIR/logs/testing"
OUT_FILE="$OUT_DIR/cortex_ui_theme_conformance_latest.json"
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

mkdir -p "$OUT_DIR"

collect_lines() {
  local cmd="$1"
  local output
  output="$(eval "$cmd" 2>/dev/null || true)"
  printf '%s' "$output"
}

to_json_array() {
  local lines="$1"
  if [[ -z "$lines" ]]; then
    echo '[]'
    return
  fi
  printf '%s\n' "$lines" | sed '/^$/d' | head -n 50 | jq -R . | jq -s .
}

count_lines() {
  local lines="$1"
  if [[ -z "$lines" ]]; then
    echo 0
    return
  fi
  printf '%s\n' "$lines" | sed '/^$/d' | wc -l | tr -d ' '
}

bg_hex_lines="$(collect_lines "rg -n 'bg-\\[#' '$TARGET_DIR'")"
dark_lines="$(collect_lines "rg -n 'dark:' '$TARGET_DIR'")"
white_border_lines="$(collect_lines "rg -n 'border-white/' '$TARGET_DIR'")"
text_white_lines="$(collect_lines "rg -n 'text-white' '$TARGET_DIR'")"
non_ascii_lines="$(collect_lines "rg -nP '[^\\x00-\\x7F]' '$TARGET_DIR'")"
named_color_lines="$(collect_lines "rg -n '(bg|text|border)-(emerald|rose|cyan|yellow|zinc|purple|amber|orange|red|green|blue|slate|gray)-[0-9]' '$TARGET_DIR'")"

bg_hex_count="$(count_lines "$bg_hex_lines")"
dark_count="$(count_lines "$dark_lines")"
white_border_count="$(count_lines "$white_border_lines")"
text_white_count="$(count_lines "$text_white_lines")"
non_ascii_count="$(count_lines "$non_ascii_lines")"
named_color_count="$(count_lines "$named_color_lines")"

violations='[]'

append_violation() {
  local rule="$1"
  local count="$2"
  local lines="$3"
  if [[ "$count" -gt 0 ]]; then
    local samples
    samples="$(to_json_array "$lines")"
    violations="$(jq -cn \
      --argjson base "$violations" \
      --arg rule "$rule" \
      --argjson count "$count" \
      --argjson samples "$samples" \
      '$base + [{rule:$rule, count:$count, samples:$samples}]')"
  fi
}

append_violation "bg_hex" "$bg_hex_count" "$bg_hex_lines"
append_violation "dark_variant" "$dark_count" "$dark_lines"
append_violation "border_white" "$white_border_count" "$white_border_lines"
append_violation "text_white" "$text_white_count" "$text_white_lines"
append_violation "non_ascii" "$non_ascii_count" "$non_ascii_lines"
append_violation "named_tailwind_color" "$named_color_count" "$named_color_lines"

overall_pass=true
if [[ "$bg_hex_count" -gt 0 || "$dark_count" -gt 0 || "$white_border_count" -gt 0 || "$text_white_count" -gt 0 || "$non_ascii_count" -gt 0 || "$named_color_count" -gt 0 ]]; then
  overall_pass=false
fi

jq -n \
  --arg schema_version "1.0.0" \
  --arg generated_at "$NOW_UTC" \
  --arg target_root "$TARGET_DIR" \
  --argjson overall_pass "$overall_pass" \
  --argjson violations "$violations" \
  --argjson bg_hex_count "$bg_hex_count" \
  --argjson dark_count "$dark_count" \
  --argjson white_border_count "$white_border_count" \
  --argjson text_white_count "$text_white_count" \
  --argjson non_ascii_count "$non_ascii_count" \
  --argjson named_color_count "$named_color_count" \
  '{
    schema_version: $schema_version,
    generated_at: $generated_at,
    target_root: $target_root,
    overall_pass: $overall_pass,
    counts: {
      bg_hex: $bg_hex_count,
      dark_variant: $dark_count,
      border_white: $white_border_count,
      text_white: $text_white_count,
      non_ascii: $non_ascii_count,
      named_tailwind_color: $named_color_count
    },
    violations: $violations
  }' > "$OUT_FILE"

echo "Wrote cortex ui theme conformance report: $OUT_FILE"

if [[ "$overall_pass" != true ]]; then
  echo "cortex ui theme conformance: FAIL"
  exit 1
fi

echo "cortex ui theme conformance: PASS"
