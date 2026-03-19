use crate::services::llm_adapter::config::LlmAdapterConfig;
use crate::services::llm_adapter::responses_types::{
    CompletedResponse, ResponsesEventEnvelope, ToolCallRequest,
};
use crate::services::llm_adapter::sse::{SseDataFrame, decode_sse_response};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmStreamEvent {
    TextDelta(String),
    ToolCallReady(ToolCallRequest),
    Completed { response_id: String },
}

pub struct LlmAdapterClient {
    http: reqwest::Client,
    config: LlmAdapterConfig,
}

impl LlmAdapterClient {
    pub fn new(config: LlmAdapterConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|err| err.to_string())?;
        Ok(Self { http, config })
    }

    pub fn config(&self) -> &LlmAdapterConfig {
        &self.config
    }

    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }

    pub async fn health_adapter(&self) -> Result<Value, String> {
        let url = format!("{}/health", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header_value())
            .send()
            .await
            .map_err(|err| format!("llm_adapter_health_request_failed:{err}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("llm_adapter_health_http_{status}:{body}"));
        }
        serde_json::from_str::<Value>(&body)
            .map_err(|err| format!("llm_adapter_health_parse_failed:{err}"))
    }

    pub async fn openapi_paths(&self) -> Result<Vec<String>, String> {
        let url = format!(
            "{}/openapi.json",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header_value())
            .send()
            .await
            .map_err(|err| format!("llm_adapter_openapi_request_failed:{err}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("llm_adapter_openapi_http_{status}:{body}"));
        }
        let parsed: Value = serde_json::from_str(&body)
            .map_err(|err| format!("llm_adapter_openapi_parse_failed:{err}"))?;
        let mut out: Vec<String> = parsed
            .get("paths")
            .and_then(|v| v.as_object())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        out.sort();
        Ok(out)
    }

    pub async fn health_upstream_models(&self) -> Result<Value, String> {
        let url = format!("{}/v1/models", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header_value())
            .send()
            .await
            .map_err(|err| format!("llm_adapter_models_request_failed:{err}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("llm_adapter_models_http_{status}:{body}"));
        }
        serde_json::from_str::<Value>(&body)
            .map_err(|err| format!("llm_adapter_models_parse_failed:{err}"))
    }

    pub fn build_base_request(
        &self,
        model: &str,
        instructions: &str,
        user_text: &str,
        tools: &[Value],
        previous_response_id: Option<&str>,
        tool_outputs: &[Value],
    ) -> Value {
        let mut input = vec![json!({
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": user_text}],
        })];
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

    pub async fn run_responses_stream(
        &self,
        body: Value,
        mut on_event: impl FnMut(LlmStreamEvent),
    ) -> Result<CompletedResponse, String> {
        let started = Instant::now();
        let base = self.config.base_url.trim_end_matches('/');
        let candidate_urls = [format!("{base}/responses"), format!("{base}/v1/responses")];

        let mut last_err: Option<String> = None;
        let mut response: Option<reqwest::Response> = None;
        for (idx, url) in candidate_urls.iter().enumerate() {
            let attempt = self
                .http
                .post(url)
                .header("Authorization", self.auth_header_value())
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            let resp = match attempt {
                Ok(resp) => resp,
                Err(err) => {
                    last_err = Some(format!("llm_adapter_request_failed:{err}"));
                    continue;
                }
            };

            if resp.status() == reqwest::StatusCode::NOT_FOUND && idx + 1 < candidate_urls.len() {
                // Adapter implementations vary (`/responses` vs `/v1/responses`). Retry on 404.
                continue;
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                last_err = Some(format!("llm_adapter_http_{status}:{text}"));
                continue;
            }

            response = Some(resp);
            break;
        }

        let response = response.ok_or_else(|| {
            last_err.unwrap_or_else(|| "llm_adapter_request_failed:unknown".to_string())
        })?;

        let frames = decode_sse_response(response).await?;
        parse_frames(frames, started.elapsed(), &mut on_event)
    }
}

fn parse_frames(
    frames: Vec<SseDataFrame>,
    _elapsed: Duration,
    on_event: &mut impl FnMut(LlmStreamEvent),
) -> Result<CompletedResponse, String> {
    let mut response_id: Option<String> = None;
    let mut full_text = String::new();
    let mut tool_calls_ready: Vec<ToolCallRequest> = Vec::new();
    let mut completed_raw: Option<Value> = None;

    for frame in frames {
        if frame.data.trim() == "[DONE]" {
            continue;
        }

        let value: Value = serde_json::from_str(&frame.data).map_err(|err| {
            format!(
                "llm_adapter_sse_json_parse_failed:{err} data={}",
                frame.data
            )
        })?;

        let envelope: ResponsesEventEnvelope = serde_json::from_value(value.clone())
            .map_err(|err| format!("llm_adapter_sse_envelope_parse_failed:{err}"))?;

        match envelope.event_type.as_str() {
            "response.created" => {
                if let Some(resp) = envelope.response.as_ref() {
                    if let Some(id) = resp.id.as_ref() {
                        response_id = Some(id.clone());
                    }
                }
            }
            "response.output_text.delta" => {
                let delta = value
                    .get("delta")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if !delta.is_empty() {
                    full_text.push_str(&delta);
                    on_event(LlmStreamEvent::TextDelta(delta));
                }
            }
            "response.completed" => {
                completed_raw = Some(value.clone());
                if let Some(resp) = envelope.response.as_ref() {
                    if let Some(id) = resp.id.as_ref() {
                        response_id = Some(id.clone());
                    }
                    tool_calls_ready = extract_ready_tool_calls(&resp.output);
                }
            }
            _ => {}
        }
    }

    let response_id = response_id.ok_or_else(|| "llm_adapter_missing_response_id".to_string())?;
    on_event(LlmStreamEvent::Completed {
        response_id: response_id.clone(),
    });

    Ok(CompletedResponse {
        response_id,
        full_text,
        tool_calls: tool_calls_ready,
        raw: completed_raw.unwrap_or(Value::Null),
    })
}

fn extract_ready_tool_calls(output: &[Value]) -> Vec<ToolCallRequest> {
    let mut out = Vec::new();
    for item in output {
        let item_type = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if item_type != "function_call" {
            continue;
        }
        let status = item
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if status != "ready" {
            continue;
        }
        let call_id = item
            .get("call_id")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("id").and_then(|v| v.as_str()))
            .unwrap_or_default()
            .to_string();
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let arguments_json = item
            .get("arguments")
            .and_then(|v| v.as_str())
            .unwrap_or("{}")
            .to_string();
        if call_id.is_empty() || name.is_empty() {
            continue;
        }
        out.push(ToolCallRequest {
            call_id,
            name,
            arguments_json,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm_adapter::sse::SseDataFrame;

    #[test]
    fn parses_text_delta_and_completed() {
        let frames = vec![
            SseDataFrame {
                data: r#"{"type":"response.created","response":{"id":"resp_1","output":[]}}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.output_text.delta","delta":"hello"}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.output_text.delta","delta":" world"}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.completed","response":{"id":"resp_1","output":[{"type":"message"}]}}"#
                    .to_string(),
            },
        ];

        let mut events = Vec::new();
        let completed = parse_frames(frames, Duration::from_millis(1), &mut |event| {
            events.push(event);
        })
        .unwrap();

        assert_eq!(completed.response_id, "resp_1");
        assert_eq!(completed.full_text, "hello world");
        assert!(completed.tool_calls.is_empty());
        assert!(
            events
                .iter()
                .any(|e| matches!(e, LlmStreamEvent::TextDelta(_)))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, LlmStreamEvent::Completed { .. }))
        );
    }

    #[test]
    fn extracts_ready_tool_calls_from_completed_output() {
        let frames = vec![SseDataFrame {
            data: r#"{"type":"response.completed","response":{"id":"resp_1","output":[{"type":"function_call","name":"echo","arguments":"{\"text\":\"hi\"}","call_id":"call_1","status":"ready"}]}}"#
                .to_string(),
        }];

        let mut ignored = Vec::new();
        let completed = parse_frames(frames, Duration::from_millis(1), &mut |event| {
            ignored.push(event);
        })
        .unwrap();

        assert_eq!(completed.tool_calls.len(), 1);
        assert_eq!(completed.tool_calls[0].call_id, "call_1");
        assert_eq!(completed.tool_calls[0].name, "echo");
        assert!(completed.tool_calls[0].arguments_json.contains("\"text\""));
    }
}
