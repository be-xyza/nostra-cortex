use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProbeRequest {
    pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,
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

    payload
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
        .collect()
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

    let models_url = format!("{canonical_base_url}/models");
    let mut supported_models = Vec::new();
    let mut models_valid = false;
    let mut models_error = None;
    let mut embedding_models = Vec::new();
    match get_json(&client, &models_url, &request.api_key).await {
        Ok((status, body)) if status.is_success() => {
            let parsed = serde_json::from_str::<Value>(&body).ok();
            if let Some(value) = parsed.as_ref() {
                supported_models = extract_supported_models(value);
            }
            models_valid = !supported_models.is_empty() || parsed.is_some();
            if !models_valid {
                models_error = Some("provider_probe_models_empty".to_string());
            }
            if request.validate_embeddings && is_openrouter {
                let embeddings_url = format!("{canonical_base_url}/models/embeddings");
                match get_json(&client, &embeddings_url, &request.api_key).await {
                    Ok((status, body)) if status.is_success() => {
                        if let Ok(value) = serde_json::from_str::<Value>(&body) {
                            embedding_models = extract_supported_models(&value);
                            for model in embedding_models.iter().cloned() {
                                if !supported_models.contains(&model) {
                                    supported_models.push(model);
                                }
                            }
                        }
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
        Ok((status, body)) => {
            models_error = Some(format!("provider_probe_models_http_{status}:{body}"));
        }
        Err(err) => {
            models_error = Some(err);
        }
    }

    if !supported_models.is_empty() {
        supported_models.sort();
        supported_models.dedup();
    }

    let selected_model = request
        .default_model
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            if request.validate_embeddings && !embedding_models.is_empty() {
                embedding_models.first().cloned()
            } else {
                supported_models.first().cloned()
            }
        });

    let mut chat_valid = !request.validate_chat;
    let mut chat_error = None;
    if request.validate_chat {
        if let Some(model) = selected_model.clone() {
            let chat_url = format!("{canonical_base_url}/chat/completions");
            let body = json!({
                "model": model,
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 1,
                "stream": false,
            });
            match post_json(&client, &chat_url, &request.api_key, body).await {
                Ok((status, _body)) if status.is_success() => {
                    chat_valid = true;
                }
                Ok((status, body)) => {
                    chat_valid = false;
                    chat_error = Some(format!("provider_probe_chat_http_{status}:{body}"));
                }
                Err(err) => {
                    chat_valid = false;
                    chat_error = Some(err);
                }
            }
        } else {
            chat_valid = false;
            chat_error = Some("provider_probe_missing_model_for_chat".to_string());
        }
    }

    let mut embeddings_valid = !request.validate_embeddings;
    let mut embeddings_error = None;
    if request.validate_embeddings {
        if let Some(model) = selected_model.clone() {
            let embeddings_url = format!("{canonical_base_url}/embeddings");
            let body = json!({
                "model": model,
                "input": "ping",
            });
            match post_json(&client, &embeddings_url, &request.api_key, body).await {
                Ok((status, _body)) if status.is_success() => {
                    embeddings_valid = true;
                }
                Ok((status, body)) => {
                    embeddings_valid = false;
                    embeddings_error = Some(format!("provider_probe_embeddings_http_{status}:{body}"));
                }
                Err(err) => {
                    embeddings_valid = false;
                    embeddings_error = Some(err);
                }
            }
        } else {
            embeddings_valid = false;
            embeddings_error = Some("provider_probe_missing_model_for_embeddings".to_string());
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
    use super::extract_supported_models;
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
}
