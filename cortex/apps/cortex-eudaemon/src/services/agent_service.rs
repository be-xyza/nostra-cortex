use cortex_runtime::agents::service as runtime_agents;
use serde::{Deserialize, Serialize};

#[cfg(feature = "service-scaffolds")]
use std::sync::OnceLock;

#[cfg(feature = "service-scaffolds")]
use tokio::sync::broadcast;

#[cfg(feature = "service-scaffolds")]
use cortex_runtime::{RuntimeError, ports::AgentProcessAdapter};

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: String,
    pub status: String,
    pub children: Vec<String>,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSystemState {
    pub nodes: Vec<AgentNode>,
}

pub struct AgentService;

#[cfg(feature = "service-scaffolds")]
static STATE_CHANNEL: OnceLock<broadcast::Sender<AgentSystemState>> = OnceLock::new();

#[cfg(feature = "service-scaffolds")]
pub fn get_agent_state_tx() -> broadcast::Sender<AgentSystemState> {
    STATE_CHANNEL
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        })
        .clone()
}

impl AgentService {
    pub async fn spawn_vector_agent() {
        tracing::info!("Spawning Vector Agent (Elna Interface)...");
        // In a real scenario, this would check if 'elna' canister is running
        // and start a background task to index notes.
        tokio::spawn(async move {
            tracing::info!("[VectorAgent] Connecting to Elna Knowledge Store...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Mock connection success/fail based on dfx
            let output = std::process::Command::new("dfx")
                .arg("canister")
                .arg("status")
                .arg("elna")
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    tracing::info!("[VectorAgent] Connected to 'elna' canister.");
                    tracing::info!("[VectorAgent] Index status: 0 documents (Cold Start)");
                }
                _ => {
                    tracing::warn!(
                        "[VectorAgent] Warning: 'elna' canister not found. Local knowledge store is offline."
                    );
                }
            }
        });
    }

    #[cfg(feature = "service-scaffolds")]
    pub async fn execute_ic_script(script: String) -> Result<String, String> {
        // Debug Commands for Simulation Testing
        if script == "debug:offline" {
            crate::gateway::runtime_host::set_local_gateway_online(false);
            return Ok("Gateway set to OFFLINE".to_string());
        }
        if script == "debug:online" {
            crate::gateway::runtime_host::set_local_gateway_online(true);
            return Ok("Gateway set to ONLINE".to_string());
        }

        // Intercept for Offline Mode
        if !crate::gateway::runtime_host::local_gateway_is_online() {
            // Create Mutation
            let mutation = crate::services::local_gateway::Mutation::new(script.clone());
            let _ = crate::gateway::runtime_host::submit_local_gateway_mutation(mutation);
            // Return "Queued" to UI so it can handle Optimistic UI
            return Ok("Status: Queued (Offline)".to_string());
        }

        let client = crate::services::dfx_client::DfxClient::new(None);

        // Candid quoting for text: "foo" -> '("foo")'
        let arg = format!("(\"{}\")", script.replace("\"", "\\\""));

        match client
            .call_canister("workflow-engine", "execute_script", Some(&arg))
            .await
        {
            Ok(output) => {
                // Parse candid record output: (record { output = "..."; ... })
                if let Some(start) = output.find("output = \"") {
                    let rest = &output[start + 10..];
                    if let Some(end) = rest.find("\";") {
                        return Ok(rest[..end].to_string());
                    }
                }
                Ok(output) // Fallback to raw output
            }
            Err(e) => Err(e.to_string()),
        }
    }

    #[cfg(feature = "service-scaffolds")]
    pub async fn send_chat_message(
        mut message: String,
        space_id: Option<String>,
    ) -> Result<(), String> {
        let client = crate::services::dfx_client::DfxClient::new(None);

        if let Some(space) = space_id {
            message = format!(
                "Context: Operating within Space ID '{}'.\nUser Query: {}",
                space, message
            );
        }

        let arg = format!("(\"{}\")", message.replace("\"", "\\\""));

        tracing::info!("[Console] Sending: {}", message);

        match client
            .call_canister("workflow-engine", "process_message", Some(&arg))
            .await
        {
            Ok(response) => {
                tracing::info!("[Console] Received: {}", response);

                let decoded =
                    crate::services::dfx_client::DfxClient::unwrap_candid_string(&response)
                        .unwrap_or(response);

                if !decoded.trim().is_empty() {
                    // Send to the console backend instead of local UI
                    crate::services::console_service::send_text("System", decoded);
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("[Console] Error: {}", e);
                Err(e.to_string())
            }
        }
    }

    pub async fn index(id: String, content: String, modality: Modality) {
        tracing::info!(
            "[VectorAgent] Indexing document: {} (Modality: {:?})",
            id,
            modality
        );
        let request = WorkerKnowledgeIndexRequest {
            space_id: "cortex-desktop".to_string(),
            source_ref: format!("cortex://desktop/{id}"),
            source_type: Some("desktop".to_string()),
            idempotency_key: runtime_agents::build_index_idempotency_key(
                &id,
                chrono::Utc::now().timestamp_millis(),
            ),
            documents: vec![WorkerKnowledgeIndexDocument {
                id: id.clone(),
                text: content,
                label: Some(format!("desktop:{id}")),
                space_id: Some("cortex-desktop".to_string()),
                source_ref: Some(format!("cortex://desktop/{id}")),
                source_type: Some("desktop".to_string()),
                tags: Some(vec!["cortex-desktop".to_string()]),
                timestamp_ms: Some(chrono::Utc::now().timestamp_millis()),
                modality: Some(modality.as_worker_modality().to_string()),
            }],
        };

        let endpoint = format!("{}/knowledge/index", worker_api_base());
        let result = reqwest::Client::new()
            .post(endpoint)
            .json(&request)
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                let parsed = resp.json::<WorkerKnowledgeIndexResponse>().await.ok();
                if let Some(payload) = parsed {
                    tracing::info!(
                        "[VectorAgent] Indexed '{}' (count={}, duplicate={})",
                        id,
                        payload.indexed_count,
                        payload.skipped_duplicate
                    );
                } else {
                    tracing::info!("[VectorAgent] Indexed '{}'", id);
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!("[VectorAgent] Index request failed: {} {}", status, body);
            }
            Err(err) => {
                tracing::error!("[VectorAgent] Index request failed: {}", err);
            }
        }
    }

    pub async fn search(query: String, modality_filter: Option<Modality>) -> Vec<SearchResult> {
        tracing::info!(
            "[VectorAgent] Searching for: \"{}\" (Filter: {:?})",
            query,
            modality_filter
        );
        if query.trim().is_empty() {
            return Vec::new();
        }

        let filters = modality_filter
            .as_ref()
            .map(|modality| WorkerKnowledgeSearchFilters {
                modalities: vec![modality.as_worker_modality().to_string()],
            });
        let request = WorkerKnowledgeSearchRequest {
            query: query.clone(),
            limit: Some(8),
            retrieval_mode: Some("hybrid".to_string()),
            filters,
            diagnostics: Some(false),
            rerank_enabled: Some(true),
        };

        let endpoint = format!("{}/knowledge/search", worker_api_base());
        let response = reqwest::Client::new()
            .post(endpoint)
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let items = resp
                    .json::<Vec<WorkerKnowledgeSearchResult>>()
                    .await
                    .unwrap_or_default();
                let mapped = items
                    .into_iter()
                    .map(SearchResult::from_worker_result)
                    .collect::<Vec<_>>();
                tracing::info!("[VectorAgent] Search returned {} result(s)", mapped.len());
                mapped
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!("[VectorAgent] Search request failed: {} {}", status, body);
                vec![SearchResult::availability_notice(format!(
                    "Search unavailable: worker returned {}",
                    status
                ))]
            }
            Err(err) => {
                tracing::error!("[VectorAgent] Search request failed: {}", err);
                vec![SearchResult::availability_notice(
                    "Search unavailable: worker endpoint unreachable.".to_string(),
                )]
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub modality: Modality,
}

