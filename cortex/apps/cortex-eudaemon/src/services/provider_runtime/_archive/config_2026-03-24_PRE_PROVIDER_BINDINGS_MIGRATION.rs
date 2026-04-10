use crate::services::cortex_ux::now_iso;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_PROVIDER_ID: &str = "llm_adapter";

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ProviderLocalityKind {
    Local,
    Tunneled,
    Cloud,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchCadenceKind {
    Immediate,
    Interval,
    TimeWindow,
    Scoped,
    Manual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchScopeKind {
    ProviderFamily,
    ProviderProfile,
    Space,
    Agent,
    Session,
    RequestGroup,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchFlushPolicy {
    OnInterval,
    OnWindowClose,
    OnThreshold,
    OnIdle,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBatchWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBatchPolicy {
    pub provider_family_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_profile_id: Option<String>,
    pub cadence_kind: ProviderBatchCadenceKind,
    pub scope_kind: ProviderBatchScopeKind,
    pub flush_policy: ProviderBatchFlushPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordering_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedupe_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_window: Option<ProviderBatchWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRuntimeContext {
    pub family_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    pub instance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    pub locality_kind: ProviderLocalityKind,
    pub last_seen_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentialBinding {
    pub credential_binding_id: String,
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_source: Option<String>,
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRuntimeSettings {
    pub provider_id: String,
    pub name: String,
    #[serde(default = "default_provider_type")]
    pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,
    pub enabled: bool,
    pub base_url: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_family_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locality_kind: Option<ProviderLocalityKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_policy: Option<ProviderBatchPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRegistryState {
    #[serde(default)]
    pub providers: Vec<ProviderRuntimeSettings>,
    #[serde(default)]
    pub credentials: Vec<ProviderCredentialBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedProviderRuntimeState {
    pub provider_id: String,
    pub name: String,
    pub provider_kind: String,
    pub enabled: bool,
    pub base_url: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_binding_id: Option<String>,
    pub has_credential: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_source: Option<String>,
    pub runtime_context: ProviderRuntimeContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_policy: Option<ProviderBatchPolicy>,
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

fn default_provider_type() -> String {
    "Llm".to_string()
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

fn trimmed_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn sanitize_identifier_component(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut last_was_separator = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            output.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else {
            if !last_was_separator {
                output.push('_');
                last_was_separator = true;
            }
        }
    }
    let collapsed = output.trim_matches('_').to_string();
    if collapsed.is_empty() {
        "unknown".to_string()
    } else {
        collapsed
    }
}

fn default_provider_family_id(provider_kind: &str, provider_id: &str) -> String {
    let normalized = provider_kind.trim();
    if normalized.is_empty() {
        return sanitize_identifier_component(provider_id);
    }

    let slug = normalized
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    sanitize_identifier_component(slug.trim_matches('-'))
}

fn default_provider_instance_id(provider_id: &str, base_url: &str) -> String {
    format!(
        "{}__{}",
        sanitize_identifier_component(provider_id),
        sanitize_identifier_component(base_url)
    )
}

fn parse_provider_locality_kind(value: &str) -> Option<ProviderLocalityKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "local" => Some(ProviderLocalityKind::Local),
        "sovereign" => Some(ProviderLocalityKind::Local),
        "tunneled" => Some(ProviderLocalityKind::Tunneled),
        "cloud" => Some(ProviderLocalityKind::Cloud),
        _ => None,
    }
}

fn parse_provider_batch_cadence_kind(value: &str) -> Option<ProviderBatchCadenceKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "immediate" => Some(ProviderBatchCadenceKind::Immediate),
        "interval" => Some(ProviderBatchCadenceKind::Interval),
        "timewindow" | "time_window" | "window" => Some(ProviderBatchCadenceKind::TimeWindow),
        "scoped" => Some(ProviderBatchCadenceKind::Scoped),
        "manual" => Some(ProviderBatchCadenceKind::Manual),
        _ => None,
    }
}

fn parse_provider_batch_scope_kind(value: &str) -> Option<ProviderBatchScopeKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "providerfamily" | "provider_family" => Some(ProviderBatchScopeKind::ProviderFamily),
        "providerprofile" | "provider_profile" => Some(ProviderBatchScopeKind::ProviderProfile),
        "space" => Some(ProviderBatchScopeKind::Space),
        "agent" => Some(ProviderBatchScopeKind::Agent),
        "session" => Some(ProviderBatchScopeKind::Session),
        "requestgroup" | "request_group" => Some(ProviderBatchScopeKind::RequestGroup),
        _ => None,
    }
}

fn parse_provider_batch_flush_policy(value: &str) -> Option<ProviderBatchFlushPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "oninterval" | "on_interval" => Some(ProviderBatchFlushPolicy::OnInterval),
        "onwindowclose" | "on_window_close" => Some(ProviderBatchFlushPolicy::OnWindowClose),
        "onthreshold" | "on_threshold" => Some(ProviderBatchFlushPolicy::OnThreshold),
        "onidle" | "on_idle" => Some(ProviderBatchFlushPolicy::OnIdle),
        "manual" => Some(ProviderBatchFlushPolicy::Manual),
        _ => None,
    }
}

pub fn infer_provider_locality_kind(
    base_url: &str,
    device_id: Option<&str>,
    environment_id: Option<&str>,
) -> ProviderLocalityKind {
    if let Some(explicit) = trimmed_env_locality_kind() {
        return explicit;
    }

    if let (Some(device), Some(environment)) = (device_id, environment_id) {
        let device = device.trim();
        let environment = environment.trim();
        if !device.is_empty() && !environment.is_empty() && device != environment {
            return ProviderLocalityKind::Tunneled;
        }
    }

    let normalized_url = base_url.to_ascii_lowercase();
    if normalized_url.contains("localhost")
        || normalized_url.contains("127.0.0.1")
        || normalized_url.contains("::1")
    {
        ProviderLocalityKind::Local
    } else {
        ProviderLocalityKind::Cloud
    }
}

fn trimmed_env_locality_kind() -> Option<ProviderLocalityKind> {
    trimmed_env("CORTEX_PROVIDER_LOCALITY_KIND")
        .and_then(|value| parse_provider_locality_kind(&value))
}

fn discovery_source_from_env() -> Option<String> {
    trimmed_env("CORTEX_PROVIDER_DISCOVERY_SOURCE")
}

fn build_runtime_context(
    provider_id: &str,
    provider_kind: &str,
    base_url: &str,
    default_model: &str,
    device_id: Option<String>,
    environment_id: Option<String>,
    locality_kind: Option<ProviderLocalityKind>,
    discovery_source: Option<String>,
    last_seen_at: String,
) -> ProviderRuntimeContext {
    let family_id = trimmed_env("CORTEX_PROVIDER_FAMILY_ID")
        .unwrap_or_else(|| default_provider_family_id(provider_kind, provider_id));
    let profile_id = trimmed_env("CORTEX_PROVIDER_PROFILE_ID")
        .or_else(|| Some(default_model.trim().to_string()).filter(|value| !value.is_empty()));
    let instance_id = trimmed_env("CORTEX_PROVIDER_INSTANCE_ID")
        .unwrap_or_else(|| default_provider_instance_id(provider_id, base_url));
    let resolved_device_id = device_id.or_else(|| trimmed_env("CORTEX_PROVIDER_DEVICE_ID"));
    let resolved_environment_id = environment_id.or_else(|| trimmed_env("CORTEX_PROVIDER_ENVIRONMENT_ID"));
    let resolved_locality_kind = locality_kind
        .or_else(|| trimmed_env("CORTEX_PROVIDER_LOCALITY_KIND").and_then(|value| parse_provider_locality_kind(&value)))
        .unwrap_or_else(|| {
            infer_provider_locality_kind(
                base_url,
                resolved_device_id.as_deref(),
                resolved_environment_id.as_deref(),
            )
        });
    let resolved_discovery_source = discovery_source
        .or_else(|| discovery_source_from_env())
        .or_else(|| {
            Some(if resolved_device_id.is_some() || resolved_environment_id.is_some() {
                "environment"
            } else {
                "heuristic"
            }
            .to_string())
        });

    ProviderRuntimeContext {
        family_id,
        profile_id,
        instance_id,
        device_id: resolved_device_id,
        environment_id: resolved_environment_id,
        locality_kind: resolved_locality_kind,
        last_seen_at,
        discovery_source: resolved_discovery_source,
    }
}

fn build_provider_batch_policy(
    provider_kind: &str,
    provider_id: &str,
    default_model: &str,
    _base_url: &str,
) -> Option<ProviderBatchPolicy> {
    let provider_kind = provider_kind.trim();
    let batch_enabled = trimmed_env("CORTEX_PROVIDER_BATCH_ENABLED")
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or_else(|| provider_kind.eq_ignore_ascii_case("doubleword"));

    if !batch_enabled {
        return None;
    }

    let provider_family_id = trimmed_env("CORTEX_PROVIDER_BATCH_FAMILY_ID")
        .unwrap_or_else(|| default_provider_family_id(provider_kind, provider_id));
    let provider_profile_id = trimmed_env("CORTEX_PROVIDER_BATCH_PROFILE_ID")
        .or_else(|| Some(default_model.trim().to_string()).filter(|value| !value.is_empty()));
    let cadence_kind = trimmed_env("CORTEX_PROVIDER_BATCH_CADENCE_KIND")
        .and_then(|value| parse_provider_batch_cadence_kind(&value))
        .unwrap_or_else(|| {
            if provider_kind.eq_ignore_ascii_case("doubleword") {
                ProviderBatchCadenceKind::Interval
            } else {
                ProviderBatchCadenceKind::Scoped
            }
        });
    let scope_kind = trimmed_env("CORTEX_PROVIDER_BATCH_SCOPE_KIND")
        .and_then(|value| parse_provider_batch_scope_kind(&value))
        .unwrap_or_else(|| {
            if provider_kind.eq_ignore_ascii_case("doubleword") {
                ProviderBatchScopeKind::RequestGroup
            } else {
                ProviderBatchScopeKind::ProviderFamily
            }
        });
    let flush_policy = trimmed_env("CORTEX_PROVIDER_BATCH_FLUSH_POLICY")
        .and_then(|value| parse_provider_batch_flush_policy(&value))
        .unwrap_or_else(|| match cadence_kind {
            ProviderBatchCadenceKind::Immediate => ProviderBatchFlushPolicy::Manual,
            ProviderBatchCadenceKind::Interval => ProviderBatchFlushPolicy::OnInterval,
            ProviderBatchCadenceKind::TimeWindow => ProviderBatchFlushPolicy::OnWindowClose,
            ProviderBatchCadenceKind::Scoped => ProviderBatchFlushPolicy::OnThreshold,
            ProviderBatchCadenceKind::Manual => ProviderBatchFlushPolicy::Manual,
        });
    let ordering_key = trimmed_env("CORTEX_PROVIDER_BATCH_ORDERING_KEY")
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| "request_group_id".to_string()));
    let dedupe_key = trimmed_env("CORTEX_PROVIDER_BATCH_DEDUPE_KEY")
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| "request_hash".to_string()));
    let interval_seconds = trimmed_env("CORTEX_PROVIDER_BATCH_INTERVAL_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| 60));
    let max_items = trimmed_env("CORTEX_PROVIDER_BATCH_MAX_ITEMS")
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| 100));
    let max_age_seconds = trimmed_env("CORTEX_PROVIDER_BATCH_MAX_AGE_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| 600));
    let timezone = trimmed_env("CORTEX_PROVIDER_BATCH_TIMEZONE")
        .or_else(|| provider_kind.eq_ignore_ascii_case("doubleword").then(|| "UTC".to_string()));

    let batch_window = if interval_seconds.is_some()
        || max_items.is_some()
        || max_age_seconds.is_some()
        || timezone.is_some()
    {
        Some(ProviderBatchWindow {
            interval_seconds,
            max_items,
            max_age_seconds,
            timezone,
        })
    } else {
        None
    };

    if batch_window.is_none()
        && ordering_key.is_none()
        && dedupe_key.is_none()
        && provider_kind.is_empty()
    {
        return None;
    }

    Some(ProviderBatchPolicy {
        provider_family_id,
        provider_profile_id,
        cadence_kind,
        scope_kind,
        flush_policy,
        ordering_key,
        dedupe_key,
        batch_window,
    })
}

