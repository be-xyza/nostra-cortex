use cortex_runtime::agents::service as runtime_agents;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::services::provider_runtime::client::{
    ProviderRuntimeClient, ProviderRuntimeStreamEvent,
};
use crate::services::provider_runtime::config::{
    ProviderRuntimeFailMode, provider_runtime_config_from_env, resolve_provider_runtime_state,
};

#[cfg(feature = "service-scaffolds")]
use std::sync::OnceLock;

#[cfg(feature = "service-scaffolds")]
use tokio::sync::broadcast;

#[cfg(feature = "service-scaffolds")]
use cortex_runtime::{RuntimeError, ports::AgentProcessAdapter};

#[cfg(feature = "service-scaffolds")]
use futures_util::Stream;
#[cfg(feature = "service-scaffolds")]
use std::pin::Pin;

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

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatAgentIdentity {
    pub id: String,
    pub label: String,
    pub route: String,
    pub mode: String,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatContextBlock {
    pub artifact_id: String,
    pub title: String,
    pub block_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mentions: Vec<String>,
    pub surface_json: Value,
    pub updated_at: String,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatSourceAnchor {
    pub kind: String,
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub artifact_id: Option<String>,
    #[serde(default)]
    pub view_id: Option<String>,
    #[serde(default)]
    pub block_id: Option<String>,
    #[serde(default)]
    pub component_id: Option<String>,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatHistoryTurn {
    pub role: String,
    pub text: String,
    pub timestamp: i64,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ChatContentPart {
    Text {
        text: String,
    },
    A2ui {
        surface_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        tree: Value,
    },
    Pointer {
        href: String,
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        artifact_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeChatRequest {
    pub author: String,
    pub text: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub source_anchor: Option<ChatSourceAnchor>,
    #[serde(default)]
    pub context_blocks: Vec<ChatContextBlock>,
    #[serde(default)]
    pub history: Vec<ChatHistoryTurn>,
    #[serde(default = "default_true")]
    pub streaming: bool,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeChatResponse {
    pub response_id: String,
    pub text: String,
    pub content: Vec<ChatContentPart>,
    pub agent: ChatAgentIdentity,
}

#[cfg(feature = "service-scaffolds")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatEvent {
    TextDelta {
        author: String,
        message: String,
        timestamp: i64,
        agent: ChatAgentIdentity,
    },
    Completed {
        response: RuntimeChatResponse,
        timestamp: i64,
    },
}

pub struct AgentService;

#[cfg(feature = "service-scaffolds")]
static STATE_CHANNEL: OnceLock<broadcast::Sender<AgentSystemState>> = OnceLock::new();

#[cfg(feature = "service-scaffolds")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "service-scaffolds")]
pub fn get_agent_state_tx() -> broadcast::Sender<AgentSystemState> {
    STATE_CHANNEL
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        })
        .clone()
}

#[cfg(feature = "service-scaffolds")]
pub(crate) fn ensure_thread_context(message: String, thread_id: Option<&str>) -> String {
    let Some(thread_id) = thread_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return message;
    };

    if message.contains("Conversation thread:") {
        return message;
    }

    format!("Conversation thread: {thread_id}\n{message}")
}

#[cfg(feature = "service-scaffolds")]
pub(crate) fn build_chat_message_payload(message: &str, thread_id: Option<&str>) -> String {
    let mut payload = serde_json::json!({
        "type": "chat_message",
        "text": message,
        "contextRefs": [],
    });
    if let Some(thread_id) = thread_id.map(str::trim).filter(|value| !value.is_empty()) {
        payload["threadId"] = serde_json::Value::String(thread_id.to_string());
    }
    payload.to_string()
}

impl AgentService {
    pub async fn spawn_vector_agent() {
        tracing::info!("Spawning Vector Agent (Elna Interface)...");
        // In a real scenario, this would check if 'elna' canister is running
        // and start a background task to index notes.
        tokio::spawn(async move {
            tracing::info!("[VectorAgent] Connecting to Elna Knowledge Store...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Mock connection success/fail based on the canonical IC CLI lane.
            let output = std::process::Command::new("icp")
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

        let client = crate::services::ic_client::IcClient::new(None);

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
        request: RuntimeChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatEvent, String>> + Send>>, String> {
        let response = if provider_runtime_config_from_env().enabled {
            match Self::run_runtime_chat(&request).await {
                Ok(response) => response,
                Err(err) => {
                    if provider_runtime_config_from_env().fail_mode
                        == ProviderRuntimeFailMode::FailClosed
                    {
                        return Err(err);
                    }
                    tracing::warn!("runtime chat fallback engaged: {}", err);
                    Self::run_legacy_chat(&request).await?
                }
            }
        } else {
            Self::run_legacy_chat(&request).await?
        };

        let timestamp = chrono::Utc::now().timestamp();
        let mut events: Vec<Result<ChatEvent, String>> = Vec::new();
        if request.streaming {
            for chunk in chunk_chat_response(response.text.clone()) {
                if chunk.trim().is_empty() {
                    continue;
                }
                events.push(Ok(ChatEvent::TextDelta {
                    author: response.agent.label.clone(),
                    message: chunk,
                    timestamp,
                    agent: response.agent.clone(),
                }));
            }
        }
        events.push(Ok(ChatEvent::Completed {
            response,
            timestamp,
        }));
        Ok(Box::pin(futures_util::stream::iter(events)))
    }

    #[cfg(feature = "service-scaffolds")]
    async fn run_runtime_chat(request: &RuntimeChatRequest) -> Result<RuntimeChatResponse, String> {
        let cfg = provider_runtime_config_from_env();
        let resolved_provider = resolve_provider_runtime_state();
        let client = ProviderRuntimeClient::new(cfg.clone())
            .map_err(|err| format!("provider_runtime_client_init_failed:{err}"))?;
        let instructions = build_runtime_chat_instructions(request);
        let user_text = build_runtime_chat_user_text(request);
        let request_body = client.build_base_request(
            &cfg.default_model,
            &instructions,
            &user_text,
            &[],
            None,
            &[],
        );

        let mut transcript = String::new();
        let completed = client
            .run_responses_stream(request_body, |event| {
                if let ProviderRuntimeStreamEvent::TextDelta(delta) = event {
                    transcript.push_str(&delta);
                }
            })
            .await?;

        let text = if transcript.trim().is_empty() {
            completed.full_text.clone()
        } else {
            transcript
        };

        let agent = ChatAgentIdentity {
            id: resolved_provider.provider_id.clone(),
            label: "Cortex Runtime".to_string(),
            route: "provider-runtime.responses".to_string(),
            mode: "runtime".to_string(),
        };

        Ok(RuntimeChatResponse {
            response_id: completed.response_id,
            text: text.clone(),
            content: build_chat_content_parts(&text, request),
            agent,
        })
    }

    #[cfg(feature = "service-scaffolds")]
    async fn run_legacy_chat(request: &RuntimeChatRequest) -> Result<RuntimeChatResponse, String> {
        let client = crate::services::ic_client::IcClient::new(None);
        let mut message = request.text.clone();

        if let Some(space) = request.space_id.as_deref() {
            message = format!(
                "Context: Operating within Space ID '{}'.\nUser Query: {}",
                space, message
            );
        }

        message = ensure_thread_context(message, request.thread_id.as_deref());
        let arg = format!(
            "(\"{}\")",
            build_chat_message_payload(message.as_str(), request.thread_id.as_deref())
                .replace("\"", "\\\"")
        );

        let response = client
            .call_canister("workflow-engine", "process_message", Some(&arg))
            .await
            .map_err(|err| err.to_string())?;
        let decoded = crate::services::ic_client::IcClient::unwrap_candid_string(&response)
            .unwrap_or(response);
        let agent = ChatAgentIdentity {
            id: "workflow-engine".to_string(),
            label: "Workflow Engine Fallback".to_string(),
            route: "workflow-engine.process_message".to_string(),
            mode: "fallback".to_string(),
        };

        Ok(RuntimeChatResponse {
            response_id: format!(
                "legacy-{}",
                request
                    .thread_id
                    .clone()
                    .unwrap_or_else(|| "chat".to_string())
            ),
            text: decoded.clone(),
            content: build_chat_content_parts(&decoded, request),
            agent,
        })
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

#[cfg(all(test, feature = "service-scaffolds"))]
mod tests {
    use super::{
        ChatContentPart, ChatContextBlock, ChatHistoryTurn, ChatSourceAnchor, RuntimeChatRequest,
        build_chat_content_parts, build_chat_message_payload, build_runtime_chat_user_text,
        chunk_chat_response, ensure_thread_context,
    };
    use serde_json::json;

    #[test]
    fn ensure_thread_context_adds_missing_thread_header() {
        let message = "User request: Summarize the current status.".to_string();

        let result = ensure_thread_context(message, Some("thread-42"));

        assert!(result.contains("Conversation thread: thread-42"));
        assert!(result.contains("User request: Summarize the current status."));
    }

    #[test]
    fn ensure_thread_context_preserves_existing_thread_header() {
        let message = "Conversation thread: thread-42\nUser request: Summarize the current status."
            .to_string();

        let result = ensure_thread_context(message.clone(), Some("thread-42"));

        assert_eq!(result, message);
    }

    #[test]
    fn build_chat_message_payload_embeds_thread_id() {
        let payload = build_chat_message_payload(
            "Conversation thread: thread-42\nUser request: Summarize the current status.",
            Some("thread-42"),
        );

        assert!(payload.contains(r#""type":"chat_message""#));
        assert!(payload.contains(r#""threadId":"thread-42""#));
        assert!(payload.contains(
            r#""text":"Conversation thread: thread-42\nUser request: Summarize the current status."#
        ));
    }

    #[test]
    fn chunk_chat_response_breaks_long_messages_on_word_boundaries() {
        let message = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega ".repeat(6);

        let chunks = chunk_chat_response(message.clone());

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 120));
        assert_eq!(
            chunks.join(" "),
            message.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }

    #[test]
    fn build_runtime_chat_user_text_captures_thread_history_and_context_bundle() {
        let request = RuntimeChatRequest {
            author: "User".to_string(),
            text: "Summarize the selected block.".to_string(),
            space_id: Some("nostra-space".to_string()),
            thread_id: Some("thread-42".to_string()),
            source_anchor: Some(ChatSourceAnchor {
                kind: "view".to_string(),
                label: "Explore".to_string(),
                href: "/explore".to_string(),
                route_id: Some("/explore".to_string()),
                artifact_id: None,
                view_id: Some("aggregate:prompts".to_string()),
                block_id: None,
                component_id: None,
            }),
            context_blocks: vec![ChatContextBlock {
                artifact_id: "artifact-1".to_string(),
                title: "Heap Parity Card".to_string(),
                block_type: "note".to_string(),
                tags: vec!["architecture".to_string()],
                mentions: vec!["chart_summary_metrics".to_string()],
                surface_json: json!({ "plain_text": "Heap parity body" }),
                updated_at: "2026-03-28T00:00:00Z".to_string(),
            }],
            history: vec![
                ChatHistoryTurn {
                    role: "user".to_string(),
                    text: "Earlier question".to_string(),
                    timestamp: 1,
                },
                ChatHistoryTurn {
                    role: "agent".to_string(),
                    text: "Earlier answer".to_string(),
                    timestamp: 2,
                },
            ],
            streaming: true,
        };

        let body = build_runtime_chat_user_text(&request);

        assert!(body.contains("Thread ID: thread-42"));
        assert!(body.contains("Recent history:\nuser: Earlier question\nagent: Earlier answer"));
        assert!(body.contains("Canonical context bundle:"));
        assert!(body.contains("- Heap Parity Card [note]"));
        assert!(body.contains("User request:\nSummarize the selected block."));
    }

    #[test]
    fn build_chat_content_parts_adds_structured_context_summary_and_pointers() {
        let request = RuntimeChatRequest {
            author: "User".to_string(),
            text: "Summarize the selected block.".to_string(),
            space_id: Some("nostra-space".to_string()),
            thread_id: Some("thread-42".to_string()),
            source_anchor: None,
            context_blocks: vec![
                ChatContextBlock {
                    artifact_id: "artifact-1".to_string(),
                    title: "Heap Parity Card".to_string(),
                    block_type: "note".to_string(),
                    tags: vec!["architecture".to_string()],
                    mentions: vec![],
                    surface_json: json!({ "plain_text": "Heap parity body" }),
                    updated_at: "2026-03-28T00:00:00Z".to_string(),
                },
                ChatContextBlock {
                    artifact_id: "artifact-2".to_string(),
                    title: "Platform Metrics".to_string(),
                    block_type: "telemetry".to_string(),
                    tags: vec!["metrics".to_string()],
                    mentions: vec![],
                    surface_json: json!({ "type": "chart" }),
                    updated_at: "2026-03-28T00:00:01Z".to_string(),
                },
            ],
            history: Vec::new(),
            streaming: true,
        };

        let parts = build_chat_content_parts("Here is the grounded summary.", &request);

        assert!(matches!(
            parts.first(),
            Some(ChatContentPart::Text { text }) if text == "Here is the grounded summary."
        ));
        assert!(matches!(
            parts.get(1),
            Some(ChatContentPart::A2ui { surface_id, .. }) if surface_id == "chat_context_summary:thread-42"
        ));
        assert!(matches!(
            parts.get(2),
            Some(ChatContentPart::Pointer { artifact_id: Some(id), .. }) if id == "artifact-1"
        ));
        assert!(matches!(
            parts.get(3),
            Some(ChatContentPart::Pointer { artifact_id: Some(id), .. }) if id == "artifact-2"
        ));
    }
}

#[cfg(feature = "service-scaffolds")]
fn chunk_chat_response(message: String) -> Vec<String> {
    const CHUNK_TARGET: usize = 120;

    if message.len() <= CHUNK_TARGET {
        return vec![message];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in message.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if next_len > CHUNK_TARGET && !current.is_empty() {
            chunks.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    if chunks.is_empty() {
        vec![message]
    } else {
        chunks
    }
}

#[cfg(feature = "service-scaffolds")]
fn build_runtime_chat_instructions(request: &RuntimeChatRequest) -> String {
    let mut sections = vec![
        "You are Cortex Runtime Chat.".to_string(),
        "Answer the user using the supplied thread history and canonical heap context bundle."
            .to_string(),
        "Ground claims in the provided context blocks and source anchor when available."
            .to_string(),
        "If context is missing, say what is missing instead of inventing facts.".to_string(),
        "Keep the answer concise, useful, and operationally relevant.".to_string(),
    ];

    if let Some(anchor) = &request.source_anchor {
        sections.push(format!(
            "Conversation source anchor: {} ({}) {}",
            anchor.label, anchor.kind, anchor.href
        ));
    }

    if !request.context_blocks.is_empty() {
        sections.push(format!(
            "Context blocks available: {}",
            request.context_blocks.len()
        ));
    }

    sections.join("\n")
}

#[cfg(feature = "service-scaffolds")]
fn build_runtime_chat_user_text(request: &RuntimeChatRequest) -> String {
    let mut sections = Vec::new();

    if let Some(thread_id) = request
        .thread_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sections.push(format!("Thread ID: {thread_id}"));
    }

    if !request.history.is_empty() {
        let history = request
            .history
            .iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|turn| format!("{}: {}", turn.role, turn.text))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("Recent history:\n{history}"));
    }

    if !request.context_blocks.is_empty() {
        let context = request
            .context_blocks
            .iter()
            .take(8)
            .map(|block| {
                format!(
                    "- {} [{}] tags={} mentions={} updated={} surface={}",
                    block.title,
                    block.block_type,
                    block.tags.join(", "),
                    block.mentions.join(", "),
                    block.updated_at,
                    summarize_surface_json(&block.surface_json)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("Canonical context bundle:\n{context}"));
    }

    sections.push(format!("User request:\n{}", request.text.trim()));
    sections.join("\n\n")
}

#[cfg(feature = "service-scaffolds")]
fn summarize_surface_json(value: &Value) -> String {
    let encoded = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    if encoded.len() <= 600 {
        return encoded;
    }
    format!("{}...", &encoded[..600])
}

#[cfg(feature = "service-scaffolds")]
fn build_chat_content_parts(text: &str, request: &RuntimeChatRequest) -> Vec<ChatContentPart> {
    let mut parts = Vec::new();
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        parts.push(ChatContentPart::Text {
            text: trimmed.to_string(),
        });
    }

    if !request.context_blocks.is_empty() {
        parts.push(ChatContentPart::A2ui {
            surface_id: format!(
                "chat_context_summary:{}",
                request
                    .thread_id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("standalone")
            ),
            title: Some("Resolved context bundle".to_string()),
            tree: build_context_summary_a2ui(request),
        });

        for block in request.context_blocks.iter().take(4) {
            parts.push(ChatContentPart::Pointer {
                href: format!("/explore?artifact_id={}", block.artifact_id),
                label: block.title.clone(),
                artifact_id: Some(block.artifact_id.clone()),
                description: Some(format!(
                    "{} · updated {}",
                    block.block_type, block.updated_at
                )),
            });
        }
    }

    parts
}

#[cfg(feature = "service-scaffolds")]
fn build_context_summary_a2ui(request: &RuntimeChatRequest) -> Value {
    let mut children = vec![json!({
        "id": "context-heading",
        "componentProperties": {
            "Heading": { "text": "Resolved context bundle" }
        }
    })];

    if let Some(anchor) = &request.source_anchor {
        children.push(json!({
            "id": "context-anchor",
            "componentProperties": {
                "Text": { "text": format!("Source: {} ({})", anchor.label, anchor.kind) }
            }
        }));
    }

    for block in request.context_blocks.iter().take(4) {
        children.push(json!({
            "id": format!("ctx-{}", block.artifact_id),
            "componentProperties": {
                "Text": {
                    "text": format!("{} [{}]", block.title, block.block_type)
                }
            }
        }));
    }

    json!({
        "type": "Container",
        "children": { "explicitList": children }
    })
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
        let output = std::process::Command::new("icp")
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
