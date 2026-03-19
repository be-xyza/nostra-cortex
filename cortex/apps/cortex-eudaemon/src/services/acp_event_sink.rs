use crate::services::acp_event_projector::{AcpProjectedEvent, to_cloud_event};
use crate::services::acp_metrics::{record_emit_attempt, record_emit_failure, record_emit_success};
use crate::services::local_gateway::get_gateway;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_adapter::AcpSessionUpdateKind;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_projector::{
    from_domain_projected_event, to_domain_trace_context, to_domain_update_kind,
};
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_meta_policy::TraceContext;
#[cfg(feature = "cortex_runtime_v0")]
use async_trait::async_trait;
#[cfg(feature = "cortex_runtime_v0")]
use cortex_runtime::event_bus::{EventOrchestrator, RuntimeSessionUpdateRequest};
#[cfg(feature = "cortex_runtime_v0")]
use cortex_runtime::ports::{EventBus, LogAdapter, NetworkAdapter, TimeProvider};
#[cfg(feature = "cortex_runtime_v0")]
use cortex_runtime::{RuntimeConfig, RuntimeError};
#[cfg(feature = "cortex_runtime_v0")]
use serde_json::Value;
#[cfg(feature = "cortex_runtime_v0")]
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AcpEventSink {
    event_log_path: PathBuf,
    log_registry_endpoint: Option<String>,
}

#[cfg(feature = "cortex_runtime_v0")]
#[derive(Debug, Clone)]
pub struct RuntimeAcpUpdateRequest {
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub update_kind: AcpSessionUpdateKind,
    pub payload: Value,
    pub trace: TraceContext,
}

impl AcpEventSink {
    pub fn load_default() -> Self {
        Self {
            event_log_path: Self::default_path(),
            log_registry_endpoint: std::env::var("CORTEX_ACP_LOG_REGISTRY_URL").ok(),
        }
    }

    #[cfg(test)]
    pub fn with_paths(event_log_path: PathBuf, log_registry_endpoint: Option<String>) -> Self {
        Self {
            event_log_path,
            log_registry_endpoint,
        }
    }

