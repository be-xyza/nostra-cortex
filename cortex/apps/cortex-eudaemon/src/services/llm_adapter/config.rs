use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmAdapterFailMode {
    Fallback,
    FailClosed,
}

impl LlmAdapterFailMode {
    pub fn from_env() -> Self {
        match std::env::var("CORTEX_LLM_ADAPTER_FAIL_MODE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("fail_closed") => Self::FailClosed,
            Some("fallback") | None => Self::Fallback,
            Some(_) => Self::Fallback,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlmAdapterConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub request_timeout: Duration,
    pub fail_mode: LlmAdapterFailMode,
    pub max_tool_steps: usize,
    pub default_model: String,
}

fn bool_env(name: &str, default: bool) -> bool {
    match std::env::var(name)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("1") | Some("true") | Some("yes") | Some("on") => true,
        Some("0") | Some("false") | Some("no") | Some("off") => false,
        None => default,
        Some(_) => default,
    }
}

fn parse_u64_env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
        .max(1)
}

fn parse_usize_env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(default)
        .max(1)
}

fn adapter_base_url_default() -> String {
    let host = std::env::var("CORTEX_LLM_ADAPTER_HOST")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = parse_u64_env("CORTEX_LLM_ADAPTER_PORT", 8080);
    format!("http://{}:{}", host, port)
}

fn adapter_api_key_default() -> String {
    if let Ok(value) = std::env::var("CORTEX_LLM_ADAPTER_API_KEY") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    if let Ok(value) = std::env::var("OPENAI_API_KEY") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    "sk-local-dev".to_string()
}

fn normalize_adapter_base_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    trimmed.strip_suffix("/v1").unwrap_or(trimmed).to_string()
}

pub fn llm_adapter_config_from_env() -> LlmAdapterConfig {
    let enabled = bool_env("CORTEX_LLM_ADAPTER_ENABLED", true);
    let base_url = std::env::var("CORTEX_LLM_ADAPTER_URL")
        .ok()
        .map(|value| normalize_adapter_base_url(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| normalize_adapter_base_url(&adapter_base_url_default()));
    let api_key = adapter_api_key_default();
    let timeout_secs = parse_u64_env("CORTEX_LLM_REQUEST_TIMEOUT_SECS", 90);
    let max_tool_steps = parse_usize_env("CORTEX_AGENT_MAX_TOOL_STEPS", 8);
    let default_model = std::env::var("NOSTRA_AGENT_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "llama3.1:8b".to_string());

    LlmAdapterConfig {
        enabled,
        base_url,
        api_key,
        request_timeout: Duration::from_secs(timeout_secs),
        fail_mode: LlmAdapterFailMode::from_env(),
        max_tool_steps,
        default_model,
    }
}