impl Modality {
    fn as_worker_modality(&self) -> &'static str {
        runtime_agents::worker_modality(&match self {
            Modality::Text => cortex_runtime::agents::types::Modality::Text,
            Modality::Image => cortex_runtime::agents::types::Modality::Image,
            Modality::Audio => cortex_runtime::agents::types::Modality::Audio,
            Modality::Video => cortex_runtime::agents::types::Modality::Video,
        })
    }

    fn from_worker_modality(raw: Option<&str>) -> Self {
        match runtime_agents::modality_from_worker(raw) {
            cortex_runtime::agents::types::Modality::Text => Modality::Text,
            cortex_runtime::agents::types::Modality::Image => Modality::Image,
            cortex_runtime::agents::types::Modality::Audio => Modality::Audio,
            cortex_runtime::agents::types::Modality::Video => Modality::Video,
        }
    }
}

impl SearchResult {
    fn from_worker_result(item: WorkerKnowledgeSearchResult) -> Self {
        let mut metadata = item.metadata.unwrap_or_default();
        if let Some(source_ref) = item.source_ref {
            metadata
                .entry("source_ref".to_string())
                .or_insert(source_ref);
        }
        if let Some(space_id) = item.space_id {
            metadata.entry("space_id".to_string()).or_insert(space_id);
        }
        if let Some(source_type) = item.source_type {
            metadata
                .entry("source_type".to_string())
                .or_insert(source_type);
        }
        if let Some(modality) = item.modality.clone() {
            metadata
                .entry("modality".to_string())
                .or_insert(modality.to_ascii_lowercase());
        }

        Self {
            id: item.id,
            score: item.score,
            content: item.content.unwrap_or_default(),
            metadata,
            modality: Modality::from_worker_modality(item.modality.as_deref()),
        }
    }

