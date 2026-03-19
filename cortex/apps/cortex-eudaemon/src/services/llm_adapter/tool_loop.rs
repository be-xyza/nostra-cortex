use crate::services::llm_adapter::client::{LlmAdapterClient, LlmStreamEvent};
use crate::services::llm_adapter::responses_types::{CompletedResponse, ToolCallRequest};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters_schema: Value,
}

pub fn builtin_tools() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "echo".to_string(),
            description: "Echo a message back to the caller.".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" }
                },
                "required": ["text"],
                "additionalProperties": false
            }),
        },
        ToolSpec {
            name: "noop".to_string(),
            description: "No-op tool for connectivity checks.".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolSpec {
            name: "time".to_string(),
            description: "Return the current UTC timestamp (RFC3339).".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
    ]
}

pub fn tool_schemas_for_adapter(tools: &[ToolSpec]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters_schema,
                }
            })
        })
        .collect()
}

pub async fn run_responses_tool_loop(
    client: &LlmAdapterClient,
    model: &str,
    instructions: &str,
    user_text: &str,
    tools: &[ToolSpec],
    mut on_event: impl FnMut(LlmStreamEvent),
) -> Result<(String, String), String> {
    let tool_schemas = tool_schemas_for_adapter(tools);
    let mut previous_response_id: Option<String> = None;
    let mut tool_outputs: Vec<Value> = Vec::new();

    let max_steps = client.config().max_tool_steps.max(1);
    let mut transcript = String::new();

    for step in 0..max_steps {
        let include_user = step == 0;
        let request_body = build_responses_request(
            model,
            instructions,
            if include_user { Some(user_text) } else { None },
            &tool_schemas,
            previous_response_id.as_deref(),
            &tool_outputs,
        );

        tool_outputs.clear();

        let completed: CompletedResponse = client
            .run_responses_stream(request_body, |event| {
                if let LlmStreamEvent::TextDelta(delta) = &event {
                    transcript.push_str(delta);
                }
                on_event(event);
            })
            .await?;

        if completed.tool_calls.is_empty() {
            return Ok((transcript, completed.response_id));
        }

        for call in completed.tool_calls.iter().cloned() {
            on_event(LlmStreamEvent::ToolCallReady(call.clone()));
            let output = execute_builtin_tool(&call).await?;
            tool_outputs.push(output);
        }

        previous_response_id = Some(completed.response_id);
    }

    Err("llm_adapter_tool_loop_max_steps_exceeded".to_string())
}

fn build_responses_request(
    model: &str,
    instructions: &str,
    user_text: Option<&str>,
    tools: &[Value],
    previous_response_id: Option<&str>,
    tool_outputs: &[Value],
) -> Value {
    let mut input: Vec<Value> = Vec::new();
    if let Some(text) = user_text {
        input.push(json!({
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": text}],
        }));
    }
    input.extend_from_slice(tool_outputs);

    let mut body = json!({
        "model": model,
        "instructions": instructions,
        "stream": true,
        "tools": tools,
        "input": input,
    });

    if let Some(prev) = previous_response_id {
        body["previous_response_id"] = Value::String(prev.to_string());
    }

    body
}

async fn execute_builtin_tool(call: &ToolCallRequest) -> Result<Value, String> {
    let args: Value = serde_json::from_str(&call.arguments_json).unwrap_or_else(|_| json!({}));

    let output_value = match call.name.as_str() {
        "echo" => {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            json!({ "text": text })
        }
        "noop" => json!({ "ok": true }),
        "time" => json!({ "utc": chrono::Utc::now().to_rfc3339() }),
        other => {
            return Err(format!("unsupported_tool:{other}"));
        }
    };

    let output_text = serde_json::to_string(&output_value)
        .map_err(|err| format!("tool_output_serialize_failed:{err}"))?;

    Ok(json!({
        "type": "function_call_output",
        "call_id": call.call_id,
        "output": output_text,
        "metadata": BTreeMap::<String, Value>::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm_adapter::config::{LlmAdapterConfig, LlmAdapterFailMode};
    use axum::extract::State;
    use axum::{Json, Router, routing::post};
    use serde_json::Value;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    async fn responses_handler(
        State(state): State<Arc<AtomicUsize>>,
        Json(_payload): Json<Value>,
    ) -> axum::response::Response {
        use axum::http::header::CONTENT_TYPE;
        use axum::response::IntoResponse;

        let call_index = state.fetch_add(1, Ordering::SeqCst);
        let body = if call_index == 0 {
            [
                "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"output\":[]}}",
                "",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\",\"output\":[{\"type\":\"function_call\",\"name\":\"echo\",\"arguments\":\"{\\\"text\\\":\\\"hi\\\"}\",\"call_id\":\"call_1\",\"status\":\"ready\"}]}}",
                "",
            ]
            .join("\n")
        } else {
            [
                "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_2\",\"output\":[]}}",
                "",
                "data: {\"type\":\"response.output_text.delta\",\"delta\":\"{\\\"intent\\\":\\\"create_context_node\\\",\\\"node_id\\\":\\\"ctx_test\\\",\\\"content\\\":\\\"ok\\\"}\"}",
                "",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_2\",\"output\":[{\"type\":\"message\"}]}}",
                "",
            ]
            .join("\n")
        };

        ([(CONTENT_TYPE, "text/event-stream")], body).into_response()
    }

    #[tokio::test]
    async fn tool_loop_executes_builtin_echo_and_completes() {
        use tokio::net::TcpListener;

        let counter = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route("/responses", post(responses_handler))
            .route("/v1/responses", post(responses_handler))
            .with_state(counter);

        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!(
                    "skipping llm_adapter tool loop test: tcp bind denied ({})",
                    err
                );
                return;
            }
            Err(err) => panic!("tcp bind failed: {}", err),
        };
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let cfg = LlmAdapterConfig {
            enabled: true,
            base_url: format!("http://{}:{}", addr.ip(), addr.port()),
            api_key: "sk-test".to_string(),
            request_timeout: Duration::from_secs(5),
            fail_mode: LlmAdapterFailMode::Fallback,
            max_tool_steps: 4,
            default_model: "llama3.1:8b".to_string(),
        };
        let client = LlmAdapterClient::new(cfg).unwrap();
        let tools = builtin_tools();

        let (text, response_id) = run_responses_tool_loop(
            &client,
            "llama3.1:8b",
            "return json only",
            "test",
            &tools,
            |_| {},
        )
        .await
        .unwrap();

        assert!(text.contains("\"intent\""));
        assert_eq!(response_id, "resp_2");
    }
}
