#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIRROR_ROOT="${REFERENCE_MIRROR_ROOT:-$ROOT_DIR/.cache/reference-mirrors}"
APPLY=false

usage() {
  cat <<'USAGE'
Usage: bash scripts/migrate_reference_mirrors_to_cache.sh [--apply]

Dry-runs by default. Moves embedded git mirrors out of research/reference and
into REFERENCE_MIRROR_ROOT only when --apply is provided.
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ "${1:-}" == "--apply" ]]; then
  APPLY=true
elif [[ $# -gt 0 ]]; then
  usage >&2
  exit 2
fi

cd "$ROOT_DIR"

if [[ "$APPLY" == true ]]; then
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "FAIL: refusing apply with tracked or staged changes present" >&2
    exit 1
  fi
fi

declare -a SCAN_ROOTS=(
  "research/reference/repos"
  "research/reference/topics"
  "research/reference/inbox"
)

declare -a CANDIDATES=()
for scan_root in "${SCAN_ROOTS[@]}"; do
  [[ -d "$scan_root" ]] || continue
  while IFS= read -r git_dir; do
    CANDIDATES+=("${git_dir%/.git}")
  done < <(find "$scan_root" -type d -name .git -prune 2>/dev/null | sort)
done

echo "Reference mirror migration"
echo "  root: $ROOT_DIR"
echo "  mirror cache: $MIRROR_ROOT"
echo "  mode: $([[ "$APPLY" == true ]] && echo apply || echo dry-run)"
echo "  candidates: ${#CANDIDATES[@]}"

if [[ ${#CANDIDATES[@]} -eq 0 ]]; then
  echo "PASS: no embedded git mirrors found"
  exit 0
fi

nested=0
for candidate in "${CANDIDATES[@]}"; do
  for possible_parent in "${CANDIDATES[@]}"; do
    [[ "$candidate" == "$possible_parent" ]] && continue
    if [[ "$candidate" == "$possible_parent/"* ]]; then
      echo "nested-collision: $candidate inside $possible_parent"
      nested=$((nested + 1))
      break
    fi
  done
done

if [[ "$nested" -gt 0 ]]; then
  echo "FAIL: nested mirror collisions require manual steward review before migration" >&2
  exit 1
fi

migrated=0
for repo_dir in "${CANDIDATES[@]}"; do
  cache_target="$MIRROR_ROOT/$repo_dir"
  echo "candidate: $repo_dir"
  echo "  -> $cache_target"

  if [[ "$APPLY" == true ]]; then
    if [[ -e "$cache_target" ]]; then
      echo "FAIL: cache target already exists: $cache_target" >&2
      exit 1
    fi
    mkdir -p "$(dirname "$cache_target")"
    mv "$repo_dir" "$cache_target"
    migrated=$((migrated + 1))
  fi
done

if [[ "$APPLY" == true ]]; then
  echo "migrated_repos=$migrated"
  echo "NOTE: no symlinks were created; keep cache paths out of Git."
else
  echo "dry-run complete (no changes made)"
  echo "Re-run with --apply only after reviewing all candidates."
fi