    pub async fn record_event(&self, projected: &AcpProjectedEvent) -> Result<(), String> {
        self.append_local(projected)?;

        let cloud_event = to_cloud_event(projected)?;
        if let Some(endpoint) = &self.log_registry_endpoint {
            match Self::emit_to_log_registry(endpoint, &projected.id, &cloud_event).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    self.enqueue_fallback(projected, &err);
                    return Ok(());
                }
            }
        }

        // No remote endpoint in local mode; local durable JSONL remains the canonical pilot record.
        Ok(())
    }

    #[cfg(feature = "cortex_runtime_v0")]
    pub async fn record_event_runtime_v0(
        &self,
        request: RuntimeAcpUpdateRequest,
    ) -> Result<AcpProjectedEvent, String> {
        let orchestrator = self.build_runtime_orchestrator();
        let result = orchestrator
            .publish_session_update(RuntimeSessionUpdateRequest {
                session_id: request.session_id,
                turn_seq: request.turn_seq,
                update_seq: request.update_seq,
                update_kind: to_domain_update_kind(request.update_kind),
                payload: request.payload,
                trace: to_domain_trace_context(&request.trace),
                timestamp_secs: None,
            })
            .await
            .map_err(|e| e.to_string())?;

        let projected = from_domain_projected_event(result.projected);
        if let Some(err) = result.network_error {
            self.enqueue_fallback(&projected, &err);
        }

        Ok(projected)
    }

    fn append_local(&self, projected: &AcpProjectedEvent) -> Result<(), String> {
        if let Some(parent) = self.event_log_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.event_log_path)
            .map_err(|e| e.to_string())?;

        let line = serde_json::to_string(projected).map_err(|e| e.to_string())?;
        writeln!(file, "{}", line).map_err(|e| e.to_string())
    }

    async fn emit_to_log_registry(
        endpoint: &str,
        idempotency_key: &str,
        cloud_event: &nostra_cloudevents::Event,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let retry_delays_ms = [250u64, 500, 1000];
        let max_retries = retry_delays_ms.len();
        let mut last_err = "unknown emit error".to_string();

        for attempt in 0..=max_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    retry_delays_ms[attempt - 1],
                ))
                .await;
            }

            record_emit_attempt();
            let response = client
                .post(endpoint)
                .header("X-Idempotency-Key", idempotency_key)
                .json(cloud_event)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    record_emit_success();
                    return Ok(());
                }
                Ok(resp) => {
                    record_emit_failure();
                    let status = resp.status();
                    last_err = format!("log-registry returned {}", status);
                    if !is_transient_status(status) {
                        return Err(last_err);
                    }
                }
                Err(err) => {
                    record_emit_failure();
                    last_err = err.to_string();
                }
            }
        }

        Err(last_err)
    }

    fn enqueue_fallback(&self, projected: &AcpProjectedEvent, err: &str) {
        let payload = json!({
            "kind": "acp_observability_event",
            "event": projected,
            "error": err,
            "timestamp": now_secs(),
            "source": "cortex-desktop"
        });

        if let Err(queue_err) = get_gateway().queue_observability_payload(&payload) {
            tracing::warn!(
                "failed to queue ACP observability fallback in local gateway: {}",
                queue_err
            );
        }

        if let Err(queue_err) = self.append_fallback_payload(&payload) {
            tracing::warn!(
                "failed to enqueue ACP observability fallback: {}",
                queue_err
            );
        }
    }

    fn append_fallback_payload(&self, payload: &serde_json::Value) -> Result<(), String> {
        let path = self.fallback_log_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let line = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        writeln!(file, "{}", line).map_err(|e| e.to_string())
    }

    fn fallback_log_path(&self) -> PathBuf {
        let stem = self
            .event_log_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("acp_events");
        self.event_log_path
            .with_file_name(format!("{}_fallback.jsonl", stem))
    }

    fn default_path() -> PathBuf {
        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".cortex").join("acp_events.jsonl")
    }

    #[cfg(feature = "cortex_runtime_v0")]
    fn build_runtime_orchestrator(&self) -> EventOrchestrator {
        let config = RuntimeConfig {
            event_source: "nostra://cortex-desktop/acp".to_string(),
            event_type_prefix: "nostra.acp".to_string(),
            remote_endpoint: self.log_registry_endpoint.clone(),
            fail_on_network_error: false,
            shadow_mode: false,
            gateway: Default::default(),
        };

        EventOrchestrator::new(
            config,
            Arc::new(DesktopTimeProvider),
            Arc::new(DesktopLogAdapter),
            Arc::new(DesktopEventBus {
                path: self.event_log_path.clone(),
            }),
            None,
            self.log_registry_endpoint
                .as_ref()
                .map(|_| Arc::new(DesktopNetworkAdapter) as Arc<dyn NetworkAdapter>),
        )
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 429 || status.is_server_error()
}

#[cfg(feature = "cortex_runtime_v0")]
struct DesktopTimeProvider;

#[cfg(feature = "cortex_runtime_v0")]
impl TimeProvider for DesktopTimeProvider {
    fn now_unix_secs(&self) -> u64 {
        now_secs()
    }

    fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(unix_secs as i64, 0)
            .ok_or(RuntimeError::InvalidTimestamp(unix_secs))?;
        Ok(timestamp.to_rfc3339())
    }
}

#[cfg(feature = "cortex_runtime_v0")]
struct DesktopLogAdapter;

#[cfg(feature = "cortex_runtime_v0")]
impl LogAdapter for DesktopLogAdapter {
    fn info(&self, message: &str) {
        tracing::info!("{}", message);
    }

    fn warn(&self, message: &str) {
        tracing::warn!("{}", message);
    }

    fn error(&self, message: &str) {
        tracing::error!("{}", message);
    }
}

#[cfg(feature = "cortex_runtime_v0")]
struct DesktopEventBus {
    path: PathBuf,
}

#[cfg(feature = "cortex_runtime_v0")]
#[async_trait]
impl EventBus for DesktopEventBus {
    async fn append_projected_event(
        &self,
        event: &cortex_domain::events::ProjectedEvent,
    ) -> Result<(), RuntimeError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| RuntimeError::Storage(e.to_string()))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| RuntimeError::Storage(e.to_string()))?;

        let line =
            serde_json::to_string(event).map_err(|e| RuntimeError::Serialization(e.to_string()))?;
        writeln!(file, "{}", line).map_err(|e| RuntimeError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "cortex_runtime_v0")]
struct DesktopNetworkAdapter;

