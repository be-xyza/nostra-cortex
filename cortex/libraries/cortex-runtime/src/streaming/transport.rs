use crate::RuntimeError;
use crate::ports::{LogAdapter, StreamingTransportAdapter, TimeProvider};
use cortex_domain::streaming::types::{
    ArtifactRealtimeAckCursor, ArtifactRealtimeBacklogItem, ArtifactRealtimeConnectAck,
    ArtifactRealtimeDisconnectAck, ArtifactRealtimeEnvelope, ArtifactRealtimePollResult,
    ArtifactRealtimeTransportStatus,
};
use std::collections::{BTreeMap, BTreeSet};

const DEFAULT_MAX_DEDUPE_WINDOW: usize = 4096;
const DEFAULT_MAX_DELIVERED_EVENTS: usize = 2048;

#[derive(Clone, Debug, Default)]
pub struct StreamingRuntimeState {
    pub next_nonce: u64,
    pub canister_nonce: u64,
    pub delivered: Vec<(u64, ArtifactRealtimeEnvelope)>,
    pub seen_op_keys: BTreeSet<String>,
    pub seen_op_order: Vec<String>,
    pub connected_clients: BTreeMap<String, u64>,
    pub replay_queue: Vec<ArtifactRealtimeBacklogItem>,
    pub ack_cursors: BTreeMap<String, ArtifactRealtimeAckCursor>,
    pub latencies_ms: Vec<u64>,
    pub duplicate_ops_dropped: u64,
    pub published_total: u64,
    pub degraded_since: Option<String>,
    pub degraded_since_unix: Option<u64>,
    pub degraded_reason: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StreamingOrchestrator {
    state: StreamingRuntimeState,
    max_dedupe_window: usize,
    max_delivered_events: usize,
}

impl Default for StreamingOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingOrchestrator {
    pub fn new() -> Self {
        Self {
            state: StreamingRuntimeState::default(),
            max_dedupe_window: DEFAULT_MAX_DEDUPE_WINDOW,
            max_delivered_events: DEFAULT_MAX_DELIVERED_EVENTS,
        }
    }

    pub fn with_state(state: StreamingRuntimeState) -> Self {
        Self {
            state,
            ..Self::new()
        }
    }

    pub fn state(&self) -> &StreamingRuntimeState {
        &self.state
    }

    pub fn connect(
        &mut self,
        actor_id: &str,
        artifact_id: &str,
        time: &dyn TimeProvider,
    ) -> Result<(u64, ArtifactRealtimeConnectAck), RuntimeError> {
        let client_nonce = time.now_unix_secs().saturating_mul(1_000);
        self.state
            .connected_clients
            .insert(actor_id.to_string(), client_nonce);

        let channel = format!("cortex:artifact:{artifact_id}");
        let connected_at = time.to_rfc3339(time.now_unix_secs())?;
        Ok((
            client_nonce,
            ArtifactRealtimeConnectAck {
                connected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel,
                mode: "runtime_orchestrated".to_string(),
                connected_at,
            },
        ))
    }

    pub fn disconnect(
        &mut self,
        actor_id: &str,
        artifact_id: &str,
        time: &dyn TimeProvider,
    ) -> Result<(Option<u64>, ArtifactRealtimeDisconnectAck), RuntimeError> {
        let client_nonce = self.state.connected_clients.remove(actor_id);
        let channel = format!("cortex:artifact:{artifact_id}");
        let disconnected_at = time.to_rfc3339(time.now_unix_secs())?;
        Ok((
            client_nonce,
            ArtifactRealtimeDisconnectAck {
                disconnected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel,
                disconnected_at,
            },
        ))
    }

    pub async fn publish_with_adapter<T: StreamingTransportAdapter>(
        &mut self,
        adapter: Option<&T>,
        envelope: ArtifactRealtimeEnvelope,
        time: &dyn TimeProvider,
        log: &dyn LogAdapter,
    ) -> Result<(), RuntimeError> {
        let dedupe_key = dedupe_key(&envelope.channel, &envelope.op_id);
        if !self.record_seen_key(dedupe_key) {
            return Ok(());
        }

        let timestamp_ms = time.now_unix_secs().saturating_mul(1_000);
        let client_nonce = self
            .state
            .connected_clients
            .get(&envelope.actor_id)
            .copied()
            .unwrap_or(timestamp_ms);

        let mut adapter_error: Option<String> = None;
        if let Some(transport) = adapter {
            if let Err(err) = transport
                .publish(&envelope, client_nonce, timestamp_ms)
                .await
            {
                adapter_error = Some(err.to_string());
                log.warn("streaming publish failed, queued for replay");
            }
        }

        self.append_local_event(envelope, 0, time)?;
        if let Some(err) = adapter_error {
            self.enqueue_replay(err, time)?;
        }
        Ok(())
    }

