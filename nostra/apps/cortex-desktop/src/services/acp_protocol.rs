use crate::services::acp_adapter::{
    AcpAdapter, AcpPolicyConfig, AcpPolicyError, AcpSessionUpdateKind, TerminalCreateRequest,
};
use crate::services::acp_event_projector::project_session_update;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_projector::AcpProjectedEvent;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_projector::project_session_update_with_timestamp;
use crate::services::acp_event_sink::AcpEventSink;
#[cfg(feature = "cortex_runtime_v0")]
use crate::services::acp_event_sink::RuntimeAcpUpdateRequest;
use crate::services::acp_permission_ledger::{AcpPermissionLedger, DecisionKind};
use crate::services::acp_session_store::{AcpSessionStore, StoredSessionUpdate};
use crate::services::file_system_service::FileSystemService;
use crate::services::terminal_service::{
    AcpTerminalOutputRequest, AcpTerminalWaitRequest, TerminalService,
};
use cortex_domain::policy::meta::{TraceContext, validate_meta};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex as TokioMutex;

static ACP_RUNTIME: OnceLock<Arc<TokioMutex<AcpProtocolRuntime>>> = OnceLock::new();

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
fn projected_event_mismatch_reasons(left: &AcpProjectedEvent, right: &AcpProjectedEvent) -> Vec<&'static str> {
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

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug)]
struct AcpRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl AcpRpcError {
    fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }

    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {}", method),
            data: None,
        }
    }

    fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: -32000,
            message: msg.into(),
            data: None,
        }
    }

    fn pilot_disabled() -> Self {
        Self {
            code: -32030,
            message: "ACP pilot is disabled".to_string(),
            data: Some(json!({
                "errorCode": "ACP_PILOT_DISABLED",
                "category": "policy"
            })),
        }
    }

    fn with_policy(err: AcpPolicyError) -> Self {
        let policy_code = match err {
            AcpPolicyError::PathNotAbsolute(_) => "ACP_POLICY_PATH_NOT_ABSOLUTE",
            AcpPolicyError::PathOutsideAllowedRoots(_) => "ACP_POLICY_PATH_OUTSIDE_ALLOWED_ROOTS",
            AcpPolicyError::CommandNotAllowed(_) => "ACP_POLICY_COMMAND_NOT_ALLOWED",
            AcpPolicyError::EnvVarNotAllowed(_) => "ACP_POLICY_ENV_NOT_ALLOWED",
            AcpPolicyError::OutputLimitExceeded { .. } => "ACP_POLICY_OUTPUT_LIMIT_EXCEEDED",
            AcpPolicyError::InvalidLineNumber(_) => "ACP_POLICY_INVALID_LINE",
            AcpPolicyError::InvalidLimit(_) => "ACP_POLICY_INVALID_LIMIT",
            AcpPolicyError::EmptyCommand => "ACP_POLICY_EMPTY_COMMAND",
            AcpPolicyError::NoAllowedRootsConfigured => "ACP_POLICY_NO_ALLOWED_ROOTS",
            AcpPolicyError::Io(_) => "ACP_POLICY_IO",
        };

        Self {
            code: -32010,
            message: err.to_string(),
            data: Some(json!({
                "errorCode": policy_code,
                "category": "policy"
            })),
        }
    }

    fn from_meta(msg: impl Into<String>) -> Self {
        Self {
            code: -32011,
            message: msg.into(),
            data: Some(json!({
                "errorCode": "ACP_META_VALIDATION_FAILED",
                "category": "policy"
            })),
        }
    }

    fn from_store(msg: impl Into<String>) -> Self {
        Self {
            code: -32020,
            message: msg.into(),
            data: Some(json!({
                "errorCode": "ACP_STATE_PERSISTENCE_FAILED",
                "category": "state"
            })),
        }
    }

    fn into_jsonrpc(self) -> JsonRpcError {
        JsonRpcError {
            code: self.code,
            message: self.message,
            data: self.data,
        }
    }
}

