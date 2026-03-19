use crate::memory_fs::ContextFs;
use crate::policy::adapter::{
    AcpPolicyError, AcpSessionUpdateKind, OperationAdapter, TerminalCreateRequest,
    ValidatedTerminalCreate,
};
use crate::policy::permissions::DecisionKind;
use crate::policy::sessions::{SessionRecord, StoredSessionUpdate};
use async_trait::async_trait;
use cortex_domain::memory_fs::{Oid, Tree};
use cortex_domain::policy::meta::{TraceContext, validate_meta};
use cortex_domain::policy::types::{JsonRpcError, JsonRpcRequest};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug)]
pub struct AcpRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl AcpRpcError {
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {}", method),
            data: None,
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: -32000,
            message: msg.into(),
            data: None,
        }
    }

    pub fn pilot_disabled() -> Self {
        Self {
            code: -32030,
            message: "ACP pilot is disabled".to_string(),
            data: Some(json!({
                "errorCode": "ACP_PILOT_DISABLED",
                "category": "policy"
            })),
        }
    }

    pub fn from_meta(msg: impl Into<String>) -> Self {
        Self {
            code: -32011,
            message: msg.into(),
            data: Some(json!({
                "errorCode": "ACP_META_VALIDATION_FAILED",
                "category": "policy"
            })),
        }
    }

    pub fn from_store(msg: impl Into<String>) -> Self {
        Self {
            code: -32020,
            message: msg.into(),
            data: Some(json!({
                "errorCode": "ACP_STATE_PERSISTENCE_FAILED",
                "category": "state"
            })),
        }
    }

    pub fn with_policy(err: AcpPolicyError) -> Self {
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

    pub fn into_jsonrpc(self) -> JsonRpcError {
        JsonRpcError {
            code: self.code,
            message: self.message,
            data: self.data,
        }
    }
}

pub fn parse_params<T: DeserializeOwned>(params: Option<Value>) -> Result<T, AcpRpcError> {
    let raw = params.unwrap_or_else(|| json!({}));
    serde_json::from_value::<T>(raw).map_err(|e| AcpRpcError::invalid_params(e.to_string()))
}

pub fn session_update_key(kind: AcpSessionUpdateKind) -> &'static str {
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
        AcpSessionUpdateKind::AgentBranching => "agent_branching",
        AcpSessionUpdateKind::ToolCallResult => "tool_call_result",
    }
}

pub trait SessionStorePort {
    fn create_session(&mut self, session_id: String, cwd: String) -> SessionRecord;
    fn get_session(&self, session_id: &str) -> Option<SessionRecord>;
    fn start_turn(&mut self, session_id: &str) -> Result<u64, String>;
    fn next_update_seq(&mut self, session_id: &str) -> Result<u64, String>;
    fn append_update(
        &mut self,
        session_id: &str,
        update: StoredSessionUpdate,
    ) -> Result<(), String>;
    fn set_cancelled(&mut self, session_id: &str, cancelled: bool) -> Result<(), String>;
    fn set_config_option(
        &mut self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> Result<(), String>;
    fn config_options(&self, session_id: &str) -> Result<Vec<Value>, String>;
    fn replay_updates(&self, session_id: &str) -> Result<Vec<StoredSessionUpdate>, String>;
    fn save(&self) -> Result<(), String>;
}

pub trait PermissionLedgerPort {
    fn record(
        &mut self,
        session_id: String,
        tool_call_id: String,
        option_id: String,
        kind: DecisionKind,
        source: String,
    );
    fn set_session_policy(&mut self, session_id: &str, policy_key: &str, kind: DecisionKind);
    fn save(&self) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct ProjectUpdateRequest {
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub update_kind: AcpSessionUpdateKind,
    pub payload: Value,
    pub trace: TraceContext,
}

#[derive(Debug, Clone)]
pub struct ProjectUpdateResult {
    pub timestamp: u64,
    pub event_id: String,
}

#[async_trait]
pub trait AcpProtocolHost {
    fn server_name(&self) -> &'static str;
    fn server_version(&self) -> String;
    fn default_workspace_path(&self) -> String;

    async fn project_update(
        &mut self,
        request: ProjectUpdateRequest,
    ) -> Result<ProjectUpdateResult, String>;

    async fn terminal_create(&mut self, request: ValidatedTerminalCreate) -> Result<Value, String>;

    async fn terminal_output(
        &mut self,
        terminal_id: String,
        limit: Option<usize>,
    ) -> Result<Value, String>;

    async fn terminal_wait_for_exit(
        &mut self,
        terminal_id: String,
        timeout_ms: Option<u64>,
    ) -> Result<Value, String>;

