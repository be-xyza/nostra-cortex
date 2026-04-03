use crate::services::provider_runtime::config::ProviderRuntimeConfig;
use crate::services::provider_runtime::responses_types::{
    CompletedResponse, ResponseOutputItem, ResponsesEventEnvelope, ToolCallRequest,
};
use crate::services::provider_runtime::sse::{SseDataFrame, decode_sse_response};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderRuntimeStreamEvent {
    TextDelta(String),
    ToolCallReady(ToolCallRequest),
    Completed { response_id: String },
}

pub struct ProviderRuntimeClient {
    http: reqwest::Client,
    config: ProviderRuntimeConfig,
}

impl ProviderRuntimeClient {
    pub fn new(config: ProviderRuntimeConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|err| err.to_string())?;
        Ok(Self { http, config })
    }

    pub fn config(&self) -> &ProviderRuntimeConfig {
        &self.config
    }

    fn auth_header_value(&self) -> Option<String> {
        let api_key = self.config.api_key.trim();
        if api_key.is_empty() {
            None
        } else {
            Some(format!("Bearer {api_key}"))
        }
    }

    fn with_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(header) = self.auth_header_value() {
            request.header("Authorization", header)
        } else {
            request
        }
    }

    fn base_url(&self) -> &str {
        self.config.base_url.trim_end_matches('/')
    }

    fn looks_like_ollama(&self) -> bool {
        let base = self.base_url().to_ascii_lowercase();
        let model = self.config.default_model.to_ascii_lowercase();
        base.contains("ollama")
            || base.contains("11434")
            || model.contains("llama")
            || model.contains("qwen")
    }

    pub async fn health_adapter(&self) -> Result<Value, String> {
        let url = format!("{}/health", self.base_url());
        let response = self
            .with_auth(self.http.get(url))
            .send()
            .await
            .map_err(|err| format!("provider_runtime_health_request_failed:{err}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("provider_runtime_health_http_{status}:{body}"));
        }
        serde_json::from_str::<Value>(&body)
            .map_err(|err| format!("provider_runtime_health_parse_failed:{err}"))
    }

    pub async fn openapi_paths(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/openapi.json", self.base_url());
        let response = self
            .with_auth(self.http.get(url))
            .send()
            .await
            .map_err(|err| format!("provider_runtime_openapi_request_failed:{err}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("provider_runtime_openapi_http_{status}:{body}"));
        }
        let parsed: Value = serde_json::from_str(&body)
            .map_err(|err| format!("provider_runtime_openapi_parse_failed:{err}"))?;
        let mut out: Vec<String> = parsed
            .get("paths")
            .and_then(|v| v.as_object())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        out.sort();
        Ok(out)
    }

    pub async fn health_upstream_models(&self) -> Result<Value, String> {
        let base = self.base_url();
        let mut candidate_urls = vec![format!("{base}/v1/models"), format!("{base}/models")];
        if self.looks_like_ollama() {
            candidate_urls.push(format!("{base}/api/tags"));
        }

        let mut last_err: Option<String> = None;
        for url in candidate_urls {
            let response = match self.with_auth(self.http.get(&url)).send().await {
                Ok(response) => response,
                Err(err) => {
                    last_err = Some(format!("provider_runtime_models_request_failed:{err}"));
                    continue;
                }
            };
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if !status.is_success() {
                last_err = Some(format!("provider_runtime_models_http_{status}:{body}"));
                continue;
            }
            match serde_json::from_str::<Value>(&body) {
                Ok(value) => return Ok(value),
                Err(err) => {
                    last_err = Some(format!("provider_runtime_models_parse_failed:{err}"));
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| "provider_runtime_models_request_failed:unknown".to_string()))
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
        mut on_event: impl FnMut(ProviderRuntimeStreamEvent),
    ) -> Result<CompletedResponse, String> {
        let started = Instant::now();
        let base = self.base_url();
        let candidate_urls = [format!("{base}/responses"), format!("{base}/v1/responses")];

        let mut last_err: Option<String> = None;
        let mut response: Option<reqwest::Response> = None;
        for (idx, url) in candidate_urls.iter().enumerate() {
            let attempt = self
                .with_auth(self.http.post(url))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            let resp = match attempt {
                Ok(resp) => resp,
                Err(err) => {
                    last_err = Some(format!("provider_runtime_request_failed:{err}"));
                    continue;
                }
            };

            if resp.status() == reqwest::StatusCode::NOT_FOUND && idx + 1 < candidate_urls.len() {
                continue;
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                last_err = Some(format!("provider_runtime_http_{status}:{text}"));
                continue;
            }

            response = Some(resp);
            break;
        }

        let response = response.ok_or_else(|| {
            last_err.unwrap_or_else(|| "provider_runtime_request_failed:unknown".to_string())
        })?;

        let frames = decode_sse_response(response).await?;
        parse_frames(frames, started.elapsed(), &mut on_event)
    }
}

fn parse_frames(
    frames: Vec<SseDataFrame>,
    _elapsed: Duration,
    on_event: &mut impl FnMut(ProviderRuntimeStreamEvent),
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
                "provider_runtime_sse_json_parse_failed:{err} data={}",
                frame.data
            )
        })?;

        let envelope: ResponsesEventEnvelope = serde_json::from_value(value.clone())
            .map_err(|err| format!("provider_runtime_sse_envelope_parse_failed:{err}"))?;

        match envelope.event_type.as_str() {
            "response.created" => {
                if let Some(resp) = envelope.response.as_ref() {
                    if let Some(id) = resp.id.as_ref() {
                        response_id = Some(id.clone());
                    }
                }
            }
            "response.output_text.delta" => {
                let delta = envelope.delta.unwrap_or_default();
                if !delta.is_empty() {
                    full_text.push_str(&delta);
                    on_event(ProviderRuntimeStreamEvent::TextDelta(delta));
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

    let response_id =
        response_id.ok_or_else(|| "provider_runtime_missing_response_id".to_string())?;
    on_event(ProviderRuntimeStreamEvent::Completed {
        response_id: response_id.clone(),
    });

    Ok(CompletedResponse {
        response_id,
        full_text,
        tool_calls: tool_calls_ready,
        raw: completed_raw.unwrap_or(Value::Null),
    })
}

fn extract_ready_tool_calls(output: &[ResponseOutputItem]) -> Vec<ToolCallRequest> {
    let mut out = Vec::new();
    for item in output {
        if let ResponseOutputItem::FunctionCall(call) = item {
            if call.status.as_deref() != Some("ready") {
                continue;
            }

            let name = call.name.clone().unwrap_or_default();
            let call_id = call.call_id.clone().unwrap_or_default();
            let arguments_json = call.arguments.clone().unwrap_or_else(|| "{}".to_string());

            if !name.is_empty() && !call_id.is_empty() {
                out.push(ToolCallRequest {
                    name,
                    call_id,
                    arguments_json,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frames_extracts_typed_tool_calls_and_text_deltas() {
        let frames = vec![
            SseDataFrame {
                data: r#"{"type":"response.created","response":{"id":"resp_1"}}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.output_text.delta","delta":"hello "}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.output_text.delta","delta":"world"}"#.to_string(),
            },
            SseDataFrame {
                data: r#"{"type":"response.completed","response":{"id":"resp_1","output":[{"type":"function_call","name":"echo","arguments":"{\"text\":\"hi\"}","call_id":"call_1","status":"ready","vendor_field":"kept"},{"type":"function_call","name":"skip","arguments":"{}","call_id":"call_2","status":"in_progress"},{"type":"message","content":[]}]}}"#.to_string(),
            },
        ];

        let mut events = Vec::new();
        let completed = parse_frames(frames, Duration::from_millis(1), &mut |event| {
            events.push(event);
        })
        .expect("parse provider runtime frames");

        assert_eq!(completed.response_id, "resp_1");
        assert_eq!(completed.full_text, "hello world");
        assert_eq!(completed.tool_calls.len(), 1);
        assert_eq!(completed.tool_calls[0].call_id, "call_1");
        assert_eq!(completed.tool_calls[0].name, "echo");
        assert!(completed.tool_calls[0].arguments_json.contains("\"text\""));
        assert_eq!(
            events,
            vec![
                ProviderRuntimeStreamEvent::TextDelta("hello ".to_string()),
                ProviderRuntimeStreamEvent::TextDelta("world".to_string()),
                ProviderRuntimeStreamEvent::Completed {
                    response_id: "resp_1".to_string(),
                },
            ]
        );
    }
}
