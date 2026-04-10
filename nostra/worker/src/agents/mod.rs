mod ollama;

pub use ollama::OllamaAgentRunner;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Tool call representation for agent-tool interaction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}

/// Response from an agent execution
#[derive(Clone, Debug)]
pub struct AgentResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FinishReason {
    Stop,
    ToolCall,
    MaxTokens,
    Error(String),
}

/// Message in the conversation history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool definition for function calling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

/// The core trait for LLM agent execution
#[async_trait]
pub trait AgentRunner: Send + Sync {
    /// Execute a single turn of conversation
    async fn execute(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse>;

    /// Get the name/identifier of this agent runner
    fn name(&self) -> &str;

    /// Get the model being used
    fn model(&self) -> &str;
}

/// Mock agent runner for testing
pub struct MockAgentRunner {
    responses: Vec<AgentResponse>,
    current: std::sync::atomic::AtomicUsize,
}

impl MockAgentRunner {
    pub fn new(responses: Vec<AgentResponse>) -> Self {
        Self {
            responses,
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create a mock that returns a simple text response
    pub fn with_text_response(text: &str) -> Self {
        Self::new(vec![AgentResponse {
            content: Some(text.to_string()),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
        }])
    }

    /// Create a mock that returns a tool call
    pub fn with_tool_call(
        name: &str,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::new(vec![AgentResponse {
            content: None,
            tool_calls: vec![ToolCall {
                name: name.to_string(),
                arguments: args,
            }],
            finish_reason: FinishReason::ToolCall,
        }])
    }
}

#[async_trait]
impl AgentRunner for MockAgentRunner {
    async fn execute(
        &self,
        _messages: &[ChatMessage],
        _tools: &[ToolDefinition],
    ) -> Result<AgentResponse> {
        let idx = self
            .current
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let response = self
            .responses
            .get(idx % self.responses.len())
            .cloned()
            .unwrap_or_else(|| AgentResponse {
                content: Some("Mock response".to_string()),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
            });
        Ok(response)
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn model(&self) -> &str {
        "mock-model"
    }
}
