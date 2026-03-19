pub use cortex_domain::agent::provider::{
    CompletionRequest, CompletionResponse, Message, ModelProviderPort, ToolCallResult, ToolSchema,
};

pub type ProviderError = String;
pub type ModelResponse = CompletionResponse;
pub type ProviderTool = ToolSchema;
pub type ToolCall = ToolCallResult;
