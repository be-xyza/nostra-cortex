use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, sync::Arc, time::Duration};

const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_LOCAL_MODEL: &str = "llama3.1:8b";

#[derive(Clone)]
struct LiveGenerationState {
    client: reqwest::Client,
    settings: Arc<LiveGenerationSettings>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveGenerationSettings {
    pub base_url: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub auth_configured: bool,
}

#[derive(Debug, Deserialize)]
pub struct GroundedGenerationRequest {
    pub question: String,
    #[serde(default)]
    pub contexts: Vec<String>,
    #[serde(default)]
    pub strict_grounding: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GroundedGenerationResponse {
    pub model: String,
    pub answer: String,
}

#[derive(Debug, Serialize)]
struct ModelHealthResponse {
    status: String,
    llm_base: String,
    generation_model: String,
    auth_configured: bool,
}

pub fn live_generation_enabled() -> bool {
    env_flag("NOSTRA_WORKER_LIVE_GENERATION_ENABLED")
}

pub fn live_generation_port() -> u16 {
    env::var("NOSTRA_WORKER_LIVE_GENERATION_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3003)
}

pub async fn start_live_generation_server(port: u16) -> anyhow::Result<()> {
    let settings = Arc::new(resolve_live_generation_settings());
    let state = LiveGenerationState {
        client: reqwest::Client::new(),
        settings,
    };
    let app = Router::new()
        .route("/health/model", get(get_model_health))
        .route("/generation/grounded", axum::routing::post(post_grounded_generation))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    println!("   > Live generation API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_model_health(State(state): State<LiveGenerationState>) -> impl IntoResponse {
    Json(ModelHealthResponse {
        status: "configured".to_string(),
        llm_base: state.settings.base_url.clone(),
        generation_model: state.settings.model.clone(),
        auth_configured: state.settings.auth_configured,
    })
}

async fn post_grounded_generation(
    State(state): State<LiveGenerationState>,
    Json(body): Json<GroundedGenerationRequest>,
) -> impl IntoResponse {
    if body.question.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "question must not be empty").into_response();
    }

    match generate_grounded_answer(&state.client, &state.settings, &body).await {
        Ok(answer) => (
            StatusCode::OK,
            Json(GroundedGenerationResponse {
                model: state.settings.model.clone(),
                answer,
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            format!("generation request failed: {error}"),
        )
            .into_response(),
    }
}

async fn generate_grounded_answer(
    client: &reqwest::Client,
    settings: &LiveGenerationSettings,
    body: &GroundedGenerationRequest,
) -> Result<String, String> {
    let system_prompt = if body.strict_grounding.unwrap_or(true) {
        "You are a grounded knowledge assistant. Use only the provided context. If context is insufficient, explicitly say so. Cite chunk IDs in brackets like [chunk-id]."
    } else {
        "You are a knowledge assistant. Prefer provided context and cite chunk IDs when possible."
    };
    let user_prompt = format!(
        "Question:\n{}\n\nContext:\n{}\n\nReturn concise answer with citations.",
        body.question,
        body.contexts.join("\n\n---\n\n")
    );
    let payload = json!({
        "model": settings.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "stream": false
    });

    let mut request = client.post(generation_chat_url(&settings.base_url)).json(&payload);
    if let Some(api_key) = generation_api_key() {
        request = request.bearer_auth(api_key);
    }

    let response = tokio::time::timeout(Duration::from_secs(settings.timeout_seconds), request.send())
        .await
        .map_err(|_| "timeout".to_string())
        .and_then(|response| response.map_err(|error| error.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("provider status {status}: {body}"));
    }

    let value = response
        .json::<serde_json::Value>()
        .await
        .map_err(|error| format!("provider response parse error: {error}"))?;
    let answer = extract_generation_answer(&value);
    if answer.is_empty() {
        return Err("provider returned an empty answer".to_string());
    }
    Ok(answer)
}

fn resolve_live_generation_settings() -> LiveGenerationSettings {
    LiveGenerationSettings {
        base_url: generation_base_url(),
        model: generation_model_from_values(
            env::var("NOSTRA_LLM_MODEL").ok().as_deref(),
            env::var("NOSTRA_LOCAL_GENERATION_MODEL").ok().as_deref(),
        ),
        timeout_seconds: env::var("NOSTRA_LLM_TIMEOUT_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(40),
        auth_configured: generation_api_key().is_some(),
    }
}

fn generation_base_url() -> String {
    env::var("NOSTRA_LLM_BASE_URL")
        .or_else(|_| env::var("NOSTRA_LLM_API_BASE"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
        .unwrap_or_else(|| DEFAULT_OLLAMA_BASE_URL.to_string())
}

fn generation_model_from_values(llm_model: Option<&str>, local_model: Option<&str>) -> String {
    llm_model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            local_model
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or(DEFAULT_LOCAL_MODEL)
        .to_string()
}

fn generation_api_key() -> Option<String> {
    env::var("NOSTRA_LLM_API_KEY")
        .or_else(|_| env::var("OPENROUTER_API_KEY"))
        .or_else(|_| env::var("OPENAI_API_KEY"))
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn generation_chat_url(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") || trimmed.ends_with("/api/chat") {
        trimmed.to_string()
    } else if trimmed.contains("openrouter.ai") || trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/api/chat")
    }
}

fn extract_generation_answer(value: &serde_json::Value) -> String {
    value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
        })
        .or_else(|| value.get("response").and_then(|content| content.as_str()))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn generation_model_prefers_live_provider_env() {
        assert_eq!(
            generation_model_from_values(Some(" ~moonshotai/kimi-latest "), Some("llama3.1:8b")),
            "~moonshotai/kimi-latest"
        );
        assert_eq!(
            generation_model_from_values(Some(""), Some("llama3.1:8b")),
            "llama3.1:8b"
        );
        assert_eq!(generation_model_from_values(None, None), "llama3.1:8b");
    }

    #[test]
    fn generation_chat_url_supports_openrouter_and_ollama_shapes() {
        assert_eq!(
            generation_chat_url("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            generation_chat_url("https://openrouter.ai/api/v1/chat/completions"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            generation_chat_url("http://localhost:11434"),
            "http://localhost:11434/api/chat"
        );
    }

    #[test]
    fn extract_generation_answer_supports_openai_and_ollama_payloads() {
        let openai_like = json!({
            "choices": [
                {
                    "message": {
                        "content": " Kimi answer "
                    }
                }
            ]
        });
        let ollama_chat = json!({
            "message": {
                "content": " Ollama chat answer "
            }
        });
        let ollama_generate = json!({
            "response": " Ollama generate answer "
        });

        assert_eq!(extract_generation_answer(&openai_like), "Kimi answer");
        assert_eq!(extract_generation_answer(&ollama_chat), "Ollama chat answer");
        assert_eq!(
            extract_generation_answer(&ollama_generate),
            "Ollama generate answer"
        );
    }
}
