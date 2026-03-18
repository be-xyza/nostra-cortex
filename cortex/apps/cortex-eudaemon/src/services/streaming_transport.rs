use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use chrono::Utc;
use ic_agent::{Agent, identity::AnonymousIdentity};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tokio::sync::{Mutex, OnceCell};

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeAckCursor {
    pub artifact_id: String,
    pub channel: String,
    pub last_sequence: u64,
    pub last_lamport: u64,
    pub last_op_id: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexRealtimeSloStatus {
    pub schema_version: String,
    pub generated_at: String,
    pub convergence_latency_p95_ms: u64,
    pub replay_backlog_depth: usize,
    pub replay_backlog_drain_time_p95_secs: u64,
    pub duplicate_op_drop_rate: f64,
    pub degraded_duration_secs_24h: u64,
    pub thresholds: std::collections::BTreeMap<String, String>,
    pub breaches: std::collections::BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexRealtimeSloBreachEvent {
    pub event_id: String,
    pub metric: String,
    pub threshold: String,
    pub observed: String,
    pub opened_at: String,
}

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeConnectAck {
    pub connected: bool,
    pub actor_id: String,
    pub artifact_id: String,
    pub channel: String,
    pub mode: String,
    pub connected_at: String,
}

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimeDisconnectAck {
    pub disconnected: bool,
    pub actor_id: String,
    pub artifact_id: String,
    pub channel: String,
    pub disconnected_at: String,
}

#[derive(Clone, Debug, Serialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRealtimePollResult {
    pub next_nonce: u64,
    pub events: Vec<ArtifactRealtimeEnvelope>,
}

#[derive(Clone, Debug)]
pub struct StreamingTransportManager {
    enabled: bool,
    canister: Option<CanisterStreamingTransport>,
    runtime: Arc<Mutex<StreamingRuntime>>,
}

#[derive(Clone, Debug, Default)]
struct StreamingRuntime {
    next_nonce: u64,
    canister_nonce: u64,
    delivered: Vec<(u64, ArtifactRealtimeEnvelope)>,
    seen_op_keys: HashSet<String>,
    seen_op_order: Vec<String>,
    connected_clients: HashMap<String, u64>,
    replay_queue: Vec<ArtifactRealtimeBacklogItem>,
    ack_cursors: HashMap<String, ArtifactRealtimeAckCursor>,
    latencies_ms: Vec<u64>,
    duplicate_ops_dropped: u64,
    published_total: u64,
    degraded_since: Option<String>,
    degraded_reason: Option<String>,
    last_error: Option<String>,
    slo_breach_opened_at: HashMap<String, String>,
    emitted_slo_alert_ids: HashSet<String>,
}

#[derive(Clone, Debug)]
struct CanisterStreamingTransport {
    host: String,
    canister_id: Principal,
    gateway_principal: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsClientKey {
    client_principal: Principal,
    client_nonce: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsOpenArgs {
    client_nonce: u64,
    gateway_principal: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsCloseArgs {
    client_key: WsClientKey,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsMessageArgs {
    client_key: WsClientKey,
    sequence_num: u64,
    timestamp: u64,
    is_service_message: bool,
    content: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsGetMessagesArgs {
    nonce: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsCertifiedMessage {
    client_key: WsClientKey,
    sequence_num: u64,
    timestamp: u64,
    is_service_message: bool,
    content: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsGetMessagesResult {
    messages: Vec<WsCertifiedMessage>,
    cert: Vec<u8>,
    tree: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ChatMessage {
    msg_type: String,
    content: String,
    conversation_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
enum WsResult {
    Ok(()),
    Err(String),
}

const REPLAY_DRAIN_RATE_OPS_PER_SEC: u64 = 50;
const MAX_DEDUPE_WINDOW: usize = 8192;
const SLO_CONVERGENCE_P95_MS: u64 = 3_000;
const SLO_REPLAY_DRAIN_P95_SECS: u64 = 60;
const SLO_DUPLICATE_DROP_RATE: f64 = 0.005;
const SLO_DAILY_DEGRADED_DURATION_SECS: u64 = 15 * 60;

fn parse_bool_env_default(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            if matches!(
                normalized.as_str(),
                "0" | "false" | "no" | "off" | "disabled"
            ) {
                return false;
            }
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on" | "enabled")
        }
        Err(_) => default,
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn get_ux_log_dir() -> PathBuf {
    env::var("NOSTRA_CORTEX_UX_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            env::var("CORTEX_IC_PROJECT_ROOT")
                .map(|root| PathBuf::from(root).join("logs").join("cortex").join("ux"))
        })
        .unwrap_or_else(|_| PathBuf::from("logs/cortex/ux"))
}

fn realtime_replay_path() -> PathBuf {
    get_ux_log_dir()
        .join("_runtime")
        .join("streaming_replay_queue.json")
}

fn realtime_ack_cursor_path() -> PathBuf {
    get_ux_log_dir()
        .join("_runtime")
        .join("streaming_ack_cursors.json")
}

fn realtime_slo_alerts_path() -> PathBuf {
    get_ux_log_dir()
        .join("_runtime")
        .join("streaming_slo_alerts.jsonl")
}

fn append_slo_alert_line(event: &CortexRealtimeSloBreachEvent) -> Result<(), String> {
    let path = realtime_slo_alerts_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let line = serde_json::to_string(event).map_err(|err| err.to_string())?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    writeln!(file, "{}", line).map_err(|err| err.to_string())
}

fn realtime_slo_registry_endpoint() -> Option<String> {
    env::var("CORTEX_REALTIME_LOG_REGISTRY_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn emit_slo_alert(event: &CortexRealtimeSloBreachEvent) -> Result<(), String> {
    if let Some(endpoint) = realtime_slo_registry_endpoint() {
        let client = reqwest::Client::new();
        let resp = client
            .post(endpoint)
            .header("X-Idempotency-Key", event.event_id.as_str())
            .json(event)
            .send()
            .await
            .map_err(|err| err.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("log-registry returned {}", resp.status()));
        }
    }
    append_slo_alert_line(event)
}

fn read_replay_queue_file() -> Vec<ArtifactRealtimeBacklogItem> {
    let path = realtime_replay_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<ArtifactRealtimeBacklogItem>>(&raw).unwrap_or_default()
}

fn write_replay_queue_file(queue: &[ArtifactRealtimeBacklogItem]) -> Result<(), String> {
    let path = realtime_replay_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let encoded = serde_json::to_string_pretty(queue).map_err(|err| err.to_string())?;
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn read_ack_cursors_file() -> HashMap<String, ArtifactRealtimeAckCursor> {
    let path = realtime_ack_cursor_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    serde_json::from_str::<HashMap<String, ArtifactRealtimeAckCursor>>(&raw).unwrap_or_default()
}

fn write_ack_cursors_file(
    cursors: &HashMap<String, ArtifactRealtimeAckCursor>,
) -> Result<(), String> {
    let path = realtime_ack_cursor_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let encoded = serde_json::to_string_pretty(cursors).map_err(|err| err.to_string())?;
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn percentile_95(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let idx = idx.saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}

fn dedupe_key(channel: &str, op_id: &str) -> String {
    format!("{channel}|{op_id}")
}

impl CanisterStreamingTransport {
    async fn from_env() -> Result<Self, String> {
        let host = env::var("NOSTRA_IC_HOST")
            .or_else(|_| env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
        let canister_id = resolve_canister_id(
            &["CANISTER_ID_NOSTRA_STREAMING", "CANISTER_ID_STREAMING"],
            "nostra_streaming",
        )
        .await?;
        let gateway_principal = env::var("NOSTRA_STREAMING_GATEWAY_PRINCIPAL")
            .ok()
            .and_then(|value| Principal::from_text(value.trim()).ok())
            .unwrap_or_else(Principal::anonymous);
        Ok(Self {
            host,
            canister_id,
            gateway_principal,
        })
    }

    async fn agent(&self) -> Result<Agent, String> {
        let agent = Agent::builder()
            .with_url(self.host.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|err| format!("failed to build streaming ic-agent: {err}"))?;

        if self.host.contains("127.0.0.1") || self.host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|err| format!("failed to fetch root key: {err}"))?;
        }
        Ok(agent)
    }

    async fn ws_open(&self, client_nonce: u64) -> Result<(), String> {
        let agent = self.agent().await?;
        let args = WsOpenArgs {
            client_nonce,
            gateway_principal: self.gateway_principal,
        };
        let payload = Encode!(&args).map_err(|err| err.to_string())?;
        let bytes = agent
            .update(&self.canister_id, "ws_open")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| format!("ws_open failed: {err}"))?;
        match Decode!(&bytes, WsResult).map_err(|err| err.to_string())? {
            WsResult::Ok(()) => Ok(()),
            WsResult::Err(err) => Err(err),
        }
    }

    async fn ws_close(&self, client_nonce: u64) -> Result<(), String> {
        let agent = self.agent().await?;
        let args = WsCloseArgs {
            client_key: WsClientKey {
                client_principal: Principal::anonymous(),
                client_nonce,
            },
        };
        let payload = Encode!(&args).map_err(|err| err.to_string())?;
        let bytes = agent
            .update(&self.canister_id, "ws_close")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| format!("ws_close failed: {err}"))?;
        match Decode!(&bytes, WsResult).map_err(|err| err.to_string())? {
            WsResult::Ok(()) => Ok(()),
            WsResult::Err(err) => Err(err),
        }
    }

    async fn ws_publish(
        &self,
        envelope: &ArtifactRealtimeEnvelope,
        client_nonce: u64,
    ) -> Result<(), String> {
        let agent = self.agent().await?;
        let content = serde_json::to_vec(envelope).map_err(|err| err.to_string())?;
        let args = WsMessageArgs {
            client_key: WsClientKey {
                client_principal: Principal::anonymous(),
                client_nonce,
            },
            sequence_num: envelope.sequence,
            timestamp: Utc::now().timestamp_millis().max(0) as u64,
            is_service_message: false,
            content,
        };
        let chat = ChatMessage {
            msg_type: envelope.event_type.clone(),
            content: envelope.channel.clone(),
            conversation_id: Some(envelope.artifact_id.clone()),
        };
        let payload = Encode!(&args, &Some(chat)).map_err(|err| err.to_string())?;
        let bytes = agent
            .update(&self.canister_id, "ws_message")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| format!("ws_message failed: {err}"))?;
        match Decode!(&bytes, WsResult).map_err(|err| err.to_string())? {
            WsResult::Ok(()) => Ok(()),
            WsResult::Err(err) => Err(err),
        }
    }

    async fn ws_poll(&self, nonce: u64) -> Result<Vec<(u64, ArtifactRealtimeEnvelope)>, String> {
        let agent = self.agent().await?;
        let args = WsGetMessagesArgs { nonce };
        let payload = Encode!(&args).map_err(|err| err.to_string())?;
        let bytes = agent
            .query(&self.canister_id, "ws_get_messages")
            .with_arg(payload)
            .call()
            .await
            .map_err(|err| format!("ws_get_messages failed: {err}"))?;
        let response = Decode!(&bytes, WsGetMessagesResult).map_err(|err| err.to_string())?;
        let mut items = Vec::new();
        for message in response.messages {
            if let Ok(envelope) =
                serde_json::from_slice::<ArtifactRealtimeEnvelope>(&message.content)
            {
                items.push((message.sequence_num, envelope));
            }
        }
        Ok(items)
    }
}

impl StreamingTransportManager {
    fn record_seen_key(runtime: &mut StreamingRuntime, key: String) -> bool {
        if runtime.seen_op_keys.contains(&key) {
            runtime.duplicate_ops_dropped = runtime.duplicate_ops_dropped.saturating_add(1);
            return false;
        }
        runtime.seen_op_keys.insert(key.clone());
        runtime.seen_op_order.push(key);
        if runtime.seen_op_order.len() > MAX_DEDUPE_WINDOW {
            let trim = runtime.seen_op_order.len() - MAX_DEDUPE_WINDOW;
            for stale in runtime.seen_op_order.drain(0..trim) {
                runtime.seen_op_keys.remove(&stale);
            }
        }
        true
    }

    fn update_ack_cursor(
        runtime: &mut StreamingRuntime,
        envelope: &ArtifactRealtimeEnvelope,
        sequence: u64,
    ) {
        runtime.ack_cursors.insert(
            envelope.channel.clone(),
            ArtifactRealtimeAckCursor {
                artifact_id: envelope.artifact_id.clone(),
                channel: envelope.channel.clone(),
                last_sequence: sequence,
                last_lamport: envelope.lamport,
                last_op_id: envelope.op_id.clone(),
                updated_at: now_iso(),
            },
        );
        let _ = write_ack_cursors_file(&runtime.ack_cursors);
    }

    fn set_degraded(runtime: &mut StreamingRuntime, reason: &str, error: Option<String>) {
        if runtime.degraded_since.is_none() {
            runtime.degraded_since = Some(now_iso());
        }
        runtime.degraded_reason = Some(reason.to_string());
        if error.is_some() {
            runtime.last_error = error;
        }
    }

    fn maybe_clear_degraded(runtime: &mut StreamingRuntime) {
        if runtime.replay_queue.is_empty() {
            runtime.degraded_since = None;
            runtime.degraded_reason = None;
            runtime.last_error = None;
        }
    }

    fn evaluate_slo_locked(runtime: &mut StreamingRuntime) -> CortexRealtimeSloStatus {
        let convergence_p95 = percentile_95(&runtime.latencies_ms);
        let replay_depth = runtime.replay_queue.len();
        let replay_drain_p95 = if replay_depth == 0 {
            0
        } else {
            ((replay_depth as u64) + REPLAY_DRAIN_RATE_OPS_PER_SEC - 1)
                / REPLAY_DRAIN_RATE_OPS_PER_SEC
        };
        let duplicate_rate = if runtime.published_total == 0 {
            0.0
        } else {
            runtime.duplicate_ops_dropped as f64 / runtime.published_total as f64
        };
        let degraded_duration_secs_24h = runtime
            .degraded_since
            .as_deref()
            .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| {
                (Utc::now().timestamp() - dt.with_timezone(&Utc).timestamp())
                    .max(0)
                    .min(24 * 60 * 60) as u64
            })
            .unwrap_or(0);

        let breaches = [
            (
                "convergence_latency_p95_ms",
                convergence_p95 > SLO_CONVERGENCE_P95_MS,
            ),
            (
                "replay_backlog_drain_time_p95_secs",
                replay_drain_p95 > SLO_REPLAY_DRAIN_P95_SECS,
            ),
            (
                "duplicate_op_drop_rate",
                duplicate_rate > SLO_DUPLICATE_DROP_RATE,
            ),
            (
                "daily_degraded_duration_secs",
                degraded_duration_secs_24h > SLO_DAILY_DEGRADED_DURATION_SECS,
            ),
        ];

        for (metric, breached) in breaches {
            if breached {
                runtime
                    .slo_breach_opened_at
                    .entry(metric.to_string())
                    .or_insert_with(now_iso);
            } else {
                runtime.slo_breach_opened_at.remove(metric);
            }
        }

        let mut thresholds = std::collections::BTreeMap::new();
        thresholds.insert(
            "convergence_latency_p95_ms".to_string(),
            SLO_CONVERGENCE_P95_MS.to_string(),
        );
        thresholds.insert(
            "replay_backlog_drain_time_p95_secs".to_string(),
            SLO_REPLAY_DRAIN_P95_SECS.to_string(),
        );
        thresholds.insert(
            "duplicate_op_drop_rate".to_string(),
            format!("{SLO_DUPLICATE_DROP_RATE:.4}"),
        );
        thresholds.insert(
            "daily_degraded_duration_secs".to_string(),
            SLO_DAILY_DEGRADED_DURATION_SECS.to_string(),
        );

        let mut breach_map = std::collections::BTreeMap::new();
        breach_map.insert(
            "convergence_latency_p95_ms".to_string(),
            convergence_p95 > SLO_CONVERGENCE_P95_MS,
        );
        breach_map.insert(
            "replay_backlog_drain_time_p95_secs".to_string(),
            replay_drain_p95 > SLO_REPLAY_DRAIN_P95_SECS,
        );
        breach_map.insert(
            "duplicate_op_drop_rate".to_string(),
            duplicate_rate > SLO_DUPLICATE_DROP_RATE,
        );
        breach_map.insert(
            "daily_degraded_duration_secs".to_string(),
            degraded_duration_secs_24h > SLO_DAILY_DEGRADED_DURATION_SECS,
        );

        CortexRealtimeSloStatus {
            schema_version: "1.0.0".to_string(),
            generated_at: now_iso(),
            convergence_latency_p95_ms: convergence_p95,
            replay_backlog_depth: replay_depth,
            replay_backlog_drain_time_p95_secs: replay_drain_p95,
            duplicate_op_drop_rate: duplicate_rate,
            degraded_duration_secs_24h,
            thresholds,
            breaches: breach_map,
        }
    }

    pub async fn from_env() -> Self {
        let enabled = parse_bool_env_default("CORTEX_COLLAB_REALTIME", true);
        let canister = if enabled {
            CanisterStreamingTransport::from_env().await.ok()
        } else {
            None
        };
        let runtime = StreamingRuntime {
            replay_queue: read_replay_queue_file(),
            ack_cursors: read_ack_cursors_file(),
            ..StreamingRuntime::default()
        };
        Self {
            enabled,
            canister,
            runtime: Arc::new(Mutex::new(runtime)),
        }
    }

    pub async fn connect(
        &self,
        actor_id: &str,
        artifact_id: &str,
    ) -> Result<ArtifactRealtimeConnectAck, String> {
        let channel = format!("cortex:artifact:{artifact_id}");
        let mut guard = self.runtime.lock().await;
        let client_nonce = Utc::now().timestamp_millis().max(0) as u64;
        guard
            .connected_clients
            .insert(actor_id.to_string(), client_nonce);
        drop(guard);

        if let Some(canister) = self.canister.as_ref() {
            if let Err(err) = canister.ws_open(client_nonce).await {
                let mut runtime = self.runtime.lock().await;
                Self::set_degraded(&mut runtime, "stream_unavailable", Some(err.clone()));
            }
        }

        Ok(ArtifactRealtimeConnectAck {
            connected: true,
            actor_id: actor_id.to_string(),
            artifact_id: artifact_id.to_string(),
            channel,
            mode: self.mode_label().to_string(),
            connected_at: now_iso(),
        })
    }

    pub async fn disconnect(
        &self,
        actor_id: &str,
        artifact_id: &str,
    ) -> Result<ArtifactRealtimeDisconnectAck, String> {
        let channel = format!("cortex:artifact:{artifact_id}");
        let mut guard = self.runtime.lock().await;
        let client_nonce = guard.connected_clients.remove(actor_id);
        drop(guard);

        if let (Some(canister), Some(nonce)) = (self.canister.as_ref(), client_nonce) {
            if let Err(err) = canister.ws_close(nonce).await {
                let mut runtime = self.runtime.lock().await;
                runtime.last_error = Some(err.clone());
                if runtime.degraded_reason.is_none() {
                    runtime.degraded_reason = Some("stream_unavailable".to_string());
                }
            }
        }

        Ok(ArtifactRealtimeDisconnectAck {
            disconnected: true,
            actor_id: actor_id.to_string(),
            artifact_id: artifact_id.to_string(),
            channel,
            disconnected_at: now_iso(),
        })
    }

    pub async fn publish(&self, envelope: ArtifactRealtimeEnvelope) -> Result<(), String> {
        let started = Instant::now();
        let mut queued_error: Option<String> = None;

        {
            let mut guard = self.runtime.lock().await;
            let key = dedupe_key(&envelope.channel, &envelope.op_id);
            if !Self::record_seen_key(&mut guard, key) {
                return Ok(());
            }
        }

        if let Some(canister) = self.canister.as_ref() {
            let client_nonce = {
                let guard = self.runtime.lock().await;
                guard
                    .connected_clients
                    .get(&envelope.actor_id)
                    .copied()
                    .unwrap_or_else(|| Utc::now().timestamp_millis().max(0) as u64)
            };
            if let Err(err) = canister.ws_publish(&envelope, client_nonce).await {
                queued_error = Some(err);
            }
        }

        let mut guard = self.runtime.lock().await;
        guard.next_nonce = guard.next_nonce.saturating_add(1);
        let current_nonce = guard.next_nonce;
        guard.delivered.push((current_nonce, envelope.clone()));
        if guard.delivered.len() > 2048 {
            let trim = guard.delivered.len() - 2048;
            guard.delivered.drain(0..trim);
        }
        guard.published_total = guard.published_total.saturating_add(1);

        let elapsed_ms = started.elapsed().as_millis() as u64;
        guard.latencies_ms.push(elapsed_ms);
        if guard.latencies_ms.len() > 512 {
            let trim = guard.latencies_ms.len() - 512;
            guard.latencies_ms.drain(0..trim);
        }

        if let Some(err) = queued_error {
            let backlog = ArtifactRealtimeBacklogItem {
                backlog_id: format!("realtime_backlog_{}", Utc::now().timestamp_millis()),
                channel: envelope.channel.clone(),
                artifact_id: envelope.artifact_id.clone(),
                op_id: envelope.op_id.clone(),
                enqueued_at: now_iso(),
                last_error: Some(err.clone()),
                envelope: envelope.clone(),
            };
            guard.replay_queue.push(backlog);
            Self::set_degraded(&mut guard, "stream_unavailable", Some(err));
            let _ = write_replay_queue_file(&guard.replay_queue);
        }
        Self::update_ack_cursor(&mut guard, &envelope, envelope.sequence);

        Ok(())
    }

    pub async fn poll(
        &self,
        since_nonce: u64,
        limit: usize,
        artifact_filter: Option<&HashSet<String>>,
    ) -> ArtifactRealtimePollResult {
        if let Some(canister) = self.canister.as_ref() {
            let canister_nonce = {
                let guard = self.runtime.lock().await;
                guard.canister_nonce
            };
            if let Ok(messages) = canister.ws_poll(canister_nonce).await {
                let mut sorted = messages;
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let mut guard = self.runtime.lock().await;
                let mut expected_seq = guard.canister_nonce.saturating_add(1);
                for (sequence_num, envelope) in sorted {
                    if sequence_num > expected_seq {
                        Self::set_degraded(&mut guard, "gap_detected", None);
                    }
                    expected_seq = sequence_num.saturating_add(1);
                    let key = dedupe_key(&envelope.channel, &envelope.op_id);
                    if !Self::record_seen_key(&mut guard, key) {
                        continue;
                    }
                    guard.canister_nonce = guard.canister_nonce.max(sequence_num);
                    guard.next_nonce = guard.next_nonce.saturating_add(1);
                    let next_nonce = guard.next_nonce;
                    Self::update_ack_cursor(&mut guard, &envelope, sequence_num);
                    guard.delivered.push((next_nonce, envelope));
                }
                if guard.delivered.len() > 2048 {
                    let trim = guard.delivered.len() - 2048;
                    guard.delivered.drain(0..trim);
                }
            } else {
                let mut guard = self.runtime.lock().await;
                if guard.degraded_reason.is_none() {
                    guard.degraded_reason = Some("stream_unavailable".to_string());
                }
            }
        }

        let guard = self.runtime.lock().await;
        let mut events = guard
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
            let drop = events.len() - limit;
            events.drain(0..drop);
        }
        ArtifactRealtimePollResult {
            next_nonce: guard.next_nonce,
            events,
        }
    }

    pub async fn replay_pending(&self) -> Result<usize, String> {
        let canister = match self.canister.as_ref() {
            Some(canister) => canister,
            None => return Ok(0),
        };

        let snapshot = {
            let guard = self.runtime.lock().await;
            guard.replay_queue.clone()
        };

        if snapshot.is_empty() {
            return Ok(0);
        }

        let mut failures = Vec::new();
        for item in snapshot {
            let nonce = {
                let guard = self.runtime.lock().await;
                guard
                    .connected_clients
                    .get(&item.envelope.actor_id)
                    .copied()
                    .unwrap_or_else(|| Utc::now().timestamp_millis().max(0) as u64)
            };
            if let Err(err) = canister.ws_publish(&item.envelope, nonce).await {
                let mut failed = item.clone();
                failed.last_error = Some(err);
                failures.push(failed);
            }
        }

        let mut guard = self.runtime.lock().await;
        guard.replay_queue = failures;
        if guard.replay_queue.is_empty() {
            Self::maybe_clear_degraded(&mut guard);
        } else if guard.degraded_reason.is_none() {
            guard.degraded_reason = Some("replay_backlog".to_string());
        }
        write_replay_queue_file(&guard.replay_queue)?;
        Ok(guard.replay_queue.len())
    }

    pub async fn backlog(&self, artifact_id: Option<&str>) -> Vec<ArtifactRealtimeBacklogItem> {
        let guard = self.runtime.lock().await;
        guard
            .replay_queue
            .iter()
            .filter(|item| artifact_id.map(|id| id == item.artifact_id).unwrap_or(true))
            .cloned()
            .collect()
    }

    pub async fn status(&self) -> ArtifactRealtimeTransportStatus {
        let guard = self.runtime.lock().await;
        let duplicate_rate = if guard.published_total == 0 {
            0.0
        } else {
            guard.duplicate_ops_dropped as f64 / guard.published_total as f64
        };
        ArtifactRealtimeTransportStatus {
            schema_version: "1.0.0".to_string(),
            generated_at: now_iso(),
            realtime_enabled: self.enabled,
            mode: self.mode_label().to_string(),
            primary_available: self.canister.is_some(),
            degraded: !guard.replay_queue.is_empty() || guard.degraded_since.is_some(),
            degraded_since: guard.degraded_since.clone(),
            degraded_reason: guard.degraded_reason.clone(),
            pending_replay: guard.replay_queue.len(),
            convergence_latency_p95_ms: percentile_95(&guard.latencies_ms),
            duplicate_op_drop_rate: duplicate_rate,
            duplicate_ops_dropped: guard.duplicate_ops_dropped,
            published_total: guard.published_total,
            last_error: guard.last_error.clone(),
        }
    }

    pub async fn ack_cursor(&self, artifact_id: &str) -> Option<ArtifactRealtimeAckCursor> {
        let channel = format!("cortex:artifact:{artifact_id}");
        let guard = self.runtime.lock().await;
        guard.ack_cursors.get(&channel).cloned()
    }

    pub async fn reset_ack_cursor(&self, artifact_id: &str) -> bool {
        let channel = format!("cortex:artifact:{artifact_id}");
        let mut guard = self.runtime.lock().await;
        let removed = guard.ack_cursors.remove(&channel).is_some();
        if removed {
            let _ = write_ack_cursors_file(&guard.ack_cursors);
        }
        removed
    }

    pub async fn integrity_report(&self, artifact_id: &str) -> ArtifactRealtimeIntegrityReport {
        let channel = format!("cortex:artifact:{artifact_id}");
        let status = self.status().await;
        let ack = self.ack_cursor(artifact_id).await;
        ArtifactRealtimeIntegrityReport {
            schema_version: "1.0.0".to_string(),
            generated_at: now_iso(),
            artifact_id: artifact_id.to_string(),
            channel,
            source_mode: status.mode,
            primary_available: status.primary_available,
            degraded: status.degraded,
            degraded_reason: status.degraded_reason,
            pending_replay: status.pending_replay,
            convergence_latency_p95_ms: status.convergence_latency_p95_ms,
            duplicate_op_drop_rate: status.duplicate_op_drop_rate,
            ack_cursor: ack,
            last_error: status.last_error,
        }
    }

    pub async fn resync_channel(
        &self,
        artifact_id: &str,
    ) -> Result<ArtifactRealtimeResyncResult, String> {
        let attempted_replay = self.backlog(Some(artifact_id)).await.len();
        let pending_replay_after = self.replay_pending().await?;
        let mut filter = HashSet::new();
        filter.insert(artifact_id.to_string());
        let _ = self.poll(0, 0, Some(&filter)).await;
        let status = self.status().await;
        let ack_cursor = self.ack_cursor(artifact_id).await;
        Ok(ArtifactRealtimeResyncResult {
            schema_version: "1.0.0".to_string(),
            generated_at: now_iso(),
            artifact_id: artifact_id.to_string(),
            attempted_replay,
            pending_replay_after,
            ack_cursor,
            degraded: status.degraded,
            degraded_reason: status.degraded_reason,
        })
    }

    pub async fn slo_status(&self) -> CortexRealtimeSloStatus {
        let mut guard = self.runtime.lock().await;
        Self::evaluate_slo_locked(&mut guard)
    }

    pub async fn slo_breaches(&self) -> Vec<CortexRealtimeSloBreachEvent> {
        let mut guard = self.runtime.lock().await;
        let status = Self::evaluate_slo_locked(&mut guard);
        let breaches: Vec<CortexRealtimeSloBreachEvent> = guard
            .slo_breach_opened_at
            .iter()
            .filter_map(|(metric, opened_at)| {
                let threshold = status.thresholds.get(metric)?.clone();
                let observed = match metric.as_str() {
                    "convergence_latency_p95_ms" => status.convergence_latency_p95_ms.to_string(),
                    "replay_backlog_drain_time_p95_secs" => {
                        status.replay_backlog_drain_time_p95_secs.to_string()
                    }
                    "duplicate_op_drop_rate" => format!("{:.6}", status.duplicate_op_drop_rate),
                    "daily_degraded_duration_secs" => status.degraded_duration_secs_24h.to_string(),
                    _ => "unknown".to_string(),
                };
                let opened_token = opened_at
                    .chars()
                    .filter(|ch| ch.is_ascii_alphanumeric())
                    .collect::<String>();
                Some(CortexRealtimeSloBreachEvent {
                    event_id: format!("slo_breach_{}_{}", metric, opened_token),
                    metric: metric.clone(),
                    threshold,
                    observed,
                    opened_at: opened_at.clone(),
                })
            })
            .collect();
        drop(guard);

        let mut emitted = Vec::<String>::new();
        for event in &breaches {
            let should_emit = {
                let guard = self.runtime.lock().await;
                !guard.emitted_slo_alert_ids.contains(&event.event_id)
            };
            if !should_emit {
                continue;
            }
            if emit_slo_alert(event).await.is_ok() {
                emitted.push(event.event_id.clone());
            }
        }
        if !emitted.is_empty() {
            let mut guard = self.runtime.lock().await;
            for event_id in emitted {
                guard.emitted_slo_alert_ids.insert(event_id);
            }
        }

        breaches
    }

    pub fn mode_label(&self) -> &'static str {
        if self.enabled {
            if self.canister.is_some() {
                "canister_primary"
            } else {
                "local_loopback"
            }
        } else {
            "realtime_disabled"
        }
    }
}

async fn resolve_canister_id(env_keys: &[&str], canister_name: &str) -> Result<Principal, String> {
    let id_str = cortex_ic_adapter::dfx::resolve_canister_id_any(env_keys, canister_name).await?;
    Principal::from_text(id_str.as_str())
        .map_err(|err| format!("invalid principal {id_str}: {err}"))
}

async fn global_manager() -> &'static Arc<StreamingTransportManager> {
    static MANAGER: OnceCell<Arc<StreamingTransportManager>> = OnceCell::const_new();
    MANAGER
        .get_or_init(|| async { Arc::new(StreamingTransportManager::from_env().await) })
        .await
}

pub async fn streaming_transport_manager() -> Arc<StreamingTransportManager> {
    global_manager().await.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loopback_publish_and_poll_are_ordered() {
        let manager = StreamingTransportManager {
            enabled: true,
            canister: None,
            runtime: Arc::new(Mutex::new(StreamingRuntime::default())),
        };
        manager
            .publish(ArtifactRealtimeEnvelope {
                schema_version: "1.0.0".to_string(),
                channel: "cortex:artifact:a".to_string(),
                artifact_id: "a".to_string(),
                session_id: "s".to_string(),
                actor_id: "actor".to_string(),
                op_id: "op-1".to_string(),
                sequence: 1,
                lamport: 1,
                event_type: "op_applied".to_string(),
                timestamp: now_iso(),
                payload: serde_json::json!({"k":1}),
            })
            .await
            .expect("publish");
        let result = manager.poll(0, 10, None).await;
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].op_id, "op-1");
    }
}
