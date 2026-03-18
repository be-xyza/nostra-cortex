use serde::{Deserialize, Serialize};

/// A single message in a conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// "system" | "user" | "assistant" | "tool"
    pub role: String,
    pub content: String,
}

/// JSON Schema definition for a tool the model may invoke, using the standard
/// OpenAI tool schema format (also accepted by Anthropic).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A single tool invocation returned by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallResult {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Everything the model needs to produce a completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSchema>,
    /// Caller-specified model identifier, e.g. "claude-3-5-sonnet-20241022".
    pub model: String,
}

/// The model's response to a CompletionRequest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Text content when the model replies with prose rather than a tool call.
    pub content: Option<String>,
    /// Tool invocations requested by the model, if any.
    pub tool_calls: Vec<ToolCallResult>,
}

/// The core trait representing the host's LLM capability.
#[async_trait::async_trait]
pub trait ModelProviderPort: Send + Sync {
    /// Provide a text completion.
    async fn complete(
        &self,
        req: CompletionRequest,
    ) -> core::result::Result<CompletionResponse, String>;

    /// Experimental: Embed a text chunk.
    async fn embed(&self, text: String) -> core::result::Result<Vec<f32>, String>;
}
