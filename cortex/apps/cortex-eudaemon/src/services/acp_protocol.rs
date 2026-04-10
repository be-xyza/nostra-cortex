use crate::services::acp_adapter::{AcpAdapter, AcpPolicyConfig};
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_projector::AcpProjectedEvent;
use crate::services::acp_event_projector::project_session_update;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_projector::project_session_update_with_timestamp;
use crate::services::acp_event_sink::AcpEventSink;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_sink::RuntimeAcpUpdateRequest;
use crate::services::acp_permission_ledger::{AcpPermissionLedger, DecisionKind};
use crate::services::acp_session_store::{AcpSessionStore, SessionRecord, StoredSessionUpdate};
use crate::services::file_system_service::FileSystemService;
use crate::services::storage_adapter::DesktopStorageAdapter;
use crate::services::terminal_service::{
    AcpTerminalOutputRequest, AcpTerminalWaitRequest, TerminalService,
};
use async_trait::async_trait;
pub use cortex_domain::policy::types::{JsonRpcRequest, JsonRpcResponse};
use cortex_runtime::memory_fs::ContextFs;
use cortex_runtime::policy::protocol::{
    AcpProtocolHost, AcpProtocolRuntime, AcpRpcError, PermissionLedgerPort, ProjectUpdateRequest,
    ProjectUpdateResult, SessionStorePort,
};
use cortex_runtime::ports::TimeProvider;
use serde_json::Value;
use std::sync::Arc;

struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now_unix_secs(&self) -> u64 {
        chrono::Utc::now().timestamp() as u64
    }

    fn to_rfc3339(&self, unix_secs: u64) -> Result<String, cortex_runtime::RuntimeError> {
        match chrono::DateTime::from_timestamp(unix_secs as i64, 0) {
            Some(dt) => Ok(dt.to_rfc3339()),
            None => Err(cortex_runtime::RuntimeError::Domain(
                "Invalid timestamp".to_string(),
            )),
        }
    }
}

