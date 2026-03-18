use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeEnvelope {
    pub schema_version: String,
    pub channel: String,
    pub artifact_id: String,
    pub session_id: String,
    pub actor_id: String,
    pub op_id: String,
    pub sequence: u64,
    pub lamport: u64,
    pub event_type: String,
    pub timestamp: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeBacklogItem {
    pub backlog_id: String,
    pub channel: String,
    pub artifact_id: String,
    pub op_id: String,
    pub enqueued_at: String,
    #[serde(default)]
    pub last_error: Option<String>,
    pub envelope: ArtifactRealtimeEnvelope,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeTransportStatus {
    pub schema_version: String,
    pub generated_at: String,
    pub realtime_enabled: bool,
    pub mode: String,
    pub primary_available: bool,
    pub degraded: bool,
    #[serde(default)]
    pub degraded_since: Option<String>,
    #[serde(default)]
    pub degraded_reason: Option<String>,
    pub pending_replay: usize,
    pub convergence_latency_p95_ms: u64,
    pub duplicate_op_drop_rate: f64,
    pub duplicate_ops_dropped: u64,
    pub published_total: u64,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeAckCursor {
    pub artifact_id: String,
    pub channel: String,
    pub last_sequence: u64,
    pub last_lamport: u64,
    pub last_op_id: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeIntegrityReport {
    pub schema_version: String,
    pub generated_at: String,
    pub artifact_id: String,
    pub channel: String,
    pub source_mode: String,
    pub primary_available: bool,
    pub degraded: bool,
    #[serde(default)]
    pub degraded_reason: Option<String>,
    pub pending_replay: usize,
    pub convergence_latency_p95_ms: u64,
    pub duplicate_op_drop_rate: f64,
    #[serde(default)]
    pub ack_cursor: Option<ArtifactRealtimeAckCursor>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeResyncResult {
    pub schema_version: String,
    pub generated_at: String,
    pub artifact_id: String,
    pub attempted_replay: usize,
    pub pending_replay_after: usize,
    #[serde(default)]
    pub ack_cursor: Option<ArtifactRealtimeAckCursor>,
    pub degraded: bool,
    #[serde(default)]
    pub degraded_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexRealtimeSloStatus {
    pub schema_version: String,
    pub generated_at: String,
    pub convergence_latency_p95_ms: u64,
    pub replay_backlog_depth: usize,
    pub replay_backlog_drain_time_p95_secs: u64,
    pub duplicate_op_drop_rate: f64,
    pub degraded_duration_secs_24h: u64,
    pub thresholds: BTreeMap<String, String>,
    pub breaches: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexRealtimeSloBreachEvent {
    pub event_id: String,
    pub metric: String,
    pub threshold: String,
    pub observed: String,
    pub opened_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeConnectAck {
    pub connected: bool,
    pub actor_id: String,
    pub artifact_id: String,
    pub channel: String,
    pub mode: String,
    pub connected_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeDisconnectAck {
    pub disconnected: bool,
    pub actor_id: String,
    pub artifact_id: String,
    pub channel: String,
    pub disconnected_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimePollResult {
    pub next_nonce: u64,
    pub events: Vec<ArtifactRealtimeEnvelope>,
}
