use crate::services::workflow_engine_client::WorkflowEngineClient;
use async_trait::async_trait;
use chrono::Utc;
use cortex_runtime::{RuntimeError, ports::UxContractStoreAdapter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

const REPLAY_QUEUE_FILE: &str = "_runtime/replay_queue.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexSyncStatus {
    pub schema_version: String,
    pub generated_at: String,
    pub mode: String,
    pub primary_available: bool,
    pub fallback_active: bool,
    pub pending_replay: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_replay_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CortexReplayResult {
    pub schema_version: String,
    pub replayed_at: String,
    pub attempted: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub pending_after: u64,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failed_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoreReadOutcome {
    pub text: Option<String>,
    pub source_of_truth: String,
    pub fallback_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoreWriteOutcome {
    pub source_of_truth: String,
    pub fallback_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ReplayOperation {
    id: String,
    kind: String,
    key: String,
    content: String,
    mime_type: String,
    enqueued_at: String,
    attempt_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
}

#[derive(Debug, Default)]
struct RuntimeSyncState {
    fallback_active: bool,
    last_error: Option<String>,
    last_replay_at: Option<String>,
}

#[async_trait]
pub trait CortexUxStore: Send + Sync {
    async fn read_text(&self, key: &str) -> Result<Option<String>, String>;
    async fn write_text(&self, key: &str, content: &str, mime_type: &str) -> Result<(), String>;
    async fn append_line(&self, key: &str, line: &str, mime_type: &str) -> Result<(), String>;
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, String>;
}

#[derive(Clone, Debug)]
pub struct LocalMirrorStore {
    root: PathBuf,
}

impl LocalMirrorStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        let normalized = key.trim_start_matches('/');
        self.root.join(normalized)
    }

    fn read_queue(&self) -> Vec<ReplayOperation> {
        let path = self.path_for(REPLAY_QUEUE_FILE);
        match fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str::<Vec<ReplayOperation>>(&raw).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    fn write_queue(&self, queue: &[ReplayOperation]) -> Result<(), String> {
        let path = self.path_for(REPLAY_QUEUE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let encoded = serde_json::to_string_pretty(queue).map_err(|err| err.to_string())?;
        fs::write(path, encoded).map_err(|err| err.to_string())
    }
}

#[async_trait]
impl CortexUxStore for LocalMirrorStore {
    async fn read_text(&self, key: &str) -> Result<Option<String>, String> {
        let path = self.path_for(key);
        match fs::read_to_string(path) {
            Ok(raw) => Ok(Some(raw)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn write_text(&self, key: &str, content: &str, _mime_type: &str) -> Result<(), String> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::write(path, content).map_err(|err| err.to_string())
    }

    async fn append_line(&self, key: &str, line: &str, mime_type: &str) -> Result<(), String> {
        let mut base = self.read_text(key).await?.unwrap_or_default();
        if !base.is_empty() && !base.ends_with('\n') {
            base.push('\n');
        }
        base.push_str(line);
        if !base.ends_with('\n') {
            base.push('\n');
        }
        self.write_text(key, &base, mime_type).await
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, String> {
        let root = self.path_for(prefix);
        if !root.exists() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let Ok(relative) = path.strip_prefix(&self.root) else {
                    continue;
                };
                let normalized = format!("/{}", relative.display().to_string().replace('\\', "/"));
                items.push(normalized);
            }
        }
        items.sort();
        Ok(items)
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowEngineVfsStore {
    client: Arc<Mutex<Option<Option<WorkflowEngineClient>>>>,
}

impl WorkflowEngineVfsStore {
    pub fn new() -> Self {
        Self { client: Arc::new(Mutex::new(None)) }
    }

    async fn get_client(&self) -> Option<WorkflowEngineClient> {
        let mut lock = self.client.lock().await;
        if let Some(res) = &*lock {
            return res.clone();
        }
        let client = WorkflowEngineClient::from_env().await.ok();
        *lock = Some(client.clone());
        client
    }

    async fn is_enabled(&self) -> bool {
        self.get_client().await.is_some() && std::env::var("CANISTER_ID_WORKFLOW_ENGINE").is_ok()
    }
}

#[async_trait]
impl CortexUxStore for WorkflowEngineVfsStore {
    async fn read_text(&self, key: &str) -> Result<Option<String>, String> {
        if !self.is_enabled().await {
            return Err("workflow engine VFS is not configured".to_string());
        }
        let Some(client) = self.get_client().await else {
            return Err("workflow engine client unavailable".to_string());
        };
        match client.read_file(key).await {
            Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            Err(err) if err.to_ascii_lowercase().contains("not found") => Ok(None),
            Err(err) => Err(err),
        }
    }

    async fn write_text(&self, key: &str, content: &str, mime_type: &str) -> Result<(), String> {
        if !self.is_enabled().await {
            return Err("workflow engine VFS is not configured".to_string());
        }
        let Some(client) = self.get_client().await else {
            return Err("workflow engine client unavailable".to_string());
        };
        client
            .write_file(key, content.as_bytes().to_vec(), mime_type)
            .await
    }

    async fn append_line(&self, key: &str, line: &str, mime_type: &str) -> Result<(), String> {
        let mut base = self.read_text(key).await?.unwrap_or_default();
        if !base.is_empty() && !base.ends_with('\n') {
            base.push('\n');
        }
        base.push_str(line);
        if !base.ends_with('\n') {
            base.push('\n');
        }
        self.write_text(key, &base, mime_type).await
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, String> {
        if !self.is_enabled().await {
            return Err("workflow engine VFS is not configured".to_string());
        }
        let Some(client) = self.get_client().await else {
            return Err("workflow engine client unavailable".to_string());
        };
        client
            .list_files(prefix)
            .await
            .map(|entries| entries.into_iter().map(|(path, _)| path).collect())
    }
}

#[derive(Clone)]
pub struct CortexUxStoreManager {
    primary: WorkflowEngineVfsStore,
    mirror: LocalMirrorStore,
    state: Arc<Mutex<RuntimeSyncState>>,
}

impl CortexUxStoreManager {
    pub fn new(local_root: PathBuf) -> Self {
        Self {
            primary: WorkflowEngineVfsStore::new(),
            mirror: LocalMirrorStore::new(local_root),
            state: Arc::new(Mutex::new(RuntimeSyncState::default())),
        }
    }

    pub async fn read_text(&self, key: &str) -> Result<StoreReadOutcome, String> {
        match self.primary.read_text(key).await {
            Ok(text) => {
                let mut state = self.state.lock().await;
                state.fallback_active = false;
                Ok(StoreReadOutcome {
                    text,
                    source_of_truth: "workflow_engine_vfs".to_string(),
                    fallback_active: false,
                    degraded_reason: None,
                })
            }
            Err(primary_err) => {
                let text = self.mirror.read_text(key).await?;
                let mut state = self.state.lock().await;
                state.fallback_active = true;
                state.last_error = Some(primary_err.clone());
                Ok(StoreReadOutcome {
                    text,
                    source_of_truth: "local_mirror".to_string(),
                    fallback_active: true,
                    degraded_reason: Some(primary_err),
                })
            }
        }
    }

    pub async fn write_text(
        &self,
        key: &str,
        content: &str,
        mime_type: &str,
    ) -> Result<StoreWriteOutcome, String> {
        match self.primary.write_text(key, content, mime_type).await {
            Ok(()) => {
                let _ = self.mirror.write_text(key, content, mime_type).await;
                let mut state = self.state.lock().await;
                state.fallback_active = false;
                drop(state);
                let _ = self.replay_pending().await;
                Ok(StoreWriteOutcome {
                    source_of_truth: "workflow_engine_vfs".to_string(),
                    fallback_active: false,
                    degraded_reason: None,
                })
            }
            Err(primary_err) => {
                self.mirror.write_text(key, content, mime_type).await?;
                self.enqueue_replay("write", key, content, mime_type, Some(primary_err.clone()))
                    .await?;
                let mut state = self.state.lock().await;
                state.fallback_active = true;
                state.last_error = Some(primary_err.clone());
                Ok(StoreWriteOutcome {
                    source_of_truth: "local_mirror".to_string(),
                    fallback_active: true,
                    degraded_reason: Some(primary_err),
                })
            }
        }
    }

    pub async fn append_line(
        &self,
        key: &str,
        line: &str,
        mime_type: &str,
    ) -> Result<StoreWriteOutcome, String> {
        match self.primary.append_line(key, line, mime_type).await {
            Ok(()) => {
                let _ = self.mirror.append_line(key, line, mime_type).await;
                let mut state = self.state.lock().await;
                state.fallback_active = false;
                drop(state);
                let _ = self.replay_pending().await;
                Ok(StoreWriteOutcome {
                    source_of_truth: "workflow_engine_vfs".to_string(),
                    fallback_active: false,
                    degraded_reason: None,
                })
            }
            Err(primary_err) => {
                self.mirror.append_line(key, line, mime_type).await?;
                self.enqueue_replay("append", key, line, mime_type, Some(primary_err.clone()))
                    .await?;
                let mut state = self.state.lock().await;
                state.fallback_active = true;
                state.last_error = Some(primary_err.clone());
                Ok(StoreWriteOutcome {
                    source_of_truth: "local_mirror".to_string(),
                    fallback_active: true,
                    degraded_reason: Some(primary_err),
                })
            }
        }
    }

    pub async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, String> {
        match self.primary.list_prefix(prefix).await {
            Ok(items) => {
                let mut state = self.state.lock().await;
                state.fallback_active = false;
                Ok(items)
            }
            Err(primary_err) => {
                let items = self.mirror.list_prefix(prefix).await?;
                let mut state = self.state.lock().await;
                state.fallback_active = true;
                state.last_error = Some(primary_err);
                Ok(items)
            }
        }
    }

    pub async fn replay_pending(&self) -> Result<CortexReplayResult, String> {
        let mut queue = self.mirror.read_queue();
        let attempted = queue.len() as u64;
        let mut succeeded = 0u64;
        let mut failed = 0u64;
        let mut failed_keys = Vec::new();
        let mut next_queue = Vec::new();

        for mut op in queue.drain(..) {
            let replay_result = if op.kind == "append" {
                self.primary
                    .append_line(&op.key, &op.content, &op.mime_type)
                    .await
            } else {
                self.primary
                    .write_text(&op.key, &op.content, &op.mime_type)
                    .await
            };

            match replay_result {
                Ok(()) => {
                    succeeded = succeeded.saturating_add(1);
                }
                Err(err) => {
                    failed = failed.saturating_add(1);
                    failed_keys.push(op.key.clone());
                    op.attempt_count = op.attempt_count.saturating_add(1);
                    op.last_error = Some(err);
                    next_queue.push(op);
                }
            }
        }

        self.mirror.write_queue(&next_queue)?;
        let mut state = self.state.lock().await;
        state.last_replay_at = Some(Utc::now().to_rfc3339());
        state.fallback_active = !next_queue.is_empty();
        if let Some(first) = next_queue.first() {
            state.last_error = first.last_error.clone();
        } else {
            state.last_error = None;
        }

        Ok(CortexReplayResult {
            schema_version: "1.0.0".to_string(),
            replayed_at: Utc::now().to_rfc3339(),
            attempted,
            succeeded,
            failed,
            pending_after: next_queue.len() as u64,
            failed_keys,
        })
    }

    pub async fn sync_status(&self) -> CortexSyncStatus {
        let queue = self.mirror.read_queue();
        let primary_available = self.primary.list_prefix("/cortex/ux").await.is_ok();
        let state = self.state.lock().await;
        CortexSyncStatus {
            schema_version: "1.0.0".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            mode: if primary_available {
                "workflow_engine_vfs_primary".to_string()
            } else {
                "local_mirror_fallback".to_string()
            },
            primary_available,
            fallback_active: state.fallback_active || !queue.is_empty(),
            pending_replay: queue.len() as u64,
            last_replay_at: state.last_replay_at.clone(),
            last_error: state.last_error.clone(),
        }
    }

    async fn enqueue_replay(
        &self,
        kind: &str,
        key: &str,
        content: &str,
        mime_type: &str,
        reason: Option<String>,
    ) -> Result<(), String> {
        let mut queue = self.mirror.read_queue();
        queue.push(ReplayOperation {
            id: format!("replay_{}", Utc::now().timestamp_millis()),
            kind: kind.to_string(),
            key: key.to_string(),
            content: content.to_string(),
            mime_type: mime_type.to_string(),
            enqueued_at: Utc::now().to_rfc3339(),
            attempt_count: 0,
            last_error: reason,
        });
        self.mirror.write_queue(&queue)
    }
}

fn cortex_ux_local_root() -> PathBuf {
    std::env::var("NOSTRA_CORTEX_UX_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CORTEX_IC_PROJECT_ROOT")
                .map(|root| PathBuf::from(root).join("logs/cortex/ux"))
        })
        .unwrap_or_else(|_| PathBuf::from("logs/cortex/ux"))
}

fn global_manager() -> &'static Arc<CortexUxStoreManager> {
    static STORE: OnceLock<Arc<CortexUxStoreManager>> = OnceLock::new();
    STORE.get_or_init(|| Arc::new(CortexUxStoreManager::new(cortex_ux_local_root())))
}

pub fn cortex_ux_store_manager() -> Arc<CortexUxStoreManager> {
    global_manager().clone()
}

pub fn is_cortex_ux_local_path(path: &Path) -> bool {
    path.starts_with(cortex_ux_local_root())
}

pub fn to_cortex_vfs_key(local_path: &Path) -> Option<String> {
    let root = cortex_ux_local_root();
    let relative = local_path.strip_prefix(&root).ok()?;
    let normalized = relative.display().to_string().replace('\\', "/");

    match normalized.as_str() {
        "cortex_ux_contract_v1.json" => Some("/cortex/ux/layout/current.json".to_string()),
        "feedback_events.jsonl" => Some("/cortex/ux/feedback/events.jsonl".to_string()),
        "feedback_queue.json" => Some("/cortex/ux/feedback/queue.json".to_string()),
        "feedback_lifecycle_events.jsonl" => {
            Some("/cortex/ux/feedback/lifecycle.jsonl".to_string())
        }
        "feedback_remeasurements.json" => {
            Some("/cortex/ux/feedback/remeasurements.json".to_string())
        }
        "candidate_evaluations.jsonl" => Some("/cortex/ux/feedback/evaluations.jsonl".to_string()),
        "promotion_decisions.jsonl" => Some("/cortex/ux/promotions/decisions.jsonl".to_string()),
        "artifacts_store.json" => Some("/cortex/ux/artifacts/store.json".to_string()),
        "artifacts_revisions.json" => Some("/cortex/ux/artifacts/revisions.json".to_string()),
        "artifacts_leases.json" => Some("/cortex/ux/artifacts/leases.json".to_string()),
        "artifact_audit_events.jsonl" => Some("/cortex/ux/artifacts/audit.jsonl".to_string()),
        "artifacts_collab_sessions.json" => {
            Some("/cortex/ux/artifacts/collab_sessions.json".to_string())
        }
        "artifacts_collab_ops.json" => Some("/cortex/ux/artifacts/collab_ops.json".to_string()),
        _ => {
            if normalized.starts_with("artifacts/crdt/") {
                Some(format!("/cortex/ux/{}", normalized))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct DesktopUxContractStoreAdapter;

#[async_trait]
impl UxContractStoreAdapter for DesktopUxContractStoreAdapter {
    async fn read_text(&self, key: &str) -> Result<Option<String>, RuntimeError> {
        cortex_ux_store_manager()
            .read_text(key)
            .await
            .map(|outcome| outcome.text)
            .map_err(RuntimeError::Storage)
    }

    async fn write_text(
        &self,
        key: &str,
        content: &str,
        mime_type: &str,
    ) -> Result<(), RuntimeError> {
        cortex_ux_store_manager()
            .write_text(key, content, mime_type)
            .await
            .map(|_| ())
            .map_err(RuntimeError::Storage)
    }

    async fn append_line(
        &self,
        key: &str,
        line: &str,
        mime_type: &str,
    ) -> Result<(), RuntimeError> {
        cortex_ux_store_manager()
            .append_line(key, line, mime_type)
            .await
            .map(|_| ())
            .map_err(RuntimeError::Storage)
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, RuntimeError> {
        cortex_ux_store_manager()
            .list_prefix(prefix)
            .await
            .map_err(RuntimeError::Storage)
    }
}