    pub async fn poll_with_adapter<T: StreamingTransportAdapter>(
        &mut self,
        adapter: Option<&T>,
        since_nonce: u64,
        limit: usize,
        artifact_filter: Option<&BTreeSet<String>>,
        time: &dyn TimeProvider,
        log: &dyn LogAdapter,
    ) -> Result<ArtifactRealtimePollResult, RuntimeError> {
        if let Some(transport) = adapter {
            match transport.poll(self.state.canister_nonce).await {
                Ok(remote) => {
                    self.state.canister_nonce = remote.next_nonce;
                    for event in remote.events {
                        let dedupe = dedupe_key(&event.channel, &event.op_id);
                        if !self.record_seen_key(dedupe) {
                            continue;
                        }
                        self.append_local_event(event, 0, time)?;
                    }
                    self.maybe_clear_degraded();
                }
                Err(err) => {
                    self.set_degraded("stream_unavailable", Some(err.to_string()), time)?;
                    log.warn("streaming poll degraded to local loopback");
                }
            }
        }
        Ok(self.poll_local(since_nonce, limit, artifact_filter))
    }

    pub fn poll_local(
        &self,
        since_nonce: u64,
        limit: usize,
        artifact_filter: Option<&BTreeSet<String>>,
    ) -> ArtifactRealtimePollResult {
        let mut events = self
            .state
            .delivered
            .iter()
            .filter(|(nonce, _)| *nonce > since_nonce)
            .filter(|(_, item)| {
                if let Some(filter) = artifact_filter {
                    filter.contains(&item.artifact_id)
                } else {
                    true
                }
            })
            .map(|(_, item)| item.clone())
            .collect::<Vec<_>>();
        if events.len() > limit {
            let trim = events.len() - limit;
            events.drain(0..trim);
        }
        ArtifactRealtimePollResult {
            next_nonce: self.state.next_nonce,
            events,
        }
    }

    pub fn status(
        &self,
        realtime_enabled: bool,
        primary_available: bool,
        time: &dyn TimeProvider,
    ) -> Result<ArtifactRealtimeTransportStatus, RuntimeError> {
        let duplicate_rate = if self.state.published_total == 0 {
            0.0
        } else {
            self.state.duplicate_ops_dropped as f64 / self.state.published_total as f64
        };

        Ok(ArtifactRealtimeTransportStatus {
            schema_version: "1.0.0".to_string(),
            generated_at: time.to_rfc3339(time.now_unix_secs())?,
            realtime_enabled,
            mode: if primary_available {
                "canister_primary".to_string()
            } else {
                "local_loopback".to_string()
            },
            primary_available,
            degraded: !self.state.replay_queue.is_empty() || self.state.degraded_since.is_some(),
            degraded_since: self.state.degraded_since.clone(),
            degraded_reason: self.state.degraded_reason.clone(),
            pending_replay: self.state.replay_queue.len(),
            convergence_latency_p95_ms: percentile_95(&self.state.latencies_ms),
            duplicate_op_drop_rate: duplicate_rate,
            duplicate_ops_dropped: self.state.duplicate_ops_dropped,
            published_total: self.state.published_total,
            last_error: self.state.last_error.clone(),
        })
    }

    pub fn ack_cursor(&self, artifact_id: &str) -> Option<ArtifactRealtimeAckCursor> {
        let channel = format!("cortex:artifact:{artifact_id}");
        self.state.ack_cursors.get(&channel).cloned()
    }

    fn append_local_event(
        &mut self,
        envelope: ArtifactRealtimeEnvelope,
        elapsed_ms: u64,
        time: &dyn TimeProvider,
    ) -> Result<(), RuntimeError> {
        self.state.next_nonce = self.state.next_nonce.saturating_add(1);
        let nonce = self.state.next_nonce;
        self.state.delivered.push((nonce, envelope.clone()));
        if self.state.delivered.len() > self.max_delivered_events {
            let trim = self.state.delivered.len() - self.max_delivered_events;
            self.state.delivered.drain(0..trim);
        }
        self.state.published_total = self.state.published_total.saturating_add(1);

        self.state.latencies_ms.push(elapsed_ms);
        if self.state.latencies_ms.len() > 512 {
            let trim = self.state.latencies_ms.len() - 512;
            self.state.latencies_ms.drain(0..trim);
        }

        self.state.ack_cursors.insert(
            envelope.channel.clone(),
            ArtifactRealtimeAckCursor {
                artifact_id: envelope.artifact_id.clone(),
                channel: envelope.channel.clone(),
                last_sequence: envelope.sequence,
                last_lamport: envelope.lamport,
                last_op_id: envelope.op_id.clone(),
                updated_at: time.to_rfc3339(time.now_unix_secs())?,
            },
        );
        Ok(())
    }