#[derive(Clone)]
pub struct AcpProtocolRuntime {
    adapter: AcpAdapter,
    session_store: AcpSessionStore,
    permission_ledger: AcpPermissionLedger,
    event_sink: AcpEventSink,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionNewParams {
    cwd: Option<String>,
    session_id: Option<String>,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionLoadParams {
    session_id: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionPromptParams {
    session_id: String,
    prompt: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionCancelParams {
    session_id: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionSetConfigOptionParams {
    session_id: String,
    config_id: String,
    value: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionRequestPermissionParams {
    session_id: String,
    tool_call_id: String,
    option_id: String,
    #[serde(default)]
    policy_key: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalCreateParams {
    session_id: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: Vec<crate::services::acp_adapter::EnvVariable>,
    cwd: Option<String>,
    output_byte_limit: Option<usize>,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalOutputParams {
    terminal_id: String,
    limit: Option<usize>,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalWaitForExitParams {
    terminal_id: String,
    timeout_ms: Option<u64>,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalKillParams {
    terminal_id: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalReleaseParams {
    terminal_id: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

impl AcpProtocolRuntime {
    pub fn load_default() -> Result<Self, String> {
        let workflow_root = FileSystemService::get_root_path();
        let cfg = AcpPolicyConfig::baseline(vec![workflow_root]);

        Ok(Self {
            adapter: AcpAdapter::new(cfg).map_err(|e| e.to_string())?,
            session_store: AcpSessionStore::load_default()?,
            permission_ledger: AcpPermissionLedger::load_default()?,
            event_sink: AcpEventSink::load_default(),
        })
    }

    async fn handle(&mut self, request: JsonRpcRequest) -> Result<Value, AcpRpcError> {
        if request.jsonrpc != "2.0" {
            return Err(AcpRpcError::invalid_request("jsonrpc must be '2.0'"));
        }

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "session/new" => self.handle_session_new(request.params).await,
            "session/load" => self.handle_session_load(request.params).await,
            "session/prompt" => self.handle_session_prompt(request.params).await,
            "session/cancel" => self.handle_session_cancel(request.params).await,
            "session/set_config_option" => {
                self.handle_session_set_config_option(request.params).await
            }
            "session/request_permission" => {
                self.handle_session_request_permission(request.params).await
            }
            "terminal/create" => self.handle_terminal_create(request.params).await,
            "terminal/output" => self.handle_terminal_output(request.params).await,
            "terminal/wait_for_exit" => self.handle_terminal_wait_for_exit(request.params).await,
            "terminal/kill" => self.handle_terminal_kill(request.params).await,
            "terminal/release" => self.handle_terminal_release(request.params).await,
            other => Err(AcpRpcError::method_not_found(other)),
        }
    }

    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        if let Some(value) = params.as_ref() {
            let _ = validate_meta(value.get("_meta"))
                .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        }

        Ok(json!({
            "protocolVersion": "0.1-pilot",
            "serverInfo": {
                "name": "cortex-desktop",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "session": [
                    "initialize",
                    "session/new",
                    "session/load",
                    "session/prompt",
                    "session/cancel",
                    "session/set_config_option",
                    "session/request_permission"
                ],
                "terminal": [
                    "terminal/create",
                    "terminal/output",
                    "terminal/wait_for_exit",
                    "terminal/kill",
                    "terminal/release"
                ]
            }
        }))
    }

    async fn handle_session_new(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: SessionNewParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let cwd = params
            .cwd
            .unwrap_or_else(|| FileSystemService::get_root_path().display().to_string());
        let canonical_cwd = self
            .adapter
            .validate_workspace_path(&cwd)
            .map_err(AcpRpcError::with_policy)?;

        let session_id = params
            .session_id
            .unwrap_or_else(|| format!("sess_{}", uuid::Uuid::new_v4().simple()));

        let record = self
            .session_store
            .create_session(session_id.clone(), canonical_cwd.display().to_string());
        self.session_store.save().map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": record.session_id,
            "cwd": record.cwd,
            "configOptions": self
                .session_store
                .config_options(&session_id)
                .map_err(AcpRpcError::from_store)?
        }))
    }

    async fn handle_session_load(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: SessionLoadParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let session = self
            .session_store
            .get_session(&params.session_id)
            .cloned()
            .ok_or_else(|| {
                AcpRpcError::invalid_params(format!("unknown session: {}", params.session_id))
            })?;

        let updates = self
            .session_store
            .replay_updates(&params.session_id)
            .map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": session.session_id,
            "cwd": session.cwd,
            "cancelled": session.cancelled,
            "configOptions": self
                .session_store
                .config_options(&params.session_id)
                .map_err(AcpRpcError::from_store)?,
            "updates": updates
        }))
    }

    async fn handle_session_prompt(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: SessionPromptParams = parse_params(params)?;
        let (trace, _) = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        if self.session_store.get_session(&params.session_id).is_none() {
            return Err(AcpRpcError::invalid_params(format!(
                "unknown session: {}",
                params.session_id
            )));
        }

        let turn_seq = self
            .session_store
            .start_turn(&params.session_id)
            .map_err(AcpRpcError::from_store)?;

        let user_update = self
            .append_update_and_project(
                &params.session_id,
                turn_seq,
                AcpSessionUpdateKind::UserMessageChunk,
                json!({ "text": params.prompt }),
                trace.clone(),
            )
            .await?;

        let mode_value = self
            .session_store
            .get_session(&params.session_id)
            .and_then(|s| s.config_options.get("mode").cloned())
            .unwrap_or_else(|| "ask".to_string());

        let agent_update = self
            .append_update_and_project(
                &params.session_id,
                turn_seq,
                AcpSessionUpdateKind::AgentMessageChunk,
                json!({
                    "text": "ACP pilot turn accepted",
                    "mode": mode_value,
                    "status": "completed"
                }),
                trace,
            )
            .await?;

        self.session_store.save().map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": params.session_id,
            "turnSeq": turn_seq,
            "stopReason": "completed",
            "updates": [user_update, agent_update]
        }))
    }

    async fn handle_session_cancel(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: SessionCancelParams = parse_params(params)?;
        let (trace, _) = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let turn_seq = self
            .session_store
            .get_session(&params.session_id)
            .map(|s| s.next_turn_seq.saturating_sub(1))
            .ok_or_else(|| {
                AcpRpcError::invalid_params(format!("unknown session: {}", params.session_id))
            })?;

        self.session_store
            .set_cancelled(&params.session_id, true)
            .map_err(AcpRpcError::from_store)?;

        let cancel_update = self
            .append_update_and_project(
                &params.session_id,
                turn_seq,
                AcpSessionUpdateKind::CurrentModeUpdate,
                json!({
                    "status": "cancelled",
                    "acceptsLateToolUpdates": true
                }),
                trace,
            )
            .await?;

        self.session_store.save().map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": params.session_id,
            "cancelled": true,
            "acceptsLateToolUpdates": true,
            "stopReason": "cancelled",
            "update": cancel_update
        }))
    }

    async fn handle_session_set_config_option(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: SessionSetConfigOptionParams = parse_params(params)?;
        let (trace, _) = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        self.session_store
            .set_config_option(&params.session_id, &params.config_id, &params.value)
            .map_err(AcpRpcError::from_store)?;

        let update = self
            .append_update_and_project(
                &params.session_id,
                0,
                AcpSessionUpdateKind::ConfigOptionUpdate,
                json!({
                    "configId": params.config_id,
                    "value": params.value
                }),
                trace,
            )
            .await?;

        self.session_store.save().map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": params.session_id,
            "configOptions": self
                .session_store
                .config_options(&params.session_id)
                .map_err(AcpRpcError::from_store)?,
            "update": update
        }))
    }

    async fn handle_session_request_permission(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: SessionRequestPermissionParams = parse_params(params)?;
        let (trace, _) = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        if self.session_store.get_session(&params.session_id).is_none() {
            return Err(AcpRpcError::invalid_params(format!(
                "unknown session: {}",
                params.session_id
            )));
        }

        let kind = match params.option_id.as_str() {
            "allow_once" => DecisionKind::AllowOnce,
            "allow_always" => DecisionKind::AllowAlways,
            "reject_once" => DecisionKind::RejectOnce,
            "reject_always" => DecisionKind::RejectAlways,
            other => {
                return Err(AcpRpcError::invalid_params(format!(
                    "unsupported permission option: {}",
                    other
                )));
            }
        };

        self.permission_ledger.record(
            params.session_id.clone(),
            params.tool_call_id.clone(),
            params.option_id.clone(),
            kind.clone(),
            params
                .source
                .clone()
                .unwrap_or_else(|| "acp_client".to_string()),
        );

        let policy_key = params.policy_key.unwrap_or_else(|| "default".to_string());
        let session_policy_applied =
            matches!(kind, DecisionKind::AllowAlways | DecisionKind::RejectAlways);
        if session_policy_applied {
            self.permission_ledger.set_session_policy(
                &params.session_id,
                &policy_key,
                kind.clone(),
            );
        }

        self.permission_ledger
            .save()
            .map_err(AcpRpcError::from_store)?;

        let turn_seq = self
            .session_store
            .get_session(&params.session_id)
            .map(|s| s.next_turn_seq.saturating_sub(1))
            .unwrap_or(0);

        let update = self
            .append_update_and_project(
                &params.session_id,
                turn_seq,
                AcpSessionUpdateKind::ToolCallUpdate,
                json!({
                    "toolCallId": params.tool_call_id,
                    "optionId": params.option_id,
                    "sessionPolicyApplied": session_policy_applied,
                    "policyKey": policy_key,
                }),
                trace,
            )
            .await?;

        self.session_store.save().map_err(AcpRpcError::from_store)?;

        Ok(json!({
            "sessionId": params.session_id,
            "toolCallId": params.tool_call_id,
            "decision": kind.as_str(),
            "sessionPolicyApplied": session_policy_applied,
            "update": update
        }))
    }

    async fn handle_terminal_create(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalCreateParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        if self.session_store.get_session(&params.session_id).is_none() {
            return Err(AcpRpcError::invalid_params(format!(
                "unknown session: {}",
                params.session_id
            )));
        }

        let request = TerminalCreateRequest {
            session_id: params.session_id,
            command: params.command,
            args: params.args,
            env: params.env,
            cwd: params.cwd,
            output_byte_limit: params.output_byte_limit,
        };

        let validated = self
            .adapter
            .validate_terminal_create(request)
            .map_err(AcpRpcError::with_policy)?;
        let result = TerminalService::acp_terminal_create(validated)
            .await
            .map_err(AcpRpcError::internal)?;

        serde_json::to_value(result).map_err(|e| AcpRpcError::internal(e.to_string()))
    }

    async fn handle_terminal_output(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalOutputParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let response = TerminalService::acp_terminal_output(AcpTerminalOutputRequest {
            terminal_id: params.terminal_id,
            limit: params.limit,
        })
        .await
        .map_err(AcpRpcError::internal)?;

        serde_json::to_value(response).map_err(|e| AcpRpcError::internal(e.to_string()))
    }

    async fn handle_terminal_wait_for_exit(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalWaitForExitParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let response = TerminalService::acp_terminal_wait_for_exit(AcpTerminalWaitRequest {
            terminal_id: params.terminal_id,
            timeout_ms: params.timeout_ms,
        })
        .await
        .map_err(AcpRpcError::internal)?;

        serde_json::to_value(response).map_err(|e| AcpRpcError::internal(e.to_string()))
    }

    async fn handle_terminal_kill(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: TerminalKillParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let response = TerminalService::acp_terminal_kill(params.terminal_id)
            .await
            .map_err(AcpRpcError::internal)?;
        serde_json::to_value(response).map_err(|e| AcpRpcError::internal(e.to_string()))
    }

    async fn handle_terminal_release(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalReleaseParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        let response = TerminalService::acp_terminal_release(params.terminal_id)
            .map_err(AcpRpcError::internal)?;
        serde_json::to_value(response).map_err(|e| AcpRpcError::internal(e.to_string()))
    }

    async fn append_update_and_project(
        &mut self,
        session_id: &str,
        turn_seq: u64,
        update_kind: AcpSessionUpdateKind,
        payload: Value,
        trace: TraceContext,
    ) -> Result<Value, AcpRpcError> {
        let update_seq = self
            .session_store
            .next_update_seq(session_id)
            .map_err(AcpRpcError::from_store)?;

        let projected = if is_cortex_runtime_v0_enabled() {
            #[cfg(feature = "cortex_runtime_v0")]
            {
                let runtime_projected = self
                    .event_sink
                    .record_event_runtime_v0(RuntimeAcpUpdateRequest {
                        session_id: session_id.to_string(),
                        turn_seq,
                        update_seq,
                        update_kind,
                        payload: payload.clone(),
                        trace: trace.clone(),
                    })
                    .await
                    .map_err(AcpRpcError::internal)?;

                if is_cortex_runtime_shadow_enabled() {
                    let legacy_projected = project_session_update_with_timestamp(
                        session_id,
                        turn_seq,
                        update_seq,
                        update_kind,
                        payload.clone(),
                        trace.clone(),
                        runtime_projected.timestamp,
                    );

                    if !projected_events_match(&legacy_projected, &runtime_projected) {
                        let reasons =
                            projected_event_mismatch_reasons(&legacy_projected, &runtime_projected);
                        tracing::warn!(
                            target: "cortex_runtime_shadow",
                            event = "cortex_runtime_shadow_mismatch",
                            session_id = %session_id,
                            turn_seq = turn_seq,
                            update_seq = update_seq,
                            mismatch_fields = %reasons.join(","),
                            "legacy and runtime projections diverged"
                        );
                    }
                }

                runtime_projected
            }
            #[cfg(not(feature = "cortex_runtime_v0"))]
            {
                return Err(AcpRpcError::internal(
                    "cortex_runtime_v0 requested but feature is not enabled",
                ));
            }
        } else {
            let projected = project_session_update(
                session_id,
                turn_seq,
                update_seq,
                update_kind,
                payload.clone(),
                trace,
            );

            self.event_sink
                .record_event(&projected)
                .await
                .map_err(AcpRpcError::internal)?;

            projected
        };

        let stored_update = StoredSessionUpdate {
            session_update: session_update_key(update_kind).to_string(),
            turn_seq,
            update_seq,
            timestamp: projected.timestamp,
            event_id: Some(projected.id),
            payload,
        };

        self.session_store
            .append_update(session_id, stored_update.clone())
            .map_err(AcpRpcError::from_store)?;

        serde_json::to_value(stored_update).map_err(|e| AcpRpcError::internal(e.to_string()))
    }
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

    let runtime = acp_runtime();
    let mut runtime_guard = runtime.lock().await;

    match runtime_guard.handle(request).await {
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

fn acp_runtime() -> Arc<TokioMutex<AcpProtocolRuntime>> {
    ACP_RUNTIME
        .get_or_init(|| {
            Arc::new(TokioMutex::new(
                AcpProtocolRuntime::load_default().expect("failed to initialize ACP runtime"),
            ))
        })
        .clone()
}

fn parse_params<T: DeserializeOwned>(params: Option<Value>) -> Result<T, AcpRpcError> {
    let raw = params.unwrap_or_else(|| json!({}));
    serde_json::from_value::<T>(raw).map_err(|e| AcpRpcError::invalid_params(e.to_string()))
}

fn session_update_key(kind: AcpSessionUpdateKind) -> &'static str {
    match kind {
        AcpSessionUpdateKind::UserMessageChunk => "user_message_chunk",
        AcpSessionUpdateKind::AgentMessageChunk => "agent_message_chunk",
        AcpSessionUpdateKind::AgentThoughtChunk => "agent_thought_chunk",
        AcpSessionUpdateKind::ToolCall => "tool_call",
        AcpSessionUpdateKind::ToolCallUpdate => "tool_call_update",
        AcpSessionUpdateKind::Plan => "plan",
        AcpSessionUpdateKind::AvailableCommandsUpdate => "available_commands_update",
        AcpSessionUpdateKind::CurrentModeUpdate => "current_mode_update",
        AcpSessionUpdateKind::ConfigOptionUpdate => "config_option_update",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn test_runtime() -> (AcpProtocolRuntime, std::path::PathBuf) {
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

        (
            AcpProtocolRuntime {
                adapter: AcpAdapter::new(cfg).unwrap(),
                session_store: AcpSessionStore::load(session_path).unwrap(),
                permission_ledger: AcpPermissionLedger::load(ledger_path).unwrap(),
                event_sink: AcpEventSink::with_paths(events_path, None),
            },
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
    async fn meta_validation_rejects_non_namespaced_key() {
        let (mut runtime, _) = test_runtime();
        let err = runtime
            .handle(rpc("initialize", json!({ "_meta": { "foo": "bar" } })))
            .await
            .unwrap_err();
        assert_eq!(err.code, -32011);
        assert_eq!(err.data.unwrap()["errorCode"], "ACP_META_VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn rpc_handler_returns_pilot_disabled_when_flag_is_off() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
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
        right.timestamp = right.timestamp + 1;

        assert!(
            projected_events_match(&left, &right),
            "timestamp-only drift should remain allowed in shadow parity"
        );
    }
}
