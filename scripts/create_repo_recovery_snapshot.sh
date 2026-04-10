#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
RECOVERY_BRANCH="${RECOVERY_BRANCH:-codex/repo-clean-state-recovery-20260410}"
OUT_DIR_INPUT="${1:-}"

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="${OUT_DIR_INPUT:-/tmp/icp-repo-clean-recovery-$timestamp}"
mkdir -p "$OUT_DIR"

git -C "$ROOT_DIR" status --porcelain=v1 > "$OUT_DIR/status.porcelain"
git -C "$ROOT_DIR" diff --binary > "$OUT_DIR/tracked_changes.patch"
git -C "$ROOT_DIR" diff --binary -- AGENTS.md docs research scripts shared > "$OUT_DIR/authored_changes.patch"
printf '%s\n' "$RECOVERY_BRANCH" > "$OUT_DIR/recovery_branch.txt"

node -e '
const fs = require("fs");
const path = process.argv[1];
const raw = fs.readFileSync(`${path}/status.porcelain`, "utf8");
const entries = [];
const counts = {};
for (const line of raw.split(/\n/).filter(Boolean)) {
  const status = line.slice(0, 2);
  const file = line.slice(3);
  let category = "unknown_scratch_or_unowned";
  if (file.includes("/node_modules/") || file.includes("/dist/") || file.includes("/.vite/") || file.includes("/test-results/")) {
    category = "generated_build_artifacts";
  } else if (file.startsWith("logs/") || file.startsWith("tmp/") || file.includes("/tmp/")) {
    category = "mutable_runtime_or_test_outputs";
  } else if (file === "AGENTS.md" || /^(cortex|nostra|research|shared|docs|scripts|tests|sdk|libraries)\//.test(file)) {
    category = "authored_source_docs_research";
  }
  entries.push({ status, path: file, category });
  counts[category] = (counts[category] || 0) + 1;
}
const cp = require("child_process");
const report = {
  generated_at: new Date().toISOString(),
  root: process.env.ROOT_DIR,
  head: cp.execFileSync("git", ["rev-parse", "HEAD"], { cwd: process.env.ROOT_DIR, encoding: "utf8" }).trim(),
  branch: cp.execFileSync("git", ["rev-parse", "--abbrev-ref", "HEAD"], { cwd: process.env.ROOT_DIR, encoding: "utf8" }).trim(),
  recovery_branch: process.env.RECOVERY_BRANCH,
  counts,
  entries,
};
fs.writeFileSync(`${path}/dirty_inventory.json`, JSON.stringify(report, null, 2) + "\n");
const untracked = entries.filter((entry) => entry.status === "??").map((entry) => entry.path);
fs.writeFileSync(`${path}/untracked_manifest.txt`, untracked.join("\n") + (untracked.length ? "\n" : ""));
console.log(JSON.stringify({ out_dir: path, counts }, null, 2));
' "$OUT_DIR"

echo "PASS: recovery snapshot created"
echo "out_dir=$OUT_DIR"
echo "recovery_branch=$RECOVERY_BRANCH"
