use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

const CHAT_PROBE_MIN_MAX_TOKENS: u32 = 16;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProbeRequest {
    pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_binding_id: Option<String>,
    #[serde(default)]
    pub use_stored_auth: bool,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub validate_key: bool,
    #[serde(default)]
    pub validate_chat: bool,
    #[serde(default)]
    pub validate_embeddings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProbeResponse {
    pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,
    pub endpoint: String,
    pub canonical_base_url: String,
    pub validate_key: bool,
    pub validate_chat: bool,
    pub validate_embeddings: bool,
    pub key_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_info: Option<Value>,
    pub models_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models_error: Option<String>,
    pub chat_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_error: Option<String>,
    pub embeddings_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeddings_error: Option<String>,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
    pub valid: bool,
    pub discovered_at: String,
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_string()
}

fn is_ollama_probe(request: &ProviderProbeRequest, canonical_base_url: &str) -> bool {
    request
        .provider_kind
        .as_ref()
        .map(|value| value.eq_ignore_ascii_case("ollama"))
        .unwrap_or(false)
        || canonical_base_url.contains("11434")
        || canonical_base_url.to_ascii_lowercase().contains("ollama")
}

fn candidate_endpoint_urls(base_url: &str, suffixes: &[&str]) -> Vec<String> {
    let base = base_url.trim_end_matches('/');
    let mut urls = Vec::new();
    for suffix in suffixes {
        let url = format!("{base}/{}", suffix.trim_start_matches('/'));
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
    urls
}

fn auth_header(api_key: &str) -> String {
    format!("Bearer {}", api_key.trim())
}

async fn get_json(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
) -> Result<(reqwest::StatusCode, String), String> {
    let response = client
        .get(url)
        .header("Authorization", auth_header(api_key))
        .send()
        .await
        .map_err(|err| format!("provider_probe_request_failed:{err}"))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Ok((status, body))
}

async fn post_json(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
    body: Value,
) -> Result<(reqwest::StatusCode, String), String> {
    let response = client
        .post(url)
        .header("Authorization", auth_header(api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("provider_probe_request_failed:{err}"))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    Ok((status, text))
}

fn extract_supported_models(payload: &Value) -> Vec<String> {
    if let Some(rows) = payload.as_array() {
        return rows
            .iter()
            .filter_map(|entry| {
                entry
                    .get("id")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .or_else(|| {
                        entry
                            .get("name")
                            .and_then(|value| value.as_str())
                            .map(str::to_string)
                    })
            })
            .collect();
    }

    let mut models = payload
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .or_else(|| {
                    entry
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                })
        })
        .collect::<Vec<_>>();

    if let Some(rows) = payload.get("models").and_then(Value::as_array) {
        for entry in rows {
            if let Some(model) = entry
                .get("id")
                .and_then(|value| value.as_str())
                .or_else(|| entry.get("name").and_then(|value| value.as_str()))
                .or_else(|| entry.get("model").and_then(|value| value.as_str()))
            {
                models.push(model.to_string());
            }
        }
    }

    models
}

fn preferred_probe_models(default_model: Option<&str>, supported_models: &[String]) -> Vec<String> {
    let mut models = Vec::new();

    if let Some(model) = default_model.map(str::trim).filter(|value| !value.is_empty()) {
        models.push(model.to_string());
    }

    for model in supported_models {
        let candidate = model.trim();
        if candidate.is_empty() || models.iter().any(|existing| existing == candidate) {
            continue;
        }
        models.push(candidate.to_string());
    }

    models
}

fn build_chat_probe_body(model: &str) -> Value {
    json!({
        "model": model,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": CHAT_PROBE_MIN_MAX_TOKENS,
        "stream": false,
    })
}

pub async fn validate_provider_probe(request: ProviderProbeRequest) -> ProviderProbeResponse {
    let canonical_base_url = normalize_base_url(&request.base_url);
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return ProviderProbeResponse {
                provider_type: request.provider_type,
                provider_kind: request.provider_kind,
                endpoint: request.base_url,
                canonical_base_url,
                validate_key: request.validate_key,
                validate_chat: request.validate_chat,
                validate_embeddings: request.validate_embeddings,
                key_error: Some(format!("provider_probe_client_init_failed:{err}")),
                models_error: Some(format!("provider_probe_client_init_failed:{err}")),
                chat_error: Some(format!("provider_probe_client_init_failed:{err}")),
                embeddings_error: Some(format!("provider_probe_client_init_failed:{err}")),
                supported_models: Vec::new(),
                valid: false,
                discovered_at: crate::services::cortex_ux::now_iso(),
                ..Default::default()
            };
        }
    };

    let mut key_valid = !request.validate_key;
    let mut key_error = None;
    let mut key_info = None;
    let is_openrouter = request
        .provider_kind
        .as_ref()
        .map(|value| value.eq_ignore_ascii_case("openrouter"))
        .unwrap_or_else(|| canonical_base_url.contains("openrouter.ai"));
    if request.validate_key && is_openrouter {
        let key_url = format!("{canonical_base_url}/key");
        match get_json(&client, &key_url, &request.api_key).await {
            Ok((status, body)) if status.is_success() => {
                key_valid = true;
                key_info = serde_json::from_str::<Value>(&body).ok();
            }
            Ok((status, body)) => {
                key_valid = false;
                key_error = Some(format!("provider_probe_key_http_{status}:{body}"));
            }
            Err(err) => {
                key_valid = false;
                key_error = Some(err);
            }
        }
    }

    let mut supported_models = Vec::new();
    let mut models_valid = false;
    let mut models_error = None;
    let mut model_urls = candidate_endpoint_urls(&canonical_base_url, &["models", "v1/models"]);
    if is_ollama_probe(&request, &canonical_base_url) {
        model_urls.push(format!("{}/api/tags", canonical_base_url.trim_end_matches('/')));
    }
    for models_url in model_urls {
        match get_json(&client, &models_url, &request.api_key).await {
            Ok((status, body)) if status.is_success() => {
                let parsed = serde_json::from_str::<Value>(&body).ok();
                if let Some(value) = parsed.as_ref() {
                    supported_models = extract_supported_models(value);
                }
                models_valid = !supported_models.is_empty() || parsed.is_some();
                if models_valid {
                    models_error = None;
                    break;
                }
                models_error = Some("provider_probe_models_empty".to_string());
            }
            Ok((status, body)) => {
                models_error = Some(format!("provider_probe_models_http_{status}:{body}"));
            }
            Err(err) => {
                models_error = Some(err);
            }
        }
    }
    if models_valid && request.validate_embeddings && is_openrouter {
        let embeddings_catalog_urls =
            candidate_endpoint_urls(&canonical_base_url, &["models/embeddings", "v1/models/embeddings"]);
        for embeddings_url in embeddings_catalog_urls {
            match get_json(&client, &embeddings_url, &request.api_key).await {
                Ok((status, body)) if status.is_success() => {
                    if let Ok(value) = serde_json::from_str::<Value>(&body) {
                        for model in extract_supported_models(&value).into_iter() {
                            if !supported_models.contains(&model) {
                                supported_models.push(model);
                            }
                        }
                    }
                    break;
                }
                Ok((status, body)) => {
                    models_error = Some(format!("provider_probe_embeddings_models_http_{status}:{body}"));
                }
                Err(err) => {
                    models_error = Some(err);
                }
            }
        }
    }

    if !supported_models.is_empty() {
        supported_models.sort();
        supported_models.dedup();
    }

    let preferred_models = preferred_probe_models(request.default_model.as_deref(), &supported_models);
    let mut selected_model = preferred_models.first().cloned();

    let mut chat_valid = !request.validate_chat;
    let mut chat_error = None;
    let mut embeddings_valid = !request.validate_embeddings;
    let mut embeddings_error = None;
    if request.validate_chat || request.validate_embeddings {
        if preferred_models.is_empty() {
            if request.validate_chat {
                chat_valid = false;
                chat_error = Some("provider_probe_missing_model_for_chat".to_string());
            }
            if request.validate_embeddings {
                embeddings_valid = false;
                embeddings_error = Some("provider_probe_missing_model_for_embeddings".to_string());
            }
        } else {
            for model in &preferred_models {
                let mut candidate_chat_valid = !request.validate_chat;
                let mut candidate_chat_error = None;
                if request.validate_chat {
                    let body = build_chat_probe_body(model);
                    for chat_url in candidate_endpoint_urls(
                        &canonical_base_url,
                        &["chat/completions", "v1/chat/completions"],
                    ) {
                        match post_json(&client, &chat_url, &request.api_key, body.clone()).await {
                            Ok((status, _body)) if status.is_success() => {
                                candidate_chat_valid = true;
                                candidate_chat_error = None;
                                break;
                            }
                            Ok((status, body)) => {
                                candidate_chat_valid = false;
                                candidate_chat_error = Some(format!("provider_probe_chat_http_{status}:{body}"));
                            }
                            Err(err) => {
                                candidate_chat_valid = false;
                                candidate_chat_error = Some(err);
                            }
                        }
                    }
                }

                let mut candidate_embeddings_valid = !request.validate_embeddings;
                let mut candidate_embeddings_error = None;
                if request.validate_embeddings {
                    let body = json!({
                        "model": model,
                        "input": "ping",
                    });
                    for embeddings_url in candidate_endpoint_urls(
                        &canonical_base_url,
                        &["embeddings", "v1/embeddings"],
                    ) {
                        match post_json(&client, &embeddings_url, &request.api_key, body.clone()).await {
                            Ok((status, _body)) if status.is_success() => {
                                candidate_embeddings_valid = true;
                                candidate_embeddings_error = None;
                                break;
                            }
                            Ok((status, body)) => {
                                candidate_embeddings_valid = false;
                                candidate_embeddings_error =
                                    Some(format!("provider_probe_embeddings_http_{status}:{body}"));
                            }
                            Err(err) => {
                                candidate_embeddings_valid = false;
                                candidate_embeddings_error = Some(err);
                            }
                        }
                    }
                }

                chat_valid = candidate_chat_valid;
                chat_error = candidate_chat_error;
                embeddings_valid = candidate_embeddings_valid;
                embeddings_error = candidate_embeddings_error;

                if candidate_chat_valid && candidate_embeddings_valid {
                    selected_model = Some(model.clone());
                    break;
                }
            }
        }
    }

    let valid = (!request.validate_key || key_valid)
        && models_valid
        && chat_valid
        && embeddings_valid;

    ProviderProbeResponse {
        provider_type: request.provider_type,
        provider_kind: request.provider_kind,
        endpoint: request.base_url,
        canonical_base_url,
        validate_key: request.validate_key,
        validate_chat: request.validate_chat,
        validate_embeddings: request.validate_embeddings,
        key_valid,
        key_error,
        key_info,
        models_valid,
        models_error,
        chat_valid,
        chat_error,
        embeddings_valid,
        embeddings_error,
        supported_models,
        selected_model,
        valid,
        discovered_at: crate::services::cortex_ux::now_iso(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_chat_probe_body,
        extract_supported_models,
        preferred_probe_models,
        CHAT_PROBE_MIN_MAX_TOKENS,
    };
    use serde_json::json;

    #[test]
    fn extracts_supported_models_from_openrouter_style_payload() {
        let payload = json!({
            "data": [
                { "id": "openai/gpt-5.2" },
                { "id": "openai/gpt-4.1" },
                { "name": "fallback-name" }
            ]
        });

        let models = extract_supported_models(&payload);
        assert_eq!(models, vec!["openai/gpt-5.2", "openai/gpt-4.1", "fallback-name"]);
    }

    #[test]
    fn extracts_supported_models_from_array_payload() {
        let payload = json!([
            { "id": "model-a" },
            { "name": "model-b" }
        ]);

        let models = extract_supported_models(&payload);
        assert_eq!(models, vec!["model-a", "model-b"]);
    }

    #[test]
    fn extracts_supported_models_from_ollama_models_payload() {
        let payload = json!({
            "models": [
                { "name": "llama3.2:8b" },
                { "model": "qwen2.5-coder" }
            ]
        });

        let models = extract_supported_models(&payload);
        assert_eq!(models, vec!["llama3.2:8b", "qwen2.5-coder"]);
    }

    #[test]
    fn prefers_explicit_default_model_before_discovered_models() {
        let candidates = preferred_probe_models(
            Some("llama3.1:8b"),
            &[
                "openai/gpt-5.4".to_string(),
                "llama3.1:8b".to_string(),
                "openai/gpt-4.1".to_string(),
            ],
        );

        assert_eq!(candidates, vec!["llama3.1:8b", "openai/gpt-5.4", "openai/gpt-4.1"]);
    }

    #[test]
    fn chat_probe_uses_provider_accepted_minimum_token_cap() {
        let body = build_chat_probe_body("openai/gpt-5.4");

        assert_eq!(body["max_tokens"], CHAT_PROBE_MIN_MAX_TOKENS);
        assert_eq!(body["model"], "openai/gpt-5.4");
    }
}
