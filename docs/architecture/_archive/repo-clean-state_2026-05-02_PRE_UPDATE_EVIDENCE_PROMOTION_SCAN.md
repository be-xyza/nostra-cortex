# Repo Clean-State Contract

## Summary

Clean worktrees are an operator-authority invariant for developer work. They are not a personal preference and they are not a Cortex runtime primitive.

## Artifact Authority Model

1. Authored source, governed docs, and research artifacts are Git authority.
2. Mutable runtime, build, and test outputs are not Git authority.
3. Logs remain runtime/operator surfaces under `logs/`, but mutable `*_latest.*` and similar generated artifacts stay local.
4. When evidence must be preserved, promote an immutable copy into a governed initiative surface instead of tracking the mutable runtime output directly.

## Request Worktree Contract

1. One request = one branch = one worktree.
2. `.worktrees/` is the canonical request-worktree directory for this repo.
3. The shared root worktree is reserved for repo-wide stewardship tasks such as hygiene recovery, portfolio alignment, or structural migrations.
4. Do not use a dirty tree as memory.
5. Do not rely on `git stash` as the primary persistence mechanism for important work.
6. Every pause point must have a durable checkpoint through either:
   - a WIP commit, preferably pushed, or
   - an explicit patch bundle saved by the checkpoint script

## Required Operator Commands

1. `bash scripts/create_repo_recovery_snapshot.sh`
2. `bash scripts/start_request_worktree.sh --branch <branch>`
3. `bash scripts/checkpoint_request.sh`
4. `bash scripts/close_request.sh`
5. `bash scripts/worktree_gc.sh`
6. `bash scripts/promote_evidence_artifact.sh --source <path> --initiative <research-dir>`

## Enforcement

1. `bash scripts/check_clean_worktree.sh`
2. `bash scripts/check_tracked_generated_artifacts.sh`
3. Alignment rules begin in `observe` mode and promote only after green stability is demonstrated.
