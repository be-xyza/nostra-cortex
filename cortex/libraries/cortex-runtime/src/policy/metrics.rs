use serde::{Deserialize, Serialize};

const METRICS_SCHEMA_VERSION: u32 = 1;
const ROLLING_WINDOW_SECS: u64 = 5 * 60;
const MAX_ROLLING_EVENTS: usize = 4096;
const MAX_DRAIN_SAMPLES: usize = 2048;

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

impl AcpMetricsState {
    fn default_with_now(now: u64) -> Self {
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
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcpMetricsStore {
    state: AcpMetricsState,
}

impl AcpMetricsStore {
    pub fn from_json(content: Option<&str>, now: u64) -> Result<Self, String> {
        let Some(content) = content else {
            return Ok(Self {
                state: AcpMetricsState::default_with_now(now),
            });
        };
        if content.trim().is_empty() {
            return Ok(Self {
                state: AcpMetricsState::default_with_now(now),
            });
        }

        let mut state: AcpMetricsState =
            serde_json::from_str(content).map_err(|e| e.to_string())?;
        state.version = METRICS_SCHEMA_VERSION;
        let mut store = Self { state };
        store.prune(now);
        Ok(store)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.state).map_err(|e| e.to_string())
    }

    pub fn snapshot(&self, now: u64) -> AcpPilotMetricsSnapshot {
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

    pub fn record_emit_attempt(&mut self, now: u64) {
        self.state.emit_attempts_total = self.state.emit_attempts_total.saturating_add(1);
        self.state.updated_at = now;
    }

    pub fn record_emit_success(&mut self, now: u64) {
        self.state.emit_success_total = self.state.emit_success_total.saturating_add(1);
        self.state.emit_samples.push(EmitSample {
            ts: now,
            success: true,
        });
        self.state.updated_at = now;
        self.prune(now);
    }

    pub fn record_emit_failure(&mut self, now: u64) {
        self.state.emit_failure_total = self.state.emit_failure_total.saturating_add(1);
        self.state.emit_samples.push(EmitSample {
            ts: now,
            success: false,
        });
        self.state.updated_at = now;
        self.prune(now);
    }

    pub fn record_fallback_queued(&mut self, now: u64) {
        self.state.fallback_queue_total = self.state.fallback_queue_total.saturating_add(1);
        self.state.updated_at = now;
    }

    pub fn record_fallback_flush(&mut self, success: bool, drain_latency_ms: u64, now: u64) {
        if success {
            self.state.fallback_flush_success_total =
                self.state.fallback_flush_success_total.saturating_add(1);
        } else {
            self.state.fallback_flush_failure_total =
                self.state.fallback_flush_failure_total.saturating_add(1);
        }

        self.state.drain_latency_ms.push(drain_latency_ms);
        self.state.updated_at = now;
        self.prune(now);
    }

    fn prune(&mut self, now: u64) {
        let cutoff = now.saturating_sub(ROLLING_WINDOW_SECS);
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
        let mut store = AcpMetricsStore::from_json(None, 100).unwrap();
        store.record_emit_attempt(101);
        store.record_emit_success(102);
        store.record_emit_attempt(103);
        store.record_emit_failure(104);
        store.record_fallback_flush(true, 100, 105);
        store.record_fallback_flush(true, 250, 106);
        store.record_fallback_flush(true, 500, 107);

        let snapshot = store.snapshot(108);
        assert_eq!(snapshot.emit_attempts_total, 2);
        assert_eq!(snapshot.emit_success_total, 1);
        assert_eq!(snapshot.emit_failure_total, 1);
        assert_eq!(snapshot.rolling_5m_attempts, 2);
        assert_eq!(snapshot.rolling_5m_successes, 1);
        assert!((snapshot.rolling_5m_success_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(snapshot.drain_latency_ms_p95, Some(500));
    }
}
