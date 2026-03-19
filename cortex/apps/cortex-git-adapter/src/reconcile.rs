use crate::adapter::AppState;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RepoCheckpoint {
    last_reconciled_at: Option<String>,
}

pub fn spawn_reconciler(state: AppState) {
    tokio::spawn(async move {
        let mut last_prune = Instant::now() - std::time::Duration::from_secs(3600);
        loop {
            state.metrics.inc_reconcile_run();

            if last_prune.elapsed() >= std::time::Duration::from_secs(3600) {
                match crate::state::prune_deliveries(
                    &state.config.state_dir,
                    state.config.delivery_retention_days,
                ) {
                    Ok(removed) => {
                        if removed > 0 {
                            tracing::info!(removed, "pruned old delivery markers");
                        }
                    }
                    Err(err) => {
                        tracing::warn!("failed to prune deliveries: {}", err);
                    }
                }
                last_prune = Instant::now();
            }

            let mut next_sleep = std::time::Duration::from_secs(30);
            let now = Utc::now();

            for repo in state.registry.iter_enabled() {
                let checkpoint_path = match checkpoint_path(&state.config.state_dir, repo) {
                    Ok(p) => p,
                    Err(err) => {
                        tracing::warn!(repo=%repo.repo_full_name, "checkpoint path error: {}", err);
                        continue;
                    }
                };
                let checkpoint = load_checkpoint(&checkpoint_path).unwrap_or_default();
                let interval = std::time::Duration::from_secs(repo.interval_secs.max(60));
                let jitter = stable_jitter(repo.repo_full_name.as_str());
                let (due, sleep_hint) = reconcile_due(now, &checkpoint, interval, jitter);
                if due {
                    state.metrics.inc_reconcile_repo_run();
                    let started = Instant::now();
                    if let Err(err) = reconcile_repo(&state, repo).await {
                        state.metrics.inc_reconcile_repo_failure();
                        state.metrics.inc_reconcile_failure();
                        tracing::warn!(repo=%repo.repo_full_name, "reconcile failed: {}", err);
                    } else {
                        let duration_ms = started.elapsed().as_millis() as u64;
                        state.metrics.inc_reconcile_repo_success();
                        state.metrics.set_reconcile_repo_last_duration_ms(duration_ms);
                        tracing::info!(repo=%repo.repo_full_name, duration_ms, "reconcile ok");
                    }
                } else {
                    next_sleep = next_sleep.min(sleep_hint);
                }
            }

            let sleep_for = next_sleep.clamp(
                std::time::Duration::from_secs(5),
                std::time::Duration::from_secs(60),
            );
            tokio::time::sleep(sleep_for).await;
        }
    });
}

fn stable_jitter(repo_full_name: &str) -> std::time::Duration {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(repo_full_name.as_bytes());
    let digest = h.finalize();
    let n = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
    // 0-29 seconds deterministic offset
    std::time::Duration::from_secs((n % 30) as u64)
}

fn reconcile_due(
    now: DateTime<Utc>,
    checkpoint: &RepoCheckpoint,
    interval: std::time::Duration,
    jitter: std::time::Duration,
) -> (bool, std::time::Duration) {
    let interval = Duration::from_std(interval).unwrap_or(Duration::seconds(60));
    let jitter = Duration::from_std(jitter).unwrap_or(Duration::seconds(0));
    let last = checkpoint
        .last_reconciled_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    // First run: spread by jitter
    let due_at = match last {
        Some(last) => last + interval + jitter,
        None => now - Duration::seconds(1) + jitter,
    };

    if now >= due_at {
        return (true, std::time::Duration::from_secs(0));
    }

    let sleep = (due_at - now)
        .to_std()
        .unwrap_or_else(|_| std::time::Duration::from_secs(5));
    (false, sleep)
}