    async fn terminal_kill(&mut self, terminal_id: String) -> Result<Value, String>;

    async fn terminal_release(&mut self, terminal_id: String) -> Result<Value, String>;

    async fn on_memory_fs_commit(
        &mut self,
        _session_id: &str,
        _branch: &str,
        _commit_oid: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub struct AcpProtocolRuntime<A, S, P, H> {
    adapter: A,
    session_store: S,
    permission_ledger: P,
    host: H,
    context_fs: Option<Arc<ContextFs>>,
}

impl<A, S, P, H> AcpProtocolRuntime<A, S, P, H>
where
    A: OperationAdapter + Send,
    S: SessionStorePort + Send,
    P: PermissionLedgerPort + Send,
    H: AcpProtocolHost + Send,
{
    pub fn new(
        adapter: A,
        session_store: S,
        permission_ledger: P,
        host: H,
        context_fs: Option<Arc<ContextFs>>,
    ) -> Self {
        Self {
            adapter,
            session_store,
            permission_ledger,
            host,
            context_fs,
        }
    }

    pub async fn handle(&mut self, request: JsonRpcRequest) -> Result<Value, AcpRpcError> {
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
            "memory_fs/write_blob" => self.handle_memory_fs_write_blob(request.params).await,
            "memory_fs/read_blob" => self.handle_memory_fs_read_blob(request.params).await,
            "memory_fs/write_tree" => self.handle_memory_fs_write_tree(request.params).await,
            "memory_fs/read_tree" => self.handle_memory_fs_read_tree(request.params).await,
            "memory_fs/commit" => self.handle_memory_fs_commit(request.params).await,
            "memory_fs/branch_create" => self.handle_memory_fs_branch_create(request.params).await,
            "memory_fs/branch_resolve" => {
                self.handle_memory_fs_branch_resolve(request.params).await
            }
            other => Err(AcpRpcError::method_not_found(other)),
        }
    }

    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        if let Some(value) = params.as_ref() {
            let _ = validate_meta(value.get("_meta"))
                .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        }
        let server_name = self.host.server_name();
        let server_version = self.host.server_version();

        Ok(json!({
            "protocolVersion": "0.1-pilot",
            "serverInfo": {
                "name": server_name,
                "version": server_version
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
                ],
                "memory_fs": [
                    "memory_fs/write_blob",
                    "memory_fs/read_blob",
                    "memory_fs/write_tree",
                    "memory_fs/read_tree",
                    "memory_fs/commit",
                    "memory_fs/branch_create",
                    "memory_fs/branch_resolve"
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
            .unwrap_or_else(|| self.host.default_workspace_path());

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

        self.host
            .terminal_create(validated)
            .await
            .map_err(AcpRpcError::internal)
    }

    async fn handle_terminal_output(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalOutputParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        self.host
            .terminal_output(params.terminal_id, params.limit)
            .await
            .map_err(AcpRpcError::internal)
    }

    async fn handle_terminal_wait_for_exit(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalWaitForExitParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        self.host
            .terminal_wait_for_exit(params.terminal_id, params.timeout_ms)
            .await
            .map_err(AcpRpcError::internal)
    }

    async fn handle_terminal_kill(&mut self, params: Option<Value>) -> Result<Value, AcpRpcError> {
        let params: TerminalKillParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        self.host
            .terminal_kill(params.terminal_id)
            .await
            .map_err(AcpRpcError::internal)
    }

    async fn handle_terminal_release(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: TerminalReleaseParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;

        self.host
            .terminal_release(params.terminal_id)
            .await
            .map_err(AcpRpcError::internal)
    }

    fn require_context_fs(&self) -> Result<&ContextFs, AcpRpcError> {
        self.context_fs
            .as_deref()
            .ok_or_else(|| AcpRpcError::internal("ContextFs is not configured for this runtime"))
    }

    async fn handle_memory_fs_write_blob(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsWriteBlobParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let content_bytes = params.content.into_bytes();
        let oid = fs
            .write_blob(content_bytes)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        Ok(json!({ "oid": oid.as_str() }))
    }

    async fn handle_memory_fs_read_blob(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsReadBlobParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let oid = Oid::new(params.oid);
        let blob = fs
            .read_blob(&oid)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        let content_str = String::from_utf8(blob.content)
            .map_err(|_| AcpRpcError::internal("Blob content is not valid UTF-8"))?;
        Ok(json!({ "content": content_str }))
    }

    async fn handle_memory_fs_write_tree(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsWriteTreeParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let oid = fs
            .write_tree(&params.tree)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        Ok(json!({ "oid": oid.as_str() }))
    }

    async fn handle_memory_fs_read_tree(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsReadTreeParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let oid = Oid::new(params.oid);
        let tree = fs
            .read_tree(&oid)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        let val = serde_json::to_value(&tree).map_err(|e| AcpRpcError::internal(e.to_string()))?;
        Ok(val)
    }

    async fn handle_memory_fs_commit(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsCommitParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let tree_oid = Oid::new(params.tree_oid);
        let oid = fs
            .commit(&params.branch, tree_oid, params.author, params.message)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;

        let session_id = params
            .meta
            .as_ref()
            .and_then(|m| m.get("session_id"))
            .and_then(|s| s.as_str())
            .unwrap_or("default")
            .to_string();
        let _ = self
            .host
            .on_memory_fs_commit(&session_id, &params.branch, oid.as_str())
            .await;

        Ok(json!({ "oid": oid.as_str() }))
    }

    async fn handle_memory_fs_branch_create(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsBranchCreateParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let commit_oid = Oid::new(params.commit_oid);
        fs.create_branch(&params.branch, commit_oid)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        Ok(json!({ "success": true }))
    }

    async fn handle_memory_fs_branch_resolve(
        &mut self,
        params: Option<Value>,
    ) -> Result<Value, AcpRpcError> {
        let params: MemFsBranchResolveParams = parse_params(params)?;
        let _ = validate_meta(params.meta.as_ref())
            .map_err(|e| AcpRpcError::from_meta(e.to_string()))?;
        let fs = self.require_context_fs()?;
        let oid = fs
            .resolve_branch(&params.branch)
            .await
            .map_err(|e| AcpRpcError::internal(e.to_string()))?;
        Ok(json!({ "oid": oid.as_str() }))
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

        let projected = self
            .host
            .project_update(ProjectUpdateRequest {
                session_id: session_id.to_string(),
                turn_seq,
                update_seq,
                update_kind,
                payload: payload.clone(),
                trace,
            })
            .await
            .map_err(AcpRpcError::internal)?;

        if let Some(fs) = &self.context_fs {
            let parent_commit = fs.resolve_branch(session_id).await.ok();
            let mut tree = match &parent_commit {
                Some(oid) => {
                    if let Ok(c) = fs.read_commit(oid).await {
                        fs.read_tree(&c.tree).await.unwrap_or_else(|_| Tree::new())
                    } else {
                        Tree::new()
                    }
                }
                None => Tree::new(),
            };

            let mut log_content = String::new();
            if let Some(cortex_domain::memory_fs::TreeEntry::Blob(blob_oid)) =
                tree.entries.get("log.md")
            {
                if let Ok(blob) = fs.read_blob(blob_oid).await {
                    if let Ok(s) = String::from_utf8(blob.content.clone()) {
                        log_content = s;
                    }
                }
            }

            let event_str = format!(
                "turn: {}, seq: {}, kind: {:?}, payload: {}\n",
                turn_seq,
                update_seq,
                update_kind,
                serde_json::to_string(&payload).unwrap_or_default()
            );
            log_content.push_str(&event_str);

            if let Ok(blob_oid) = fs.write_blob(log_content.into_bytes()).await {
                tree.entries.insert(
                    "log.md".to_string(),
                    cortex_domain::memory_fs::TreeEntry::Blob(blob_oid),
                );
                if let Ok(tree_oid) = fs.write_tree(&tree).await {
                    let _ = fs
                        .commit(
                            session_id,
                            tree_oid,
                            "system".to_string(),
                            format!("Append session update {:?}", update_kind),
                        )
                        .await;
                }
            }
        }

        let stored_update = StoredSessionUpdate {
            session_update: session_update_key(update_kind).to_string(),
            turn_seq,
            update_seq,
            timestamp: projected.timestamp,
            event_id: Some(projected.event_id),
            payload,
        };

        self.session_store
            .append_update(session_id, stored_update.clone())
            .map_err(AcpRpcError::from_store)?;

        serde_json::to_value(stored_update).map_err(|e| AcpRpcError::internal(e.to_string()))
    }
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
    env: Vec<crate::policy::adapter::EnvVariable>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsWriteBlobParams {
    content: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsReadBlobParams {
    oid: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsWriteTreeParams {
    tree: Tree,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsReadTreeParams {
    oid: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsCommitParams {
    branch: String,
    tree_oid: String,
    author: String,
    message: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsBranchCreateParams {
    branch: String,
    commit_oid: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemFsBranchResolveParams {
    branch: String,
    #[serde(default, rename = "_meta")]
    meta: Option<Value>,
}
