use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

const METRICS_SCHEMA_VERSION: u32 = 1;
const ROLLING_WINDOW_SECS: u64 = 5 * 60;
const MAX_ROLLING_EVENTS: usize = 4096;
const MAX_DRAIN_SAMPLES: usize = 2048;

static ACP_METRICS: OnceLock<Arc<Mutex<AcpMetricsStore>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcpPilotMetricsSnapshot {
    pub emit_attempts_total: u64,
    pub emit_success_total: u64,
    pub emit_failure_total: u64,
    pub fallback_queue_total: u64,
    pub fallback_flush_success_total: u64,
    pub fallback_flush_failure_total: u64,
    pub rolling_5m_success_rate: f64,
    pub rolling_5m_attempts: u64,
    pub rolling_5m_successes: u64,
    pub drain_latency_ms_p95: Option<u64>,
    pub drain_latency_ms_samples: usize,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EmitSample {
    ts: u64,
    success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AcpMetricsState {
    version: u32,
    emit_attempts_total: u64,
    emit_success_total: u64,
    emit_failure_total: u64,
    fallback_queue_total: u64,
    fallback_flush_success_total: u64,
    fallback_flush_failure_total: u64,
    #[serde(default)]
    emit_samples: Vec<EmitSample>,
    #[serde(default)]
    drain_latency_ms: Vec<u64>,
    updated_at: u64,
}

impl Default for AcpMetricsState {
    fn default() -> Self {
        Self {
            version: METRICS_SCHEMA_VERSION,
            emit_attempts_total: 0,
            emit_success_total: 0,
            emit_failure_total: 0,
            fallback_queue_total: 0,
            fallback_flush_success_total: 0,
            fallback_flush_failure_total: 0,
            emit_samples: Vec::new(),
            drain_latency_ms: Vec::new(),
            updated_at: now_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcpMetricsStore {
    path: PathBuf,
    state: AcpMetricsState,
}

impl AcpMetricsStore {
    pub fn load_default() -> Result<Self, String> {
        Self::load(Self::default_path())
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self {
                path,
                state: AcpMetricsState::default(),
            });
        }

        let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut state: AcpMetricsState = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        state.version = METRICS_SCHEMA_VERSION;
        let mut store = Self { path, state };
        store.prune();
        Ok(store)
    }

    pub fn snapshot(&self) -> AcpPilotMetricsSnapshot {
        let now = now_secs();
        let cutoff = now.saturating_sub(ROLLING_WINDOW_SECS);

        let mut rolling_attempts = 0u64;
        let mut rolling_successes = 0u64;
        for sample in &self.state.emit_samples {
            if sample.ts >= cutoff {
                rolling_attempts += 1;
                if sample.success {
                    rolling_successes += 1;
                }
            }
        }

        let rolling_rate = if rolling_attempts == 0 {
            0.0
        } else {
            rolling_successes as f64 / rolling_attempts as f64
        };

        AcpPilotMetricsSnapshot {
            emit_attempts_total: self.state.emit_attempts_total,
            emit_success_total: self.state.emit_success_total,
            emit_failure_total: self.state.emit_failure_total,
            fallback_queue_total: self.state.fallback_queue_total,
            fallback_flush_success_total: self.state.fallback_flush_success_total,
            fallback_flush_failure_total: self.state.fallback_flush_failure_total,
            rolling_5m_success_rate: rolling_rate,
            rolling_5m_attempts: rolling_attempts,
            rolling_5m_successes: rolling_successes,
            drain_latency_ms_p95: percentile_p95(&self.state.drain_latency_ms),
            drain_latency_ms_samples: self.state.drain_latency_ms.len(),
            updated_at: self.state.updated_at,
        }
    }

    pub fn record_emit_attempt(&mut self) -> Result<(), String> {
        self.state.emit_attempts_total = self.state.emit_attempts_total.saturating_add(1);
        self.state.updated_at = now_secs();
        self.save()
    }

    pub fn record_emit_success(&mut self) -> Result<(), String> {
        self.state.emit_success_total = self.state.emit_success_total.saturating_add(1);
        self.state.emit_samples.push(EmitSample {
            ts: now_secs(),
            success: true,
        });
        self.state.updated_at = now_secs();
        self.prune();
        self.save()
    }

    pub fn record_emit_failure(&mut self) -> Result<(), String> {
        self.state.emit_failure_total = self.state.emit_failure_total.saturating_add(1);
        self.state.emit_samples.push(EmitSample {
            ts: now_secs(),
            success: false,
        });
        self.state.updated_at = now_secs();
        self.prune();
        self.save()
    }

    pub fn record_fallback_queued(&mut self) -> Result<(), String> {
        self.state.fallback_queue_total = self.state.fallback_queue_total.saturating_add(1);
        self.state.updated_at = now_secs();
        self.save()
    }

    pub fn record_fallback_flush(
        &mut self,
        success: bool,
        drain_latency_ms: u64,
    ) -> Result<(), String> {
        if success {
            self.state.fallback_flush_success_total =
                self.state.fallback_flush_success_total.saturating_add(1);
        } else {
            self.state.fallback_flush_failure_total =
                self.state.fallback_flush_failure_total.saturating_add(1);
        }

        self.state.drain_latency_ms.push(drain_latency_ms);
        self.state.updated_at = now_secs();
        self.prune();
        self.save()
    }

    fn prune(&mut self) {
        let cutoff = now_secs().saturating_sub(ROLLING_WINDOW_SECS);
        self.state.emit_samples.retain(|s| s.ts >= cutoff);
        if self.state.emit_samples.len() > MAX_ROLLING_EVENTS {
            let overflow = self.state.emit_samples.len() - MAX_ROLLING_EVENTS;
            self.state.emit_samples.drain(0..overflow);
        }

        if self.state.drain_latency_ms.len() > MAX_DRAIN_SAMPLES {
            let overflow = self.state.drain_latency_ms.len() - MAX_DRAIN_SAMPLES;
            self.state.drain_latency_ms.drain(0..overflow);
        }
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let data = serde_json::to_string_pretty(&self.state).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, data).map_err(|e| e.to_string())
    }

    fn default_path() -> PathBuf {
        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".cortex").join("acp_metrics.json")
    }
}

pub fn acp_metrics() -> Arc<Mutex<AcpMetricsStore>> {
    ACP_METRICS
        .get_or_init(|| {
            Arc::new(Mutex::new(AcpMetricsStore::load_default().unwrap_or_else(
                |err| {
                    tracing::warn!("failed to load ACP metrics store: {}", err);
                    AcpMetricsStore {
                        path: std::env::temp_dir().join("acp_metrics_fallback.json"),
                        state: AcpMetricsState::default(),
                    }
                },
            )))
        })
        .clone()
}

pub fn get_acp_metrics_snapshot() -> AcpPilotMetricsSnapshot {
    let store = acp_metrics();
    let snapshot = match store.lock() {
        Ok(guard) => guard.snapshot(),
        Err(_) => AcpPilotMetricsSnapshot::default(),
    };
    snapshot
}

pub fn record_emit_attempt() {
    mutate_metrics(|store| store.record_emit_attempt());
}

pub fn record_emit_success() {
    mutate_metrics(|store| store.record_emit_success());
}

pub fn record_emit_failure() {
    mutate_metrics(|store| store.record_emit_failure());
}

pub fn record_fallback_queued() {
    mutate_metrics(|store| store.record_fallback_queued());
}

pub fn record_fallback_flush(success: bool, drain_latency_ms: u64) {
    mutate_metrics(|store| store.record_fallback_flush(success, drain_latency_ms));
}

fn mutate_metrics<F>(mutator: F)
where
    F: FnOnce(&mut AcpMetricsStore) -> Result<(), String>,
{
    let store = acp_metrics();
    match store.lock() {
        Ok(mut guard) => {
            if let Err(err) = mutator(&mut guard) {
                tracing::warn!("failed to update ACP metrics: {}", err);
            }
        }
        Err(_) => tracing::warn!("failed to acquire ACP metrics lock"),
    };
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn percentile_p95(samples: &[u64]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_unstable();

    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let pos = idx.saturating_sub(1).min(sorted.len() - 1);
    Some(sorted[pos])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_success_rate_and_percentile_are_calculated() {
        let path = std::env::temp_dir().join(format!("acp_metrics_{}.json", uuid::Uuid::new_v4()));
        let mut store = AcpMetricsStore::load(path).unwrap();

        store.record_emit_attempt().unwrap();
        store.record_emit_success().unwrap();
        store.record_emit_attempt().unwrap();
        store.record_emit_failure().unwrap();
        store.record_fallback_flush(true, 100).unwrap();
        store.record_fallback_flush(true, 250).unwrap();
        store.record_fallback_flush(true, 500).unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.emit_attempts_total, 2);
        assert_eq!(snapshot.emit_success_total, 1);
        assert_eq!(snapshot.emit_failure_total, 1);
        assert_eq!(snapshot.rolling_5m_attempts, 2);
        assert_eq!(snapshot.rolling_5m_successes, 1);
        assert!((snapshot.rolling_5m_success_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(snapshot.drain_latency_ms_p95, Some(500));
    }

    #[test]
    fn fallback_queue_counter_increments() {
        let path = std::env::temp_dir().join(format!("acp_metrics_{}.json", uuid::Uuid::new_v4()));
        let mut store = AcpMetricsStore::load(path).unwrap();

        store.record_fallback_queued().unwrap();
        store.record_fallback_queued().unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.fallback_queue_total, 2);
    }
}