    fn enqueue_replay(
        &mut self,
        error: String,
        time: &dyn TimeProvider,
    ) -> Result<(), RuntimeError> {
        if let Some((_, last)) = self.state.delivered.last() {
            let stamp = time.to_rfc3339(time.now_unix_secs())?;
            self.state.replay_queue.push(ArtifactRealtimeBacklogItem {
                backlog_id: format!("realtime_backlog_{}", time.now_unix_secs()),
                channel: last.channel.clone(),
                artifact_id: last.artifact_id.clone(),
                op_id: last.op_id.clone(),
                enqueued_at: stamp,
                last_error: Some(error.clone()),
                envelope: last.clone(),
            });
            self.set_degraded("stream_unavailable", Some(error), time)?;
        }
        Ok(())
    }

    fn set_degraded(
        &mut self,
        reason: &str,
        error: Option<String>,
        time: &dyn TimeProvider,
    ) -> Result<(), RuntimeError> {
        if self.state.degraded_since.is_none() {
            self.state.degraded_since = Some(time.to_rfc3339(time.now_unix_secs())?);
            self.state.degraded_since_unix = Some(time.now_unix_secs());
        }
        self.state.degraded_reason = Some(reason.to_string());
        if let Some(err) = error {
            self.state.last_error = Some(err);
        }
        Ok(())
    }

    fn maybe_clear_degraded(&mut self) {
        if self.state.replay_queue.is_empty() {
            self.state.degraded_since = None;
            self.state.degraded_since_unix = None;
            self.state.degraded_reason = None;
            self.state.last_error = None;
        }
    }

    fn record_seen_key(&mut self, key: String) -> bool {
        if self.state.seen_op_keys.contains(&key) {
            self.state.duplicate_ops_dropped = self.state.duplicate_ops_dropped.saturating_add(1);
            return false;
        }
        self.state.seen_op_keys.insert(key.clone());
        self.state.seen_op_order.push(key);
        if self.state.seen_op_order.len() > self.max_dedupe_window {
            let trim = self.state.seen_op_order.len() - self.max_dedupe_window;
            for stale in self.state.seen_op_order.drain(0..trim) {
                self.state.seen_op_keys.remove(&stale);
            }
        }
        true
    }
}

fn dedupe_key(channel: &str, op_id: &str) -> String {
    format!("{channel}:{op_id}")
}

fn percentile_95(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() - 1) * 95) / 100;
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedTime;

    impl TimeProvider for FixedTime {
        fn now_unix_secs(&self) -> u64 {
            1_700_000_000
        }
        fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
            Ok(format!("t-{unix_secs}"))
        }
    }

    struct NoopLog;

    impl LogAdapter for NoopLog {
        fn info(&self, _message: &str) {}
        fn warn(&self, _message: &str) {}
        fn error(&self, _message: &str) {}
    }

    #[test]
    fn local_publish_and_poll_are_ordered() {
        futures::executor::block_on(async {
            let mut orchestrator = StreamingOrchestrator::new();
            let envelope = ArtifactRealtimeEnvelope {
                schema_version: "1.0.0".to_string(),
                channel: "cortex:artifact:a".to_string(),
                artifact_id: "a".to_string(),
                session_id: "s".to_string(),
                actor_id: "actor".to_string(),
                op_id: "op-1".to_string(),
                sequence: 1,
                lamport: 1,
                event_type: "op_applied".to_string(),
                timestamp: "t-1".to_string(),
                payload: serde_json::json!({"k":1}),
            };
            orchestrator
                .publish_with_adapter::<NoopTransport>(None, envelope, &FixedTime, &NoopLog)
                .await
                .expect("publish");
            let result = orchestrator
                .poll_with_adapter::<NoopTransport>(None, 0, 10, None, &FixedTime, &NoopLog)
                .await
                .expect("poll");
            assert_eq!(result.events.len(), 1);
            assert_eq!(result.events[0].op_id, "op-1");
        });
    }

    struct NoopTransport;

    #[async_trait::async_trait]
    impl StreamingTransportAdapter for NoopTransport {
        async fn connect(
            &self,
            actor_id: &str,
            artifact_id: &str,
            _client_nonce: u64,
        ) -> Result<ArtifactRealtimeConnectAck, RuntimeError> {
            Ok(ArtifactRealtimeConnectAck {
                connected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel: format!("cortex:artifact:{artifact_id}"),
                mode: "noop".to_string(),
                connected_at: "t-0".to_string(),
            })
        }

        async fn disconnect(
            &self,
            actor_id: &str,
            artifact_id: &str,
            _client_nonce: Option<u64>,
        ) -> Result<ArtifactRealtimeDisconnectAck, RuntimeError> {
            Ok(ArtifactRealtimeDisconnectAck {
                disconnected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel: format!("cortex:artifact:{artifact_id}"),
                disconnected_at: "t-0".to_string(),
            })
        }

        async fn publish(
            &self,
            _envelope: &ArtifactRealtimeEnvelope,
            _client_nonce: u64,
            _timestamp_ms: u64,
        ) -> Result<(), RuntimeError> {
            Ok(())
        }

        async fn poll(&self, _nonce: u64) -> Result<ArtifactRealtimePollResult, RuntimeError> {
            Ok(ArtifactRealtimePollResult {
                next_nonce: 0,
                events: Vec::new(),
            })
        }
    }
}