pub fn is_acp_pilot_enabled() -> bool {
    match std::env::var("CORTEX_ACP_PILOT") {
        Ok(raw) => matches!(
            raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn is_cortex_runtime_v0_enabled() -> bool {
    if !cfg!(feature = "cortex_runtime_v0") {
        return false;
    }

    match std::env::var("CORTEX_RUNTIME_V0") {
        Ok(raw) => matches!(
            raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

#[cfg(feature = "cortex_runtime_v0")]
fn is_cortex_runtime_shadow_enabled() -> bool {
    match std::env::var("CORTEX_RUNTIME_SHADOW") {
        Ok(raw) => matches!(
            raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

#[cfg(feature = "cortex_runtime_v0")]
fn projected_events_match(left: &AcpProjectedEvent, right: &AcpProjectedEvent) -> bool {
    left.id == right.id
        && left.session_id == right.session_id
        && left.turn_seq == right.turn_seq
        && left.update_seq == right.update_seq
        && left.session_update_kind == right.session_update_kind
        && left.projection_kind == right.projection_kind
        && left.payload == right.payload
        && left.trace == right.trace
}

#[cfg(feature = "cortex_runtime_v0")]
fn projected_event_mismatch_reasons(
    left: &AcpProjectedEvent,
    right: &AcpProjectedEvent,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if left.id != right.id {
        reasons.push("id");
    }
    if left.session_id != right.session_id {
        reasons.push("session_id");
    }
    if left.turn_seq != right.turn_seq {
        reasons.push("turn_seq");
    }
    if left.update_seq != right.update_seq {
        reasons.push("update_seq");
    }
    if left.session_update_kind != right.session_update_kind {
        reasons.push("session_update_kind");
    }
    if left.projection_kind != right.projection_kind {
        reasons.push("projection_kind");
    }
    if left.payload != right.payload {
        reasons.push("payload");
    }
    if left.trace != right.trace {
        reasons.push("trace");
    }
    reasons
}

#[derive(Clone)]
struct DesktopProtocolHost {
    event_sink: AcpEventSink,
}

impl DesktopProtocolHost {
    fn load_default() -> Self {
        Self {
            event_sink: AcpEventSink::load_default(),
        }
    }

    #[cfg(test)]
    fn with_event_sink(event_sink: AcpEventSink) -> Self {
        Self { event_sink }
    }
}

#[async_trait]
impl AcpProtocolHost for DesktopProtocolHost {
    fn server_name(&self) -> &'static str {
        "cortex-desktop"
    }

    fn server_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn default_workspace_path(&self) -> String {
        FileSystemService::get_root_path().display().to_string()
    }

    async fn project_update(
        &mut self,
        request: ProjectUpdateRequest,
    ) -> Result<ProjectUpdateResult, String> {
        if is_cortex_runtime_v0_enabled() {
            #[cfg(feature = "cortex_runtime_v0")]
            {
                let runtime_projected = self
                    .event_sink
                    .record_event_runtime_v0(RuntimeAcpUpdateRequest {
                        session_id: request.session_id.clone(),
                        turn_seq: request.turn_seq,
                        update_seq: request.update_seq,
                        update_kind: request.update_kind,
                        payload: request.payload.clone(),
                        trace: request.trace.clone(),
                    })
                    .await?;

                if is_cortex_runtime_shadow_enabled() {
                    let legacy_projected = project_session_update_with_timestamp(
                        &request.session_id,
                        request.turn_seq,
                        request.update_seq,
                        request.update_kind,
                        request.payload,
                        request.trace,
                        runtime_projected.timestamp,
                    );

                    if !projected_events_match(&legacy_projected, &runtime_projected) {
                        let reasons =
                            projected_event_mismatch_reasons(&legacy_projected, &runtime_projected);
                        tracing::warn!(
                            target: "cortex_runtime_shadow",
                            event = "cortex_runtime_shadow_mismatch",
                            session_id = %request.session_id,
                            turn_seq = request.turn_seq,
                            update_seq = request.update_seq,
                            mismatch_fields = %reasons.join(","),
                            "legacy and runtime projections diverged"
                        );
                    }
                }

                return Ok(ProjectUpdateResult {
                    timestamp: runtime_projected.timestamp,
                    event_id: runtime_projected.id,
                });
            }
            #[cfg(not(feature = "cortex_runtime_v0"))]
            {
                return Err("cortex_runtime_v0 requested but feature is not enabled".to_string());
            }
        }

        let projected = project_session_update(
            &request.session_id,
            request.turn_seq,
            request.update_seq,
            request.update_kind,
            request.payload,
            request.trace,
        );

        self.event_sink.record_event(&projected).await?;
        Ok(ProjectUpdateResult {
            timestamp: projected.timestamp,
            event_id: projected.id,
        })
    }

    async fn terminal_create(
        &mut self,
        request: crate::services::acp_adapter::ValidatedTerminalCreate,
    ) -> Result<Value, String> {
        let response = TerminalService::acp_terminal_create(request).await?;
        serde_json::to_value(response).map_err(|e| e.to_string())
    }

    async fn terminal_output(
        &mut self,
        terminal_id: String,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let response =
            TerminalService::acp_terminal_output(AcpTerminalOutputRequest { terminal_id, limit })
                .await?;
        serde_json::to_value(response).map_err(|e| e.to_string())
    }

    async fn terminal_wait_for_exit(
        &mut self,
        terminal_id: String,
        timeout_ms: Option<u64>,
    ) -> Result<Value, String> {
        let response = TerminalService::acp_terminal_wait_for_exit(AcpTerminalWaitRequest {
            terminal_id,
            timeout_ms,
        })
        .await?;
        serde_json::to_value(response).map_err(|e| e.to_string())
    }

    async fn terminal_kill(&mut self, terminal_id: String) -> Result<Value, String> {
        let response = TerminalService::acp_terminal_kill(terminal_id).await?;
        serde_json::to_value(response).map_err(|e| e.to_string())
    }

    async fn terminal_release(&mut self, terminal_id: String) -> Result<Value, String> {
        let response = TerminalService::acp_terminal_release(terminal_id)?;
        serde_json::to_value(response).map_err(|e| e.to_string())
    }

    async fn on_memory_fs_commit(
        &mut self,
        session_id: &str,
        branch: &str,
        commit_oid: &str,
    ) -> Result<(), String> {
        let content = format!(
            "Title: Agent Memory Trace: {} [{}]\nSession ID: {}\nBranch: {}\nCommit OID: {}\nThis is a semantic summary of the agent's recent working memory trace. It captures the trajectory and decisions made during the sequence.",
            session_id, branch, session_id, branch, commit_oid
        );
        let id = format!("{}-{}", session_id, commit_oid);
        let _ = crate::services::agent_service::AgentService::index(
            id,
            content,
            crate::services::agent_service::Modality::Text,
        )
        .await;
        Ok(())
    }
}

impl SessionStorePort for AcpSessionStore {
    fn create_session(&mut self, session_id: String, cwd: String) -> SessionRecord {
        AcpSessionStore::create_session(self, session_id, cwd)
    }

    fn get_session(&self, session_id: &str) -> Option<SessionRecord> {
        AcpSessionStore::get_session(self, session_id).cloned()
    }

    fn start_turn(&mut self, session_id: &str) -> Result<u64, String> {
        AcpSessionStore::start_turn(self, session_id)
    }

    fn next_update_seq(&mut self, session_id: &str) -> Result<u64, String> {
        AcpSessionStore::next_update_seq(self, session_id)
    }

    fn append_update(
        &mut self,
        session_id: &str,
        update: StoredSessionUpdate,
    ) -> Result<(), String> {
        AcpSessionStore::append_update(self, session_id, update)
    }

    fn set_cancelled(&mut self, session_id: &str, cancelled: bool) -> Result<(), String> {
        AcpSessionStore::set_cancelled(self, session_id, cancelled)
    }

    fn set_config_option(
        &mut self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> Result<(), String> {
        AcpSessionStore::set_config_option(self, session_id, config_id, value)
    }

    fn config_options(&self, session_id: &str) -> Result<Vec<Value>, String> {
        AcpSessionStore::config_options(self, session_id)
    }

    fn replay_updates(&self, session_id: &str) -> Result<Vec<StoredSessionUpdate>, String> {
        AcpSessionStore::replay_updates(self, session_id)
    }

    fn save(&self) -> Result<(), String> {
        AcpSessionStore::save(self)
    }
}

impl PermissionLedgerPort for AcpPermissionLedger {
    fn record(
        &mut self,
        session_id: String,
        tool_call_id: String,
        option_id: String,
        kind: DecisionKind,
        source: String,
    ) {
        AcpPermissionLedger::record(self, session_id, tool_call_id, option_id, kind, source)
    }

    fn set_session_policy(&mut self, session_id: &str, policy_key: &str, kind: DecisionKind) {
        AcpPermissionLedger::set_session_policy(self, session_id, policy_key, kind)
    }

    fn save(&self) -> Result<(), String> {
        AcpPermissionLedger::save(self)
    }
}

fn load_runtime() -> Result<
    AcpProtocolRuntime<AcpAdapter, AcpSessionStore, AcpPermissionLedger, DesktopProtocolHost>,
    String,
> {
    let workflow_root = FileSystemService::get_root_path();
    let cfg = AcpPolicyConfig::baseline(vec![workflow_root.clone()]);

    let storage_dir = workflow_root.join(".cortex").join("storage");
    let storage = Arc::new(DesktopStorageAdapter::new(storage_dir));
    let time = Arc::new(SystemTimeProvider);
    let context_fs = Arc::new(ContextFs::new(storage, time));

    Ok(AcpProtocolRuntime::new(
        AcpAdapter::new(cfg).map_err(|e| e.to_string())?,
        AcpSessionStore::load_default()?,
        AcpPermissionLedger::load_default()?,
        DesktopProtocolHost::load_default(),
        Some(context_fs),
    ))
}

pub async fn handle_rpc_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();
    if !is_acp_pilot_enabled() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(AcpRpcError::pilot_disabled().into_jsonrpc()),
        };
    }

    let mut runtime = match load_runtime() {
        Ok(runtime) => runtime,
        Err(err) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(AcpRpcError::internal(err).into_jsonrpc()),
            };
        }
    };

    match runtime.handle(request).await {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        },
        Err(err) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(err.into_jsonrpc()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cortex_runtime_v0")]
    use crate::services::acp_adapter::AcpSessionUpdateKind;
    #[cfg(feature = "cortex_runtime_v0")]
    use crate::services::acp_meta_policy::TraceContext;
    use serde_json::json;
    use std::collections::HashSet;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn test_runtime() -> (
        AcpProtocolRuntime<AcpAdapter, AcpSessionStore, AcpPermissionLedger, DesktopProtocolHost>,
        std::path::PathBuf,
    ) {
        let root = std::env::temp_dir().join(format!("acp_protocol_root_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();

        let mut cfg = AcpPolicyConfig::baseline(vec![root.clone()]);
        cfg.allowed_terminal_commands = HashSet::from_iter([
            "echo".to_string(),
            "sh".to_string(),
            "ls".to_string(),
            "cat".to_string(),
        ]);
        cfg.allowed_env_vars = HashSet::from_iter(["RUST_LOG".to_string()]);

        let session_path =
            std::env::temp_dir().join(format!("acp_sessions_{}.json", uuid::Uuid::new_v4()));
        let ledger_path =
            std::env::temp_dir().join(format!("acp_ledger_{}.json", uuid::Uuid::new_v4()));
        let events_path =
            std::env::temp_dir().join(format!("acp_events_{}.jsonl", uuid::Uuid::new_v4()));

        let storage = Arc::new(DesktopStorageAdapter::new(
            root.join(".cortex").join("storage"),
        ));
        let time = Arc::new(SystemTimeProvider);
        let context_fs = Arc::new(ContextFs::new(storage, time));

        (
            AcpProtocolRuntime::new(
                AcpAdapter::new(cfg).unwrap(),
                AcpSessionStore::load(session_path).unwrap(),
                AcpPermissionLedger::load(ledger_path).unwrap(),
                DesktopProtocolHost::with_event_sink(AcpEventSink::with_paths(events_path, None)),
                Some(context_fs),
            ),
            root,
        )
    }

    fn rpc(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: method.to_string(),
            params: Some(params),
        }
    }

    #[tokio::test]
    async fn initialize_returns_capabilities() {
        let (mut runtime, _) = test_runtime();
        let response = runtime.handle(rpc("initialize", json!({}))).await.unwrap();
        assert_eq!(response["protocolVersion"], "0.1-pilot");
        assert!(response["capabilities"]["session"].is_array());
    }

    #[tokio::test]
    async fn session_new_rejects_non_absolute_cwd() {
        let (mut runtime, _) = test_runtime();
        let err = runtime
            .handle(rpc("session/new", json!({ "cwd": "relative/path" })))
            .await
            .unwrap_err();
        assert_eq!(err.code, -32010);
        assert_eq!(
            err.data.unwrap()["errorCode"],
            "ACP_POLICY_PATH_NOT_ABSOLUTE"
        );
    }

    #[tokio::test]
    async fn session_new_rejects_legacy_snake_case_param_aliases() {
        let (mut runtime, root) = test_runtime();
        let err = runtime
            .handle(rpc(
                "session/new",
                json!({
                    "cwd": root.display().to_string(),
                    "session_id": "sess_legacy_alias"
                }),
            ))
            .await
            .unwrap_err();

        assert_eq!(err.code, -32602);
        assert!(err.message.contains("unknown field"));
        assert!(err.message.contains("session_id"));
    }

    #[tokio::test]
    async fn session_prompt_and_load_preserve_ordering() {
        let (mut runtime, root) = test_runtime();
        let created = runtime
            .handle(rpc(
                "session/new",
                json!({ "cwd": root.display().to_string() }),
            ))
            .await
            .unwrap();
        let session_id = created["sessionId"].as_str().unwrap().to_string();

        let prompted = runtime
            .handle(rpc(
                "session/prompt",
                json!({ "sessionId": session_id, "prompt": "Hello ACP" }),
            ))
            .await
            .unwrap();
        assert_eq!(prompted["stopReason"], "completed");
        assert_eq!(prompted["updates"].as_array().unwrap().len(), 2);

        let loaded = runtime
            .handle(rpc("session/load", json!({ "sessionId": session_id })))
            .await
            .unwrap();
        let updates = loaded["updates"].as_array().unwrap();
        assert!(updates.len() >= 2);
        assert!(
            updates[0]["update_seq"].as_u64().unwrap()
                <= updates[1]["update_seq"].as_u64().unwrap()
        );
    }

    #[tokio::test]
    async fn session_cancel_reports_late_update_acceptance() {
        let (mut runtime, root) = test_runtime();
        let created = runtime
            .handle(rpc(
                "session/new",
                json!({ "cwd": root.display().to_string() }),
            ))
            .await
            .unwrap();
        let session_id = created["sessionId"].as_str().unwrap().to_string();

        let _ = runtime
            .handle(rpc(
                "session/prompt",
                json!({ "sessionId": session_id, "prompt": "Cancel me" }),
            ))
            .await
            .unwrap();

        let cancelled = runtime
            .handle(rpc("session/cancel", json!({ "sessionId": session_id })))
            .await
            .unwrap();
        assert_eq!(cancelled["stopReason"], "cancelled");
        assert_eq!(cancelled["acceptsLateToolUpdates"], true);
    }

    #[tokio::test]
    async fn session_request_permission_maps_all_decision_kinds() {
        let (mut runtime, root) = test_runtime();
        let created = runtime
            .handle(rpc(
                "session/new",
                json!({ "cwd": root.display().to_string() }),
            ))
            .await
            .unwrap();
        let session_id = created["sessionId"].as_str().unwrap().to_string();

        for option in ["allow_once", "reject_once", "allow_always", "reject_always"] {
            let response = runtime
                .handle(rpc(
                    "session/request_permission",
                    json!({
                        "sessionId": session_id,
                        "toolCallId": format!("call_{}", option),
                        "optionId": option,
                        "policyKey": "execute"
                    }),
                ))
                .await
                .unwrap();

            assert_eq!(response["decision"], option);
            let expect_policy = option.ends_with("always");
            assert_eq!(response["sessionPolicyApplied"], expect_policy);
        }
    }

    #[tokio::test]
    async fn rpc_handler_returns_pilot_disabled_when_flag_is_off() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("CORTEX_ACP_PILOT");
        let response = handle_rpc_request(rpc("initialize", json!({}))).await;
        assert_eq!(response.error.unwrap().code, -32030);
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[test]
    fn shadow_projection_matcher_rejects_non_allowed_drift() {
        let left = project_session_update(
            "sess_shadow",
            1,
            1,
            AcpSessionUpdateKind::Plan,
            json!({"value": 1}),
            TraceContext::default(),
        );
        let mut right = left.clone();
        right.payload = json!({"value": 2});

        assert!(
            !projected_events_match(&left, &right),
            "shadow parity should fail on payload drift"
        );
        assert!(
            projected_event_mismatch_reasons(&left, &right).contains(&"payload"),
            "payload drift reason must be captured"
        );
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[test]
    fn shadow_projection_matcher_rejects_projection_kind_drift() {
        let left = project_session_update(
            "sess_shadow",
            1,
            1,
            AcpSessionUpdateKind::Plan,
            json!({"value": 1}),
            TraceContext::default(),
        );
        let mut right = left.clone();
        right.projection_kind = crate::services::acp_adapter::NostraProjectionKind::ModeChanged;

        assert!(
            !projected_events_match(&left, &right),
            "shadow parity should fail on projection kind drift"
        );
        assert!(
            projected_event_mismatch_reasons(&left, &right).contains(&"projection_kind"),
            "projection kind drift reason must be captured"
        );
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[test]
    fn shadow_projection_matcher_rejects_event_id_drift() {
        let left = project_session_update(
            "sess_shadow",
            1,
            1,
            AcpSessionUpdateKind::Plan,
            json!({"value": 1}),
            TraceContext::default(),
        );
        let mut right = left.clone();
        right.id = "acp_evt_mismatch".to_string();

        assert!(
            !projected_events_match(&left, &right),
            "shadow parity should fail on event id drift"
        );
        assert!(
            projected_event_mismatch_reasons(&left, &right).contains(&"id"),
            "event id drift reason must be captured"
        );
    }

    #[cfg(feature = "cortex_runtime_v0")]
    #[test]
    fn shadow_projection_matcher_allows_timestamp_only_drift() {
        let left = project_session_update(
            "sess_shadow",
            1,
            1,
            AcpSessionUpdateKind::Plan,
            json!({"value": 1}),
            TraceContext::default(),
        );
        let mut right = left.clone();
        right.timestamp += 1;

        assert!(
            projected_events_match(&left, &right),
            "timestamp-only drift should remain allowed in shadow parity"
        );
    }
}
