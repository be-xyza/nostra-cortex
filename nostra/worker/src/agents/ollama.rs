use super::{
    AgentResponse, AgentRunner, ChatMessage, FinishReason, MessageRole, ToolCall, ToolDefinition,
};
use crate::config_service::ConfigService;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ollama Agent Runner using OpenAI-compatible chat API
pub struct OllamaAgentRunner {
    api_base: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaAgentRunner {
    /// Create from ConfigService (Nostra standard pattern)
    pub fn from_config(model: &str) -> Self {
        let config = ConfigService::get();
        let api_base = config
            .get_llm_config()
            .map(|c| c.api_base.clone())
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        Self {
            api_base,
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with explicit endpoint (for testing)
    pub fn new(api_base: &str, model: &str) -> Self {
        Self {
            api_base: api_base.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Convert our messages to Ollama chat format
    fn build_request(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> OllamaChatRequest {
        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: m.content.clone(),
                tool_calls: m.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .map(|tc| OllamaToolCall {
                            id: uuid::Uuid::new_v4().to_string(),
                            r#type: "function".to_string(),
                            function: OllamaFunctionCall {
                                name: tc.name.clone(),
                                arguments: serde_json::to_value(&tc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect()
                }),
                tool_call_id: m.tool_call_id.clone(),
            })
            .collect();

        let ollama_tools: Option<Vec<OllamaTool>> = if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .iter()
                    .map(|t| OllamaTool {
                        r#type: "function".to_string(),
                        function: OllamaFunction {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            parameters: t.parameters.clone(),
                        },
                    })
                    .collect(),
            )
        };

        OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            tools: ollama_tools,
            stream: false,
        }
    }
}

#[async_trait]
impl AgentRunner for OllamaAgentRunner {
    async fn execute(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse> {
        let request = self.build_request(messages, tools);
        let url = format!("{}/api/chat", self.api_base);

        println!(
            "[OllamaAgent] Sending request to {} with model {}",
            url, self.model
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
        println!("[OllamaAgent] Raw response: {}", body);

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Ollama returned error status {}: {}",
                status,
                body
            ));
        }

        let ollama_response: OllamaChatResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        // Convert Ollama response to our AgentResponse
        let mut tool_calls: Vec<ToolCall> = ollama_response
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                let arguments: HashMap<String, serde_json::Value> = match tc.function.arguments {
                    serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
                    serde_json::Value::Object(map) => map.into_iter().collect(),
                    _ => HashMap::new(),
                };
                ToolCall {
                    name: tc.function.name,
                    arguments,
                }
            })
            .collect();

        // Fallback: If tool_calls is empty, check content for hallucinated JSON
        if tool_calls.is_empty() {
            let content = &ollama_response.message.content;
            if let Some(start) = content.find('{') {
                if let Some(end) = content.rfind('}') {
                    let json_str = &content[start..=end];
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let (Some(name), Some(params)) = (
                            value.get("name").and_then(|v| v.as_str()),
                            value.get("parameters").and_then(|v| v.as_object()),
                        ) {
                            tool_calls.push(ToolCall {
                                name: name.to_string(),
                                arguments: params
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect(),
                            });
                        } else if let (Some(name), Some(args)) = (
                            value.get("name").and_then(|v| v.as_str()),
                            value.get("arguments").and_then(|v| v.as_object()),
                        ) {
                            // Support "arguments" key as well
                            tool_calls.push(ToolCall {
                                name: name.to_string(),
                                arguments: args
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect(),
                            });
                        }
                    }
                }
            }
        }

        let finish_reason = if !tool_calls.is_empty() {
            FinishReason::ToolCall
        } else if ollama_response.done {
            FinishReason::Stop
        } else {
            FinishReason::MaxTokens
        };

        Ok(AgentResponse {
            content: if ollama_response.message.content.is_empty() {
                None
            } else {
                Some(ollama_response.message.content)
            },
            tool_calls,
            finish_reason,
        })
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// Ollama API types (OpenAI-compatible format)

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool>>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct OllamaToolCall {
    id: String,
    r#type: String,
    function: OllamaFunctionCall,
}

#[derive(Serialize, Deserialize)]
struct OllamaFunctionCall {
    name: String,
    arguments: serde_json::Value, // Can be map or string
}

#[derive(Serialize)]
struct OllamaTool {
    r#type: String,
    function: OllamaFunction,
}

#[derive(Serialize)]
struct OllamaFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Option<Vec<OllamaToolCall>>,
}