pub fn build_provider_runtime_settings_from_env() -> ProviderRuntimeSettings {
    let enabled = bool_env("CORTEX_LLM_ADAPTER_ENABLED", true);
    let base_url = std::env::var("CORTEX_LLM_ADAPTER_URL")
        .ok()
        .map(|value| normalize_adapter_base_url(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| normalize_adapter_base_url(&adapter_base_url_default()));
    let default_model = std::env::var("NOSTRA_AGENT_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "llama3.1:8b".to_string());
    let provider_kind = infer_provider_kind(&base_url, &default_model);
    let device_id = trimmed_env("CORTEX_PROVIDER_DEVICE_ID");
    let environment_id = trimmed_env("CORTEX_PROVIDER_ENVIRONMENT_ID");
    let locality_kind = Some(infer_provider_locality_kind(
        &base_url,
        device_id.as_deref(),
        environment_id.as_deref(),
    ));
    let discovery_source = discovery_source_from_env();
    let batch_policy = build_provider_batch_policy(&provider_kind, DEFAULT_PROVIDER_ID, &default_model, &base_url);

    ProviderRuntimeSettings {
        provider_id: DEFAULT_PROVIDER_ID.to_string(),
        name: "Primary LLM Adapter".to_string(),
        provider_type: default_provider_type(),
        provider_kind: Some(provider_kind),
        enabled,
        base_url,
        default_model: default_model.clone(),
        adapter_set_ref: Some("adapter.default".to_string()),
        credential_binding_id: None,
        provider_family_id: trimmed_env("CORTEX_PROVIDER_FAMILY_ID"),
        profile_id: trimmed_env("CORTEX_PROVIDER_PROFILE_ID"),
        instance_id: trimmed_env("CORTEX_PROVIDER_INSTANCE_ID"),
        device_id,
        environment_id,
        locality_kind,
        discovery_source,
        batch_policy,
        updated_at: None,
        supported_models: vec![default_model.clone()],
        metadata: std::collections::BTreeMap::new(),
    }
}

fn provider_registry_state_path() -> PathBuf {
    if let Ok(value) = std::env::var("CORTEX_PROVIDER_STATE_PATH") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(value) = std::env::var("NOSTRA_WORKSPACE_ROOT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed)
                .join("_system")
                .join("providers.json");
        }
    }

    PathBuf::from("_system").join("providers.json")
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    let tmp_path = PathBuf::from(format!("{}.tmp", path.display()));
    let mut file = fs::File::create(&tmp_path).map_err(|err| err.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|err| err.to_string())?;
    file.sync_all().map_err(|err| err.to_string())?;
    fs::rename(&tmp_path, path).map_err(|err| err.to_string())
}

pub fn infer_provider_kind(base_url: &str, default_model: &str) -> String {
    let normalized_url = base_url.to_ascii_lowercase();
    let normalized_model = default_model.to_ascii_lowercase();
    if normalized_url.contains("anthropic") || normalized_model.contains("claude") {
        "Anthropic".to_string()
    } else if normalized_url.contains("doubleword") || normalized_model.contains("doubleword") {
        "DoubleWord".to_string()
    } else if normalized_url.contains("openrouter") {
        "OpenRouter".to_string()
    } else if normalized_url.contains("ollama")
        || normalized_url.contains("127.0.0.1")
        || normalized_model.contains("llama")
    {
        "Ollama".to_string()
    } else {
        "OpenAI".to_string()
    }
}

pub fn load_provider_registry_state() -> Result<ProviderRegistryState, String> {
    let path = provider_registry_state_path();
    if !path.exists() {
        return Ok(ProviderRegistryState::default());
    }
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

pub fn save_provider_registry_state(state: &ProviderRegistryState) -> Result<(), String> {
    save_json(&provider_registry_state_path(), state)
}

pub type ProviderCredentialBindingRecord = ProviderCredentialBinding;
pub type ProviderSettingsRecord = ProviderRuntimeSettings;
pub type ProviderSettingsStore = ProviderRegistryState;

pub fn load_provider_settings_store() -> Result<ProviderSettingsStore, String> {
    load_provider_registry_state()
}

pub fn save_provider_settings_store(store: &ProviderSettingsStore) -> Result<(), String> {
    save_provider_registry_state(store)
}

pub fn upsert_provider_runtime_settings(settings: ProviderRuntimeSettings) -> Result<(), String> {
    let mut state = load_provider_registry_state()?;
    if let Some(existing) = state
        .providers
        .iter_mut()
        .find(|item| item.provider_id == settings.provider_id)
    {
        *existing = settings;
    } else {
        state.providers.push(settings);
    }
    save_provider_registry_state(&state)
}

pub fn upsert_provider_credential_binding(
    binding: ProviderCredentialBinding,
) -> Result<(), String> {
    let mut state = load_provider_registry_state()?;
    if let Some(existing) = state
        .credentials
        .iter_mut()
        .find(|item| item.credential_binding_id == binding.credential_binding_id)
    {
        *existing = binding;
    } else {
        state.credentials.push(binding);
    }
    save_provider_registry_state(&state)
}

fn provider_runtime_settings_from_env() -> ProviderRuntimeSettings {
    build_provider_runtime_settings_from_env()
}

pub fn resolve_provider_runtime_state() -> ResolvedProviderRuntimeState {
    let env_settings = provider_runtime_settings_from_env();
    let loaded = load_provider_registry_state().ok();
    let stored = loaded
        .as_ref()
        .and_then(|state| {
            state
                .providers
                .iter()
                .find(|item| item.provider_id == DEFAULT_PROVIDER_ID)
                .cloned()
        })
        .unwrap_or(env_settings.clone());

    let provider_kind = stored
        .provider_kind
        .clone()
        .or_else(|| env_settings.provider_kind.clone())
        .unwrap_or_else(|| infer_provider_kind(&stored.base_url, &stored.default_model));
    let base_url = normalize_adapter_base_url(&stored.base_url);
    let default_model = stored.default_model.clone();
    let device_id = stored
        .device_id
        .clone()
        .or_else(|| env_settings.device_id.clone());
    let environment_id = stored
        .environment_id
        .clone()
        .or_else(|| env_settings.environment_id.clone());
    let locality_kind = stored
        .locality_kind
        .or_else(|| env_settings.locality_kind)
        .unwrap_or_else(|| infer_provider_locality_kind(
            &base_url,
            device_id.as_deref(),
            environment_id.as_deref(),
        ));
    let batch_policy = stored
        .batch_policy
        .clone()
        .or_else(|| env_settings.batch_policy.clone());
    let discovery_source = stored
        .discovery_source
        .clone()
        .or_else(|| env_settings.discovery_source.clone())
        .or_else(|| Some("registry".to_string()));
    let runtime_context = build_runtime_context(
        &stored.provider_id,
        &provider_kind,
        &base_url,
        &default_model,
        device_id.clone(),
        environment_id.clone(),
        Some(locality_kind),
        discovery_source.clone(),
        stored
            .updated_at
            .clone()
            .unwrap_or_else(|| now_iso()),
    );

    let credential = loaded.as_ref().and_then(|state| {
        stored.credential_binding_id.as_ref().and_then(|binding_id| {
            state
                .credentials
                .iter()
                .find(|item| item.credential_binding_id == *binding_id)
                .cloned()
        })
    });

    ResolvedProviderRuntimeState {
        provider_id: stored.provider_id,
        name: stored.name,
        provider_kind,
        enabled: stored.enabled,
        base_url,
        default_model,
        adapter_set_ref: stored.adapter_set_ref,
        credential_binding_id: stored.credential_binding_id,
        has_credential: credential
            .as_ref()
            .map(|item| !item.api_key.trim().is_empty())
            .unwrap_or_else(|| !adapter_api_key_default().trim().is_empty()),
        credential_source: credential
            .and_then(|item| item.credential_source)
            .or_else(|| Some("environment".to_string())),
        runtime_context,
        batch_policy,
    }
}

pub fn provider_runtime_settings_from_resolved_state(
    resolved: &ResolvedProviderRuntimeState,
) -> ProviderRuntimeSettings {
    ProviderRuntimeSettings {
        provider_id: resolved.provider_id.clone(),
        name: resolved.name.clone(),
        provider_type: default_provider_type(),
        provider_kind: Some(resolved.provider_kind.clone()),
        enabled: resolved.enabled,
        base_url: resolved.base_url.clone(),
        default_model: resolved.default_model.clone(),
        adapter_set_ref: resolved.adapter_set_ref.clone(),
        credential_binding_id: resolved.credential_binding_id.clone(),
        provider_family_id: Some(resolved.runtime_context.family_id.clone()),
        profile_id: resolved.runtime_context.profile_id.clone(),
        instance_id: Some(resolved.runtime_context.instance_id.clone()),
        device_id: resolved.runtime_context.device_id.clone(),
        environment_id: resolved.runtime_context.environment_id.clone(),
        locality_kind: Some(resolved.runtime_context.locality_kind),
        discovery_source: resolved.runtime_context.discovery_source.clone(),
        batch_policy: resolved.batch_policy.clone(),
        updated_at: Some(resolved.runtime_context.last_seen_at.clone()),
        supported_models: vec![resolved.default_model.clone()]
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .collect(),
        metadata: std::collections::BTreeMap::new(),
    }
}

pub fn adapter_api_key_from_env() -> Option<String> {
    let value = adapter_api_key_default();
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub fn effective_provider_credential_source(
    binding: Option<&ProviderCredentialBindingRecord>,
) -> Option<String> {
    binding
        .and_then(|item| item.credential_source.clone())
        .or_else(|| Some("environment".to_string()))
}

pub fn llm_adapter_config_from_env() -> LlmAdapterConfig {
    let resolved = resolve_provider_runtime_state();
    let state = load_provider_registry_state().ok();
    let api_key = state
        .as_ref()
        .and_then(|loaded| {
            resolved.credential_binding_id.as_ref().and_then(|binding_id| {
                loaded
                    .credentials
                    .iter()
                    .find(|item| item.credential_binding_id == *binding_id)
                    .map(|item| item.api_key.clone())
            })
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(adapter_api_key_default);
    let timeout_secs = parse_u64_env("CORTEX_LLM_REQUEST_TIMEOUT_SECS", 90);
    let max_tool_steps = parse_usize_env("CORTEX_AGENT_MAX_TOOL_STEPS", 8);

    LlmAdapterConfig {
        enabled: resolved.enabled,
        base_url: resolved.base_url,
        api_key,
        request_timeout: Duration::from_secs(timeout_secs),
        fail_mode: LlmAdapterFailMode::from_env(),
        max_tool_steps,
        default_model: resolved.default_model,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_cleared_provider_env<F: FnOnce()>(f: F) {
        let keys = [
            "CORTEX_PROVIDER_FAMILY_ID",
            "CORTEX_PROVIDER_PROFILE_ID",
            "CORTEX_PROVIDER_INSTANCE_ID",
            "CORTEX_PROVIDER_DEVICE_ID",
            "CORTEX_PROVIDER_ENVIRONMENT_ID",
            "CORTEX_PROVIDER_LOCALITY_KIND",
            "CORTEX_PROVIDER_DISCOVERY_SOURCE",
        ];
        let saved = keys
            .iter()
            .map(|key| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for key in &keys {
            std::env::remove_var(key);
        }

        f();

        for (key, value) in saved {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn resolve_runtime_state_defaults_to_environment_contract() {
        with_cleared_provider_env(|| {
            let state = resolve_provider_runtime_state();
            assert_eq!(state.provider_id, "llm_adapter");
            assert!(!state.base_url.trim().is_empty());
            assert!(!state.default_model.trim().is_empty());
            assert!(!state.runtime_context.family_id.trim().is_empty());
            assert!(!state.runtime_context.instance_id.trim().is_empty());
        });
    }

    #[test]
    fn provider_registry_state_round_trips_runtime_settings_and_credentials() {
        let binding = ProviderCredentialBinding {
            credential_binding_id: "cred-openai".to_string(),
            provider_id: "llm_adapter".to_string(),
            label: Some("Primary".to_string()),
            credential_source: Some("manual".to_string()),
            api_key: "sk-test".to_string(),
            created_at: Some("2026-03-22T00:00:00Z".to_string()),
            updated_at: Some("2026-03-22T00:00:00Z".to_string()),
            metadata: std::collections::BTreeMap::new(),
        };
        let settings = ProviderRuntimeSettings {
            provider_id: "llm_adapter".to_string(),
            name: "Primary LLM Adapter".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("OpenAI".to_string()),
            enabled: true,
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-5.4".to_string(),
            adapter_set_ref: Some("adapter.primary".to_string()),
            credential_binding_id: Some("cred-openai".to_string()),
            provider_family_id: Some("openai".to_string()),
            profile_id: Some("gpt-5.4".to_string()),
            instance_id: Some("llm_adapter__api.openai.com".to_string()),
            device_id: Some("macbook-pro".to_string()),
            environment_id: Some("local".to_string()),
            locality_kind: Some(ProviderLocalityKind::Local),
            discovery_source: Some("registry".to_string()),
            batch_policy: Some(ProviderBatchPolicy {
                provider_family_id: "openai".to_string(),
                provider_profile_id: Some("gpt-5.4".to_string()),
                cadence_kind: ProviderBatchCadenceKind::Scoped,
                scope_kind: ProviderBatchScopeKind::ProviderFamily,
                flush_policy: ProviderBatchFlushPolicy::Manual,
                ordering_key: Some("request_id".to_string()),
                dedupe_key: Some("request_hash".to_string()),
                batch_window: Some(ProviderBatchWindow {
                    interval_seconds: Some(60),
                    max_items: Some(50),
                    max_age_seconds: Some(300),
                    timezone: Some("UTC".to_string()),
                }),
            }),
            updated_at: Some("2026-03-22T00:00:00Z".to_string()),
            supported_models: vec!["gpt-5.4".to_string(), "gpt-4.1".to_string()],
            metadata: std::collections::BTreeMap::new(),
        };
        let state = ProviderRegistryState {
            providers: vec![settings],
            credentials: vec![binding],
        };

        let encoded = serde_json::to_string(&state).expect("encode");
        let decoded: ProviderRegistryState = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded.providers.len(), 1);
        assert_eq!(decoded.credentials.len(), 1);
        assert_eq!(
            decoded.providers[0].credential_binding_id.as_deref(),
            Some("cred-openai")
        );
        assert!(decoded.providers[0].batch_policy.is_some());
        assert_eq!(
            decoded.credentials[0].credential_source.as_deref(),
            Some("manual")
        );
    }

    #[test]
    fn doubleword_provider_defaults_to_batch_policy() {
        with_cleared_provider_env(|| {
            std::env::set_var("CORTEX_LLM_ADAPTER_URL", "https://api.doubleword.ai/v1/batch");
            std::env::set_var("NOSTRA_AGENT_MODEL", "doubleword-batch-8k");

            let settings = build_provider_runtime_settings_from_env();
            assert_eq!(settings.provider_kind.as_deref(), Some("DoubleWord"));
            let policy = settings.batch_policy.expect("doubleword policy");
            assert_eq!(policy.provider_family_id, "doubleword");
            assert_eq!(policy.cadence_kind, ProviderBatchCadenceKind::Interval);
            assert_eq!(policy.scope_kind, ProviderBatchScopeKind::RequestGroup);
            assert_eq!(policy.flush_policy, ProviderBatchFlushPolicy::OnInterval);
            assert_eq!(policy.ordering_key.as_deref(), Some("request_group_id"));
            assert_eq!(policy.dedupe_key.as_deref(), Some("request_hash"));
            assert_eq!(
                policy.batch_window.as_ref().and_then(|window| window.interval_seconds),
                Some(60)
            );
        });
    }

    #[test]
    fn provider_locality_inference_prefers_explicit_tunnel_identity_over_loopback() {
        with_cleared_provider_env(|| {
            let locality = infer_provider_locality_kind(
                "http://127.0.0.1:11434",
                Some("macbook-pro"),
                Some("vps-mirror"),
            );

            assert_eq!(locality, ProviderLocalityKind::Tunneled);
        });
    }

    #[test]
    fn provider_locality_parser_treats_sovereign_as_local_for_compatibility() {
        assert_eq!(
            parse_provider_locality_kind("Sovereign"),
            Some(ProviderLocalityKind::Local)
        );
    }

    #[test]
    fn provider_runtime_context_defaults_to_local_loopback_metadata() {
        with_cleared_provider_env(|| {
            let context = build_runtime_context(
                "llm_adapter",
                "OpenRouter",
                "http://127.0.0.1:11434",
                "gpt-5.4",
                None,
                None,
                None,
                None,
                "2026-03-22T00:00:00Z".to_string(),
            );

            assert_eq!(context.family_id, "openrouter");
            assert_eq!(context.profile_id.as_deref(), Some("gpt-5.4"));
            assert_eq!(context.instance_id, "llm_adapter__http_127.0.0.1_11434");
            assert_eq!(context.locality_kind, ProviderLocalityKind::Local);
            assert_eq!(context.last_seen_at, "2026-03-22T00:00:00Z");
        });
    }
}
