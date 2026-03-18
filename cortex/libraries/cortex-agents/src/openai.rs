use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionTool, ChatCompletionTools, CreateChatCompletionRequestArgs,
        FunctionObjectArgs,
    },
};
use async_trait::async_trait;

use crate::provider::{
    CompletionRequest, CompletionResponse, ModelProviderPort, ProviderError, ToolCall,
};

pub struct OpenAiProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: model.into(),
        }
    }
}

#[async_trait]
impl ModelProviderPort for OpenAiProvider {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let mut request_messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        for msg in &req.messages {
            let req_msg: ChatCompletionRequestMessage = match msg.role.as_str() {
                "user" => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content.clone())
                    .build()
                    .map_err(|e: OpenAIError| e.to_string())?
                    .into(),
                "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content.clone())
                    .build()
                    .map_err(|e: OpenAIError| e.to_string())?
                    .into(),
                "tool" => ChatCompletionRequestToolMessageArgs::default()
                    // Reusing content for tool output. Assuming name/id is folded into state elsewhere or not strictly required for this demo.
                    .content(msg.content.clone())
                    .tool_call_id("tool_call_placeholder")
                    .build()
                    .map_err(|e: OpenAIError| e.to_string())?
                    .into(),
                "system" => ChatCompletionRequestSystemMessageArgs::default()
                    .content(msg.content.clone())
                    .build()
                    .map_err(|e: OpenAIError| e.to_string())?
                    .into(),
                _ => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content.clone())
                    .build()
                    .map_err(|e: OpenAIError| e.to_string())?
                    .into(),
            };
            request_messages.push(req_msg);
        }

        let mut openai_tools = Vec::new();
        for tool in &req.tools {
            let function = FunctionObjectArgs::default()
                .name(&tool.name)
                .description(&tool.description)
                .parameters(tool.parameters.clone())
                .build()
                .map_err(|e: OpenAIError| e.to_string())?;

            let chat_tool = ChatCompletionTools::Function(ChatCompletionTool { function });

            openai_tools.push(chat_tool);
        }

        let mut request_args = CreateChatCompletionRequestArgs::default();
        request_args
            .model(if req.model.trim().is_empty() {
                &self.model
            } else {
                req.model.as_str()
            })
            .messages(request_messages);

        if !openai_tools.is_empty() {
            request_args.tools(openai_tools);
        }

        let request = request_args
            .build()
            .map_err(|e: OpenAIError| e.to_string())?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e: OpenAIError| e.to_string())?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| "No choices returned from OpenAI".to_string())?;

        let message = &choice.message;
        let mut parsed_calls = Vec::new();

        if let Some(tool_calls) = &message.tool_calls {
            for t_enum in tool_calls {
                if let ChatCompletionMessageToolCalls::Function(t) = t_enum {
                    let arguments: serde_json::Value = serde_json::from_str(&t.function.arguments)
                        .unwrap_or_else(|_| serde_json::json!({}));
                    parsed_calls.push(ToolCall {
                        id: t.id.clone(),
                        name: t.function.name.clone(),
                        arguments,
                    });
                }
            }
        }

        Ok(CompletionResponse {
            content: message.content.clone(),
            tool_calls: parsed_calls,
        })
    }

    async fn embed(&self, _text: String) -> Result<Vec<f32>, ProviderError> {
        Err("OpenAI embeddings are not wired in this provider yet".to_string())
    }
}
