use async_trait::async_trait;
use cortex_agents::openai::OpenAiProvider;
use cortex_domain::agent::contracts::AgentIntent;
use cortex_domain::agent::provider::{
    CompletionRequest, CompletionResponse, Message, ModelProviderPort,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

use crate::temporal::Activity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningInput {
    pub contribution_id: String,
    pub space_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExecutionTrace {
    pub provider: String,
    pub latency_ms: u64,
    pub retry_count: u32,
    #[serde(default)]
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutput {
    pub action_targets_json: Vec<String>,
    pub provider_trace: ProviderExecutionTrace,
}

pub struct ReasoningActivity;

#[async_trait]
impl Activity<ReasoningInput, ReasoningOutput> for ReasoningActivity {
    const NAME: &'static str = "ReasoningActivity";

    async fn execute(&self, input: ReasoningInput) -> Result<ReasoningOutput, String> {
        let mut attempts = 0u32;
        let mut last_failure: Option<String> = None;

        let providers = provider_chain();
        for (provider_name, provider) in providers {
            attempts = attempts.saturating_add(1);
            let started = Instant::now();
            match provider
                .complete(CompletionRequest {
                    messages: vec![
                        Message {
                            role: "system".to_string(),
                            content: "You are SystemsArchitect. Return concise, actionable output."
                                .to_string(),
                        },
                        Message {
                            role: "user".to_string(),
                            content: format!(
                                "Design contribution scaffold for {} in space {}",
                                input.contribution_id, input.space_id
                            ),
                        },
                    ],
                    tools: Vec::new(),
                    model: model_for_provider(provider_name).to_string(),
                })
                .await
            {
                Ok(response) => {
                    let latency_ms = started.elapsed().as_millis() as u64;
                    let intent = intent_from_response(&input.contribution_id, response);
                    let serialized = serde_json::to_string(&intent)
                        .map_err(|err| format!("Failed to serialize agent intent: {err}"))?;
                    return Ok(ReasoningOutput {
                        action_targets_json: vec![serialized],
                        provider_trace: ProviderExecutionTrace {
                            provider: provider_name.to_string(),
                            latency_ms,
                            retry_count: attempts.saturating_sub(1),
                            failure_class: None,
                        },
                    });
                }
                Err(err) => {
                    last_failure = Some(format!("{provider_name}:{err}"));
                }
            }
        }

        let fallback_intent = AgentIntent::CreateContextNode {
            node_id: format!("ctx_{}", input.contribution_id),
            content: format!("Scaffold contribution {}", input.contribution_id),
        };
        let serialized = serde_json::to_string(&fallback_intent)
            .map_err(|err| format!("Failed to serialize fallback intent: {err}"))?;

        Ok(ReasoningOutput {
            action_targets_json: vec![serialized],
            provider_trace: ProviderExecutionTrace {
                provider: "fallback_deterministic".to_string(),
                latency_ms: 0,
                retry_count: attempts,
                failure_class: last_failure,
            },
        })
    }
}

fn intent_from_response(contribution_id: &str, response: CompletionResponse) -> AgentIntent {
    let content = response
        .content
        .unwrap_or_else(|| format!("Scaffold contribution {contribution_id}"));
    AgentIntent::CreateContextNode {
        node_id: format!("ctx_{}", contribution_id),
        content,
    }
}

fn provider_chain() -> Vec<(&'static str, Box<dyn ModelProviderPort>)> {
    let mut providers: Vec<(&'static str, Box<dyn ModelProviderPort>)> = Vec::new();

    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        providers.push((
            "openai",
            Box::new(OpenAiProvider::new(
                api_key,
                std::env::var("NOSTRA_AGENT_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            )),
        ));
    }

    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        providers.push((
            "anthropic",
            Box::new(AnthropicProvider::new(
                api_key,
                std::env::var("NOSTRA_AGENT_ANTHROPIC_MODEL")
                    .unwrap_or_else(|_| "claude-3-5-sonnet-latest".to_string()),
            )),
        ));
    }

    providers
}

fn model_for_provider(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "claude-3-5-sonnet-latest",
        _ => "gpt-4o-mini",
    }
}

struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelProviderPort for AnthropicProvider {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, String> {
        let system_prompt = req
            .messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        let messages = req
            .messages
            .iter()
            .filter(|message| message.role != "system")
            .map(|message| {
                json!({
                    "role": message.role,
                    "content": [{ "type": "text", "text": message.content }]
                })
            })
            .collect::<Vec<_>>();

        let body = json!({
            "model": if req.model.trim().is_empty() { self.model.clone() } else { req.model },
            "max_tokens": 512,
            "system": system_prompt,
            "messages": messages
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|err| format!("anthropic_request_failed:{err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(format!("anthropic_http_{status}:{body_text}"));
        }

        let parsed: serde_json::Value = response
            .json()
            .await
            .map_err(|err| format!("anthropic_parse_failed:{err}"))?;

        let content = parsed
            .get("content")
            .and_then(|value| value.as_array())
            .and_then(|items| {
                items.iter().find_map(|item| {
                    if item.get("type").and_then(|value| value.as_str()) == Some("text") {
                        item.get("text")
                            .and_then(|value| value.as_str())
                            .map(|value| value.to_string())
                    } else {
                        None
                    }
                })
            });

        Ok(CompletionResponse {
            content,
            tool_calls: Vec::new(),
        })
    }

    async fn embed(&self, _text: String) -> Result<Vec<f32>, String> {
        Err("anthropic_embedding_not_supported".to_string())
    }
}
