use cortex_runtime::policy::metrics as runtime_metrics;
use std::fs;
use std::path::PathBuf;

pub use runtime_metrics::AcpPilotMetricsSnapshot;

fn load_metrics_store() -> Result<(runtime_metrics::AcpMetricsStore, PathBuf), String> {
    let path = default_path();
    let now = now_secs();
    let content = if path.exists() {
        Some(fs::read_to_string(&path).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let store = runtime_metrics::AcpMetricsStore::from_json(content.as_deref(), now)?;
    Ok((store, path))
}

fn save_metrics_store(
    store: &runtime_metrics::AcpMetricsStore,
    path: &PathBuf,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = store.to_json()?;
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn get_acp_metrics_snapshot() -> AcpPilotMetricsSnapshot {
    match load_metrics_store() {
        Ok((store, _)) => store.snapshot(now_secs()),
        Err(_) => AcpPilotMetricsSnapshot::default(),
    }
}

pub fn record_emit_attempt() {
    mutate_metrics(|store, now| store.record_emit_attempt(now));
}

pub fn record_emit_success() {
    mutate_metrics(|store, now| store.record_emit_success(now));
}

pub fn record_emit_failure() {
    mutate_metrics(|store, now| store.record_emit_failure(now));
}

#[allow(dead_code)]
pub fn record_fallback_queued() {
    mutate_metrics(|store, now| store.record_fallback_queued(now));
}

pub fn record_fallback_flush(success: bool, drain_latency_ms: u64) {
    mutate_metrics(|store, now| store.record_fallback_flush(success, drain_latency_ms, now));
}

fn mutate_metrics<F>(mutator: F)
where
    F: FnOnce(&mut runtime_metrics::AcpMetricsStore, u64),
{
    let now = now_secs();
    match load_metrics_store() {
        Ok((mut store, path)) => {
            mutator(&mut store, now);
            if let Err(err) = save_metrics_store(&store, &path) {
                tracing::warn!("failed to persist ACP metrics: {}", err);
            }
        }
        Err(err) => tracing::warn!("failed to load ACP metrics store: {}", err),
    }
}

fn default_path() -> PathBuf {
    let base = home::home_dir().unwrap_or_else(std::env::temp_dir);
    base.join(".cortex").join("acp_metrics.json")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_shape_is_available() {
        let snapshot = get_acp_metrics_snapshot();
        assert!(snapshot.updated_at <= now_secs());
    }
}