async fn reconcile_repo(state: &AppState, repo: &crate::config::RepoConfig) -> Result<(), String> {
    let checkpoint_path = checkpoint_path(&state.config.state_dir, repo)?;
    let now = Utc::now();
    let lookback = Duration::seconds(repo.lookback_secs.min(60 * 60 * 24 * 30) as i64);
    let since = now - lookback;
    let max_pages = state.config.reconcile_max_pages.max(1);
    let per_page = state.config.reconcile_per_page.max(1);
    let settings = state.config.projector_settings();

    // Commits
    if repo.ingest_push {
        let mut page = 1usize;
        loop {
            let entries = state
                .github_api
                .list_commits_since(
                    &repo.repo_full_name,
                    repo.branch(),
                    since,
                    per_page,
                    page,
                )
                .await
                .map_err(|e| e.to_string())?;
            if entries.is_empty() {
                break;
            }
            for commit in entries {
                let payload = serde_json::json!({
                    "repository": {
                        "full_name": repo.repo_full_name,
                        "html_url": format!("https://github.com/{}", repo.repo_full_name),
                        "default_branch": repo.branch(),
                        "visibility": ""
                    },
                    "commits": [{
                        "id": commit.get("sha").and_then(|v| v.as_str()).unwrap_or(""),
                        "message": commit.get("commit").and_then(|v| v.get("message")).and_then(|v| v.as_str()).unwrap_or(""),
                        "url": commit.get("html_url").and_then(|v| v.as_str()).unwrap_or(""),
                        "timestamp": commit.get("commit").and_then(|v| v.get("committer")).and_then(|v| v.get("date")).and_then(|v| v.as_str()).unwrap_or(""),
                        "author": {
                            "name": commit.get("commit").and_then(|v| v.get("author")).and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or(""),
                            "email": commit.get("commit").and_then(|v| v.get("author")).and_then(|v| v.get("email")).and_then(|v| v.as_str()).unwrap_or("")
                        }
                    }]
                });
                for cmd in crate::projector::project_event_to_kip(repo, "push", &payload, &settings)? {
                    state
                        .sink
                        .execute_kip(&cmd)
                        .await
                        .map_err(|e| format!("executeKip failed: {e}"))?;
                }
            }
            page += 1;
            if page > max_pages {
                break;
            }
        }
    }

    // PRs
    if repo.ingest_pull_request {
        let mut page = 1usize;
        loop {
            let pulls = state
                .github_api
                .list_pulls_updated(&repo.repo_full_name, "all", per_page, page)
                .await
                .map_err(|e| e.to_string())?;
            if pulls.is_empty() {
                break;
            }
            for pr in pulls {
                let updated_at = pr.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
                if let Ok(dt) = DateTime::parse_from_rfc3339(updated_at) {
                    if dt.with_timezone(&Utc) < since {
                        break;
                    }
                }
                let payload = serde_json::json!({
                    "repository": { "full_name": repo.repo_full_name, "html_url": format!("https://github.com/{}", repo.repo_full_name), "default_branch": "main", "visibility": "" },
                    "pull_request": pr
                });
                for cmd in crate::projector::project_event_to_kip(repo, "pull_request", &payload, &settings)? {
                    state.sink.execute_kip(&cmd).await.map_err(|e| e.to_string())?;
                }
            }
            page += 1;
            if page > max_pages {
                break;
            }
        }
    }

    // Issues
    if repo.ingest_issues {
        let mut page = 1usize;
        loop {
            let issues = state
                .github_api
                .list_issues_since(&repo.repo_full_name, since, per_page, page)
                .await
                .map_err(|e| e.to_string())?;
            if issues.is_empty() {
                break;
            }
            for issue in issues {
                if issue.get("pull_request").is_some() {
                    continue;
                }
                let payload = serde_json::json!({
                    "repository": { "full_name": repo.repo_full_name, "html_url": format!("https://github.com/{}", repo.repo_full_name), "default_branch": "main", "visibility": "" },
                    "issue": issue
                });
                for cmd in crate::projector::project_event_to_kip(repo, "issues", &payload, &settings)? {
                    state.sink.execute_kip(&cmd).await.map_err(|e| e.to_string())?;
                }
            }
            page += 1;
            if page > max_pages {
                break;
            }
        }
    }

    persist_checkpoint(&checkpoint_path, now)
}

fn checkpoint_path(root: &PathBuf, repo: &crate::config::RepoConfig) -> Result<PathBuf, String> {
    let safe = crate::config::sanitize_path_component(repo.repo_full_name.as_str());
    Ok(root.join("repos").join(format!("{safe}.json")))
}

fn load_checkpoint(path: &PathBuf) -> Option<RepoCheckpoint> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn persist_checkpoint(path: &PathBuf, now: DateTime<Utc>) -> Result<(), String> {
    let mut checkpoint = load_checkpoint(path).unwrap_or_default();
    checkpoint.last_reconciled_at = Some(now.to_rfc3339());
    let raw =
        serde_json::to_string_pretty(&checkpoint).map_err(|e| format!("encode checkpoint: {e}"))?;
    crate::state::atomic_write_text(path, raw.as_str())
}