    fn availability_notice(message: String) -> Self {
        Self {
            id: "search-unavailable".to_string(),
            score: 0.0,
            content: message,
            metadata: std::collections::HashMap::new(),
            modality: Modality::Text,
        }
    }
}

fn worker_api_base() -> String {
    std::env::var("NOSTRA_WORKER_API_BASE")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:3003".to_string())
}

#[derive(Debug, Clone, Serialize)]
struct WorkerKnowledgeIndexDocument {
    id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modality: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerKnowledgeIndexRequest {
    space_id: String,
    source_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<String>,
    idempotency_key: String,
    documents: Vec<WorkerKnowledgeIndexDocument>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkerKnowledgeIndexResponse {
    indexed_count: usize,
    skipped_duplicate: bool,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerKnowledgeSearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieval_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<WorkerKnowledgeSearchFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rerank_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerKnowledgeSearchFilters {
    modalities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct WorkerKnowledgeSearchResult {
    id: String,
    #[serde(default)]
    score: f32,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    modality: Option<String>,
    #[serde(default)]
    source_ref: Option<String>,
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    source_type: Option<String>,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Clone, Default)]
pub struct DesktopAgentProcessAdapter;

#[cfg(feature = "service-scaffolds")]
#[async_trait::async_trait]
impl AgentProcessAdapter for DesktopAgentProcessAdapter {
    async fn spawn_supervised(&self, program: &str, args: &[String]) -> Result<(), RuntimeError> {
        let program = program.to_string();
        let args = args.to_vec();
        tokio::spawn(async move {
            let _ = std::process::Command::new(program).args(args).output();
        });
        Ok(())
    }

    async fn probe_canister_status(&self, canister: &str) -> Result<bool, RuntimeError> {
        let output = std::process::Command::new("dfx")
            .arg("canister")
            .arg("status")
            .arg(canister)
            .output()
            .map_err(|err| RuntimeError::Network(err.to_string()))?;
        Ok(output.status.success())
    }

    async fn emit_log(&self, line: &str) -> Result<(), RuntimeError> {
        tracing::info!("[AgentZero] {}", line);
        Ok(())
    }
}