#[cfg(feature = "cortex_runtime_v0")]
#[async_trait]
impl NetworkAdapter for DesktopNetworkAdapter {
    async fn post_json(
        &self,
        endpoint: &str,
        idempotency_key: &str,
        body: &Value,
    ) -> Result<(), RuntimeError> {
        let cloud_event: nostra_cloudevents::Event = serde_json::from_value(body.clone())
            .map_err(|e| RuntimeError::Serialization(e.to_string()))?;
        AcpEventSink::emit_to_log_registry(endpoint, idempotency_key, &cloud_event)
            .await
            .map_err(RuntimeError::Network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::acp_adapter::AcpSessionUpdateKind;
    use crate::services::acp_event_projector::project_session_update;
    #[cfg(feature = "cortex_runtime_v0")]
    use crate::services::acp_event_projector::{
        project_session_update_with_timestamp, to_cloud_event as legacy_to_cloud_event,
        to_domain_trace_context, to_domain_update_kind,
    };
    use crate::services::acp_meta_policy::TraceContext;
    #[cfg(feature = "cortex_runtime_v0")]
    use cortex_domain::events::{
        ProjectSessionUpdateInput, project_session_update as domain_project,
    };
    #[cfg(feature = "cortex_runtime_v0")]
    use cortex_runtime::event_bus::to_cloud_event as runtime_to_cloud_event;
    #[cfg(feature = "cortex_runtime_v0")]
    use cortex_runtime::{RuntimeError, ports::TimeProvider};

    #[tokio::test]
    async fn writes_local_jsonl_event() {
        let path = std::env::temp_dir().join(format!(
            "acp_event_sink_{}.jsonl",
            uuid::Uuid::new_v4().simple()
        ));
        let sink = AcpEventSink::with_paths(path.clone(), None);

        let projected = project_session_update(
            "sess_1",
            1,
            1,
            AcpSessionUpdateKind::AgentMessageChunk,
            serde_json::json!({"text": "ok"}),
            TraceContext::default(),
        );

        sink.record_event(&projected).await.unwrap();

        let raw = fs::read_to_string(path).unwrap();
        assert!(raw.contains("sess_1"));
        assert!(raw.contains("agent_message_chunk"));
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[tokio::test]
    async fn runtime_v0_path_writes_event() {
        let path = std::env::temp_dir().join(format!(
            "acp_event_sink_v0_{}.jsonl",
            uuid::Uuid::new_v4().simple()
        ));
        let sink = AcpEventSink::with_paths(path.clone(), None);

        let projected = sink
            .record_event_runtime_v0(RuntimeAcpUpdateRequest {
                session_id: "sess_v0".to_string(),
                turn_seq: 1,
                update_seq: 1,
                update_kind: AcpSessionUpdateKind::Plan,
                payload: serde_json::json!({"ok": true}),
                trace: TraceContext::default(),
            })
            .await
            .unwrap();

        assert_eq!(projected.session_id, "sess_v0");
        let raw = fs::read_to_string(path).unwrap();
        assert!(raw.contains("sess_v0"));
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[tokio::test]
    async fn runtime_v0_matches_legacy_projection_shape() {
        let path = std::env::temp_dir().join(format!(
            "acp_event_sink_v0_regression_{}.jsonl",
            uuid::Uuid::new_v4().simple()
        ));
        let sink = AcpEventSink::with_paths(path, None);

        let payload = serde_json::json!({"prompt": "hello"});
        let trace = TraceContext::default();
        let runtime = sink
            .record_event_runtime_v0(RuntimeAcpUpdateRequest {
                session_id: "sess_regression".to_string(),
                turn_seq: 2,
                update_seq: 3,
                update_kind: AcpSessionUpdateKind::ToolCallUpdate,
                payload: payload.clone(),
                trace: trace.clone(),
            })
            .await
            .unwrap();

        let legacy = project_session_update_with_timestamp(
            "sess_regression",
            2,
            3,
            AcpSessionUpdateKind::ToolCallUpdate,
            payload,
            trace,
            runtime.timestamp,
        );

        assert_eq!(runtime.id, legacy.id);
        assert_eq!(runtime.projection_kind, legacy.projection_kind);
        assert_eq!(runtime.session_update_kind, legacy.session_update_kind);
        assert_eq!(runtime.payload, legacy.payload);
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[tokio::test]
    async fn runtime_v0_matches_legacy_for_all_update_kinds() {
        let path = std::env::temp_dir().join(format!(
            "acp_event_sink_v0_matrix_{}.jsonl",
            uuid::Uuid::new_v4().simple()
        ));
        let sink = AcpEventSink::with_paths(path, None);

        for kind in all_update_kinds() {
            let payload = serde_json::json!({"kind": format!("{:?}", kind)});
            let trace = TraceContext::default();

            let runtime = sink
                .record_event_runtime_v0(RuntimeAcpUpdateRequest {
                    session_id: "sess_matrix".to_string(),
                    turn_seq: 7,
                    update_seq: 9,
                    update_kind: kind,
                    payload: payload.clone(),
                    trace: trace.clone(),
                })
                .await
                .unwrap();

            let legacy = project_session_update_with_timestamp(
                "sess_matrix",
                7,
                9,
                kind,
                payload,
                trace,
                runtime.timestamp,
            );

            assert_eq!(runtime.id, legacy.id, "event id mismatch for {:?}", kind);
            assert_eq!(
                runtime.projection_kind, legacy.projection_kind,
                "projection mismatch for {:?}",
                kind
            );
            assert_eq!(
                runtime.session_update_kind, legacy.session_update_kind,
                "update kind mismatch for {:?}",
                kind
            );
            assert_eq!(
                runtime.payload, legacy.payload,
                "payload mismatch for {:?}",
                kind
            );
            assert_eq!(runtime.trace, legacy.trace, "trace mismatch for {:?}", kind);
        }
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[test]
    fn cloud_event_parity_allows_timestamp_format_differences_only() {
        let time_provider = DeterministicTimeProvider;
        let timestamp_seed = 314_u64;

        for kind in all_update_kinds() {
            let trace = TraceContext::default();
            let payload = serde_json::json!({"kind": format!("{:?}", kind)});
            let timestamp = deterministic_timestamp_from_seed(timestamp_seed);

            let legacy = project_session_update_with_timestamp(
                "sess_cloud",
                4,
                5,
                kind,
                payload.clone(),
                trace.clone(),
                timestamp,
            );
            let legacy_cloud = legacy_to_cloud_event(&legacy).unwrap();

            let domain_projected = domain_project(ProjectSessionUpdateInput {
                session_id: "sess_cloud".to_string(),
                turn_seq: 4,
                update_seq: 5,
                kind: to_domain_update_kind(kind),
                payload,
                trace: to_domain_trace_context(&trace),
                timestamp,
            })
            .unwrap();
            let runtime_cloud = runtime_to_cloud_event(
                &domain_projected,
                "nostra://cortex-desktop/acp",
                "nostra.acp",
                &time_provider,
            )
            .unwrap();

            assert_eq!(
                legacy_cloud.id, runtime_cloud.id,
                "id mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.source, runtime_cloud.source,
                "source mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.subject, runtime_cloud.subject,
                "subject mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.spec_version, runtime_cloud.spec_version,
                "spec_version mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.type_, runtime_cloud.type_,
                "type mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.data, runtime_cloud.data,
                "data mismatch for {:?}",
                kind
            );
            assert_eq!(
                legacy_cloud.time.unwrap().timestamp(),
                runtime_cloud.time.unwrap().timestamp(),
                "timestamp second mismatch for {:?}",
                kind
            );
        }
    }

    #[cfg(feature = "cortex_runtime_v0")]
    struct DeterministicTimeProvider;

    #[cfg(feature = "cortex_runtime_v0")]
    impl TimeProvider for DeterministicTimeProvider {
        fn now_unix_secs(&self) -> u64 {
            1_736_000_000
        }

        fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
            let ts = chrono::DateTime::<chrono::Utc>::from_timestamp(unix_secs as i64, 0)
                .ok_or(RuntimeError::InvalidTimestamp(unix_secs))?;
            Ok(ts.to_rfc3339())
        }
    }

    #[cfg(feature = "cortex_runtime_v0")]
    fn all_update_kinds() -> [AcpSessionUpdateKind; 9] {
        [
            AcpSessionUpdateKind::UserMessageChunk,
            AcpSessionUpdateKind::AgentMessageChunk,
            AcpSessionUpdateKind::AgentThoughtChunk,
            AcpSessionUpdateKind::ToolCall,
            AcpSessionUpdateKind::ToolCallUpdate,
            AcpSessionUpdateKind::Plan,
            AcpSessionUpdateKind::AvailableCommandsUpdate,
            AcpSessionUpdateKind::CurrentModeUpdate,
            AcpSessionUpdateKind::ConfigOptionUpdate,
        ]
    }

    #[cfg(feature = "cortex_runtime_v0")]
    fn deterministic_timestamp_from_seed(seed: u64) -> u64 {
        // Test-only deterministic time source to stabilize replay and cloud-event parity.
        1_730_000_000 + (seed % 86_400)
    }
}
