use crate::services::cortex_ux::now_iso;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_LLM_BINDING_ID: &str = "llm.default";
pub const DEFAULT_PROVIDER_RUNTIME_HOST_ENV: &str = "CORTEX_PROVIDER_RUNTIME_HOST";
pub const DEFAULT_PROVIDER_RUNTIME_PORT_ENV: &str = "CORTEX_PROVIDER_RUNTIME_PORT";
pub const DEFAULT_PROVIDER_RUNTIME_URL_ENV: &str = "CORTEX_PROVIDER_RUNTIME_URL";
pub const DEFAULT_PROVIDER_RUNTIME_API_KEY_ENV: &str = "CORTEX_PROVIDER_RUNTIME_API_KEY";
pub const DEFAULT_PROVIDER_RUNTIME_ENABLED_ENV: &str = "CORTEX_PROVIDER_RUNTIME_ENABLED";
pub const DEFAULT_PROVIDER_RUNTIME_FAIL_MODE_ENV: &str = "CORTEX_PROVIDER_RUNTIME_FAIL_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRuntimeFailMode {
    Fallback,
    FailClosed,
}

impl ProviderRuntimeFailMode {
    pub fn from_env() -> Self {
        match std::env::var(DEFAULT_PROVIDER_RUNTIME_FAIL_MODE_ENV)
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
pub struct ProviderRuntimeConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub request_timeout: Duration,
    pub fail_mode: ProviderRuntimeFailMode,
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
pub enum AuthBindingTargetKind {
    Provider,
    Host,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthBindingType {
    None,
    ApiKey,
    BearerToken,
    Pat,
    SshKey,
    SshPassword,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthBindingRecord {
    pub auth_binding_id: String,
    #[serde(default = "default_auth_binding_target_kind")]
    pub target_kind: AuthBindingTargetKind,
    pub target_id: String,
    #[serde(default = "default_auth_binding_type")]
    pub auth_type: AuthBindingType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default)]
    pub secret: String,
    #[serde(default)]
    pub has_secret: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionBindingRecord {
    pub binding_id: String,
    pub provider_type: String,
    pub bound_provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeHostKind {
    Local,
    Vps,
    Tunnel,
    Managed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHostRecord {
    pub host_id: String,
    pub name: String,
    pub host_kind: RuntimeHostKind,
    pub endpoint: String,
    pub locality_kind: ProviderLocalityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health: Option<Value>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDiscoveryRecord {
    pub provider_id: String,
    pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,
    pub endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_health: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_health_error: Option<String>,
    #[serde(default)]
    pub openapi_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_models_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topology: Option<ProviderRuntimeContext>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    pub enabled: bool,
    pub base_url: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_binding_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRegistryState {
    #[serde(default)]
    pub providers: Vec<ProviderRuntimeSettings>,
    #[serde(default)]
    pub runtime_hosts: Vec<RuntimeHostRecord>,
    pub auth_bindings: Vec<AuthBindingRecord>,
    #[serde(default)]
    pub execution_bindings: Vec<ExecutionBindingRecord>,
    #[serde(default)]
    pub discovery: Vec<ProviderDiscoveryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedProviderRuntimeState {
    pub provider_id: String,
    pub name: String,
    pub provider_kind: String,
    pub host_id: String,
    pub enabled: bool,
    pub base_url: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_binding_id: Option<String>,
    pub auth_type: AuthBindingType,
    pub has_auth_secret: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_source: Option<String>,
    pub runtime_context: ProviderRuntimeContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_policy: Option<ProviderBatchPolicy>,
}

fn default_auth_binding_target_kind() -> AuthBindingTargetKind {
    AuthBindingTargetKind::Provider
}

fn default_auth_binding_type() -> AuthBindingType {
    AuthBindingType::ApiKey
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

fn normalize_provider_type(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(default_provider_type)
}

fn normalized_provider_type_slug(value: &str) -> String {
    normalize_provider_type(Some(value)).to_ascii_lowercase()
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
    let host = std::env::var(DEFAULT_PROVIDER_RUNTIME_HOST_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = parse_u64_env(DEFAULT_PROVIDER_RUNTIME_PORT_ENV, 8080);
    format!("http://{}:{}", host, port)
}

fn adapter_api_key_default() -> String {
    if let Ok(value) = std::env::var(DEFAULT_PROVIDER_RUNTIME_API_KEY_ENV) {
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
    String::new()
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

fn provider_locality_suffix(locality_kind: ProviderLocalityKind) -> &'static str {
    match locality_kind {
        ProviderLocalityKind::Local => "local",
        ProviderLocalityKind::Tunneled => "tunneled",
        ProviderLocalityKind::Cloud => "primary",
    }
}

fn provider_locality_display(locality_kind: ProviderLocalityKind) -> &'static str {
    match locality_kind {
        ProviderLocalityKind::Local => "Local",
        ProviderLocalityKind::Tunneled => "Tunneled",
        ProviderLocalityKind::Cloud => "Primary",
    }
}

fn derive_real_provider_id(
    provider_kind: &str,
    provider_type: &str,
    locality_kind: ProviderLocalityKind,
) -> String {
    let prefix = if provider_kind.trim().is_empty() {
        normalized_provider_type_slug(provider_type)
    } else {
        sanitize_identifier_component(provider_kind)
    };
    format!("{prefix}_{}", provider_locality_suffix(locality_kind))
}

fn default_provider_name(
    provider_kind: &str,
    provider_type: &str,
    locality_kind: ProviderLocalityKind,
) -> String {
    let prefix = if provider_kind.trim().is_empty() {
        normalize_provider_type(Some(provider_type))
    } else {
        provider_kind.trim().to_string()
    };
    format!("{prefix} {}", provider_locality_display(locality_kind))
}

fn infer_runtime_host_kind(
    endpoint: &str,
    locality_kind: ProviderLocalityKind,
    environment_id: Option<&str>,
) -> RuntimeHostKind {
    let environment = environment_id.unwrap_or_default().trim().to_ascii_lowercase();
    if environment.contains("vps") || environment.contains("hetzner") {
        return RuntimeHostKind::Vps;
    }

    match locality_kind {
        ProviderLocalityKind::Local => RuntimeHostKind::Local,
        ProviderLocalityKind::Tunneled => RuntimeHostKind::Tunnel,
        ProviderLocalityKind::Cloud => {
            let normalized = endpoint.trim().to_ascii_lowercase();
            if normalized.contains("204.168.175.150") {
                RuntimeHostKind::Vps
            } else {
                RuntimeHostKind::Managed
            }
        }
    }
}

pub fn default_runtime_host_id(
    provider_id: &str,
    endpoint: &str,
    locality_kind: ProviderLocalityKind,
    environment_id: Option<&str>,
) -> String {
    match infer_runtime_host_kind(endpoint, locality_kind, environment_id) {
        RuntimeHostKind::Local => "host.local.primary".to_string(),
        RuntimeHostKind::Vps => "host.vps.primary".to_string(),
        RuntimeHostKind::Tunnel => "host.tunnel.primary".to_string(),
        RuntimeHostKind::Managed => format!("host.managed.{provider_id}"),
    }
}

fn default_none_auth_binding_id(provider_id: &str) -> String {
    format!("auth.none.{provider_id}")
}

fn provider_requires_key(provider_kind: &str, locality_kind: ProviderLocalityKind) -> bool {
    if locality_kind == ProviderLocalityKind::Local && provider_kind.eq_ignore_ascii_case("ollama") {
        return false;
    }

    !provider_kind.eq_ignore_ascii_case("ollama")
}

fn infer_auth_binding_type(provider_kind: &str) -> AuthBindingType {
    if provider_kind.eq_ignore_ascii_case("ollama") {
        AuthBindingType::None
    } else {
        AuthBindingType::ApiKey
    }
}

pub fn binding_id_for_provider_type(provider_type: &str) -> String {
    match normalize_provider_type(Some(provider_type)).as_str() {
        "Llm" => DEFAULT_LLM_BINDING_ID.to_string(),
        other => format!("{}.default", other.to_ascii_lowercase()),
    }
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
    let enabled = bool_env(DEFAULT_PROVIDER_RUNTIME_ENABLED_ENV, true);
    let base_url = std::env::var(DEFAULT_PROVIDER_RUNTIME_URL_ENV)
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
    let resolved_locality_kind = locality_kind.unwrap_or(ProviderLocalityKind::Cloud);
    let discovery_source = discovery_source_from_env();
    let provider_id = derive_real_provider_id(&provider_kind, "Llm", resolved_locality_kind);
    let host_id = default_runtime_host_id(
        &provider_id,
        &base_url,
        resolved_locality_kind,
        environment_id.as_deref(),
    );
    let batch_policy = build_provider_batch_policy(&provider_kind, &provider_id, &default_model, &base_url);

    ProviderRuntimeSettings {
        provider_id: provider_id.clone(),
        name: default_provider_name(&provider_kind, "Llm", resolved_locality_kind),
        provider_type: default_provider_type(),
        provider_kind: Some(provider_kind.clone()),
        host_id: Some(host_id),
        enabled,
        base_url,
        default_model: default_model.clone(),
        adapter_set_ref: None,
        auth_binding_id: (!provider_requires_key(&provider_kind, resolved_locality_kind))
            .then(|| default_none_auth_binding_id(&provider_id)),
        provider_family_id: trimmed_env("CORTEX_PROVIDER_FAMILY_ID"),
        profile_id: trimmed_env("CORTEX_PROVIDER_PROFILE_ID"),
        instance_id: trimmed_env("CORTEX_PROVIDER_INSTANCE_ID"),
        device_id,
        environment_id,
        locality_kind,
        discovery_source,
        batch_policy,
        updated_at: None,
        supported_models: Vec::new(),
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
        return Ok(migrate_provider_registry_state(ProviderRegistryState::default()));
    }
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content)
        .map(migrate_provider_registry_state)
        .map_err(|err| err.to_string())
}

pub fn save_provider_registry_state(state: &ProviderRegistryState) -> Result<(), String> {
    let migrated = migrate_provider_registry_state(state.clone());
    save_json(&provider_registry_state_path(), &migrated)
}

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

pub fn upsert_auth_binding(
    binding: AuthBindingRecord,
) -> Result<(), String> {
    let mut state = load_provider_registry_state()?;
    if let Some(existing) = state
        .auth_bindings
        .iter_mut()
        .find(|item| item.auth_binding_id == binding.auth_binding_id)
    {
        *existing = binding;
    } else {
        state.auth_bindings.push(binding);
    }
    save_provider_registry_state(&state)
}

fn provider_runtime_settings_from_env() -> ProviderRuntimeSettings {
    build_provider_runtime_settings_from_env()
}

fn provider_discovery_record_from_settings(
    settings: &ProviderRuntimeSettings,
    supported_models: Vec<String>,
) -> ProviderDiscoveryRecord {
    let provider_type = normalize_provider_type(Some(&settings.provider_type));
    let provider_kind = settings
        .provider_kind
        .clone()
        .or_else(|| Some(infer_provider_kind(&settings.base_url, &settings.default_model)));
    ProviderDiscoveryRecord {
        provider_id: settings.provider_id.clone(),
        provider_type,
        provider_kind,
        endpoint: settings.base_url.clone(),
        default_model: Some(settings.default_model.clone()).filter(|value| !value.trim().is_empty()),
        supported_models,
        adapter_health: None,
        adapter_health_error: None,
        openapi_paths: Vec::new(),
        upstream_models_error: None,
        fail_mode: None,
        topology: Some(provider_runtime_context_from_resolved_state(
            &ResolvedProviderRuntimeState {
                provider_id: settings.provider_id.clone(),
                name: settings.name.clone(),
                provider_kind: settings
                    .provider_kind
                    .clone()
                    .unwrap_or_else(|| infer_provider_kind(&settings.base_url, &settings.default_model)),
                enabled: settings.enabled,
                base_url: settings.base_url.clone(),
                default_model: settings.default_model.clone(),
                adapter_set_ref: settings.adapter_set_ref.clone(),
                host_id: settings.host_id.clone().unwrap_or_else(|| {
                    default_runtime_host_id(
                        &settings.provider_id,
                        &settings.base_url,
                        settings.locality_kind.unwrap_or(ProviderLocalityKind::Cloud),
                        settings.environment_id.as_deref(),
                    )
                }),
                auth_binding_id: settings.auth_binding_id.clone(),
                auth_type: infer_auth_binding_type(
                    settings.provider_kind.as_deref().unwrap_or(""),
                ),
                has_auth_secret: false,
                auth_source: None,
                runtime_context: build_runtime_context(
                    &settings.provider_id,
                    settings
                        .provider_kind
                        .as_deref()
                        .unwrap_or(""),
                    &settings.base_url,
                    &settings.default_model,
                    settings.device_id.clone(),
                    settings.environment_id.clone(),
                    settings.locality_kind,
                    settings.discovery_source.clone(),
                    settings.updated_at.clone().unwrap_or_else(now_iso),
                ),
                batch_policy: settings.batch_policy.clone(),
            },
        )),
        updated_at: settings.updated_at.clone(),
        metadata: BTreeMap::new(),
    }
}

fn provider_runtime_context_from_resolved_state(
    resolved: &ResolvedProviderRuntimeState,
) -> ProviderRuntimeContext {
    resolved.runtime_context.clone()
}

fn provider_runtime_context_from_settings(
    settings: &ProviderRuntimeSettings,
) -> ProviderRuntimeContext {
    let provider_kind = settings
        .provider_kind
        .clone()
        .unwrap_or_else(|| infer_provider_kind(&settings.base_url, &settings.default_model));

    build_runtime_context(
        &settings.provider_id,
        &provider_kind,
        &settings.base_url,
        &settings.default_model,
        settings.device_id.clone(),
        settings.environment_id.clone(),
        settings.locality_kind,
        settings.discovery_source.clone(),
        settings.updated_at.clone().unwrap_or_else(now_iso),
    )
}

fn upsert_discovery_record(
    discovery: &mut Vec<ProviderDiscoveryRecord>,
    record: ProviderDiscoveryRecord,
) {
    if let Some(existing) = discovery
        .iter_mut()
        .find(|item| item.provider_id == record.provider_id)
    {
        *existing = record;
    } else {
        discovery.push(record);
    }
}

fn preferred_binding_provider_id(
    state: &ProviderRegistryState,
    provider_type: &str,
) -> Option<String> {
    let binding_id = binding_id_for_provider_type(provider_type);
    state
        .execution_bindings
        .iter()
        .find(|binding| binding.binding_id == binding_id)
        .map(|binding| binding.bound_provider_id.clone())
}

fn upsert_runtime_host(
    hosts: &mut Vec<RuntimeHostRecord>,
    host: RuntimeHostRecord,
) {
    if let Some(existing) = hosts.iter_mut().find(|item| item.host_id == host.host_id) {
        *existing = host;
    } else {
        hosts.push(host);
    }
}

fn runtime_host_from_provider(provider: &ProviderRuntimeSettings) -> RuntimeHostRecord {
    let locality_kind = provider.locality_kind.unwrap_or_else(|| {
        infer_provider_locality_kind(
            &provider.base_url,
            provider.device_id.as_deref(),
            provider.environment_id.as_deref(),
        )
    });
    let host_id = provider
        .host_id
        .clone()
        .unwrap_or_else(|| {
            default_runtime_host_id(
                &provider.provider_id,
                &provider.base_url,
                locality_kind,
                provider.environment_id.as_deref(),
            )
        });
    let host_kind = infer_runtime_host_kind(&provider.base_url, locality_kind, provider.environment_id.as_deref());

    RuntimeHostRecord {
        host_id,
        name: match host_kind {
            RuntimeHostKind::Local => "Local machine".to_string(),
            RuntimeHostKind::Vps => "Primary VPS".to_string(),
            RuntimeHostKind::Tunnel => "Tunneled runtime".to_string(),
            RuntimeHostKind::Managed => provider
                .provider_kind
                .clone()
                .unwrap_or_else(|| "Managed runtime".to_string()),
        },
        host_kind,
        endpoint: provider.base_url.clone(),
        locality_kind,
        device_id: provider.device_id.clone(),
        environment_id: provider.environment_id.clone(),
        health: None,
        capabilities: vec![normalize_provider_type(Some(&provider.provider_type))],
        updated_at: provider.updated_at.clone(),
        metadata: BTreeMap::new(),
    }
}

fn default_auth_binding_for_provider(provider: &ProviderRuntimeSettings) -> Option<AuthBindingRecord> {
    let provider_kind = provider
        .provider_kind
        .clone()
        .unwrap_or_else(|| infer_provider_kind(&provider.base_url, &provider.default_model));
    let locality_kind = provider.locality_kind.unwrap_or_else(|| {
        infer_provider_locality_kind(
            &provider.base_url,
            provider.device_id.as_deref(),
            provider.environment_id.as_deref(),
        )
    });

    if provider_requires_key(&provider_kind, locality_kind) {
        return None;
    }

    Some(AuthBindingRecord {
        auth_binding_id: default_none_auth_binding_id(&provider.provider_id),
        target_kind: AuthBindingTargetKind::Provider,
        target_id: provider.provider_id.clone(),
        auth_type: AuthBindingType::None,
        label: Some(format!("{} access", provider.name)),
        source: Some("provider".to_string()),
        secret: String::new(),
        has_secret: false,
        created_at: provider.updated_at.clone(),
        updated_at: provider.updated_at.clone().or_else(|| Some(now_iso())),
        metadata: BTreeMap::new(),
    })
}

pub fn migrate_provider_registry_state(mut state: ProviderRegistryState) -> ProviderRegistryState {
    let mut migrated_providers = Vec::with_capacity(state.providers.len());
    let mut discovery = state.discovery.clone();
    let mut runtime_hosts = state.runtime_hosts.clone();

    for mut provider in state.providers.drain(..) {
        let provider_type = normalize_provider_type(Some(&provider.provider_type));
        let provider_kind = provider
            .provider_kind
            .clone()
            .unwrap_or_else(|| infer_provider_kind(&provider.base_url, &provider.default_model));
        let locality_kind = provider.locality_kind.unwrap_or_else(|| {
            infer_provider_locality_kind(
                &provider.base_url,
                provider.device_id.as_deref(),
                provider.environment_id.as_deref(),
            )
        });
        let supported_models = provider.supported_models.clone();

        if provider.name.trim().is_empty() {
            provider.name = default_provider_name(&provider_kind, &provider_type, locality_kind);
        }
        provider.host_id = provider.host_id.clone().or_else(|| {
            Some(default_runtime_host_id(
                &provider.provider_id,
                &provider.base_url,
                locality_kind,
                provider.environment_id.as_deref(),
            ))
        });
        provider.adapter_set_ref = provider
            .adapter_set_ref
            .clone()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        if !supported_models.is_empty() {
            upsert_discovery_record(
                &mut discovery,
                provider_discovery_record_from_settings(&provider, supported_models),
            );
        }
        provider.supported_models = Vec::new();
        provider.provider_type = provider_type;
        provider.provider_kind = Some(provider_kind);
        provider.locality_kind = Some(locality_kind);
        if provider.auth_binding_id.as_deref().map(str::trim).unwrap_or("").is_empty() {
            provider.auth_binding_id = default_auth_binding_for_provider(&provider)
                .map(|binding| binding.auth_binding_id);
        }
        upsert_runtime_host(&mut runtime_hosts, runtime_host_from_provider(&provider));
        migrated_providers.push(provider);
    }

    let provider_lookup = migrated_providers
        .iter()
        .map(|provider| (provider.provider_id.clone(), provider.clone()))
        .collect::<BTreeMap<_, _>>();

    for binding in &mut state.auth_bindings {
        binding.auth_binding_id = binding.auth_binding_id.trim().to_string();
        binding.target_id = binding.target_id.trim().to_string();
        if let Some(provider) = provider_lookup.get(&binding.target_id) {
            let context = provider_runtime_context_from_settings(provider);
            binding
                .metadata
                .insert("providerId".to_string(), provider.provider_id.clone());
            binding.metadata.insert(
                "providerType".to_string(),
                normalize_provider_type(Some(&provider.provider_type)),
            );
            if let Some(provider_kind) = provider.provider_kind.as_ref() {
                binding
                    .metadata
                    .insert("providerKind".to_string(), provider_kind.clone());
            }
            binding
                .metadata
                .insert("familyId".to_string(), context.family_id.clone());
            if let Some(profile_id) = context.profile_id.as_ref() {
                binding
                    .metadata
                    .insert("profileId".to_string(), profile_id.to_string());
            }
            binding
                .metadata
                .insert("instanceId".to_string(), context.instance_id.clone());
            binding.metadata.insert(
                "localityKind".to_string(),
                format!("{:?}", context.locality_kind),
            );
            binding
                .metadata
                .insert("lastSeenAt".to_string(), context.last_seen_at.clone());
            if let Some(discovery_source) = context.discovery_source.as_ref() {
                binding
                    .metadata
                    .insert("discoverySource".to_string(), discovery_source.to_string());
            }
            if binding.auth_type != AuthBindingType::None {
                binding.auth_type = infer_auth_binding_type(provider.provider_kind.as_deref().unwrap_or(""));
            }
            binding.has_secret = !binding.secret.trim().is_empty();
            binding.target_kind = AuthBindingTargetKind::Provider;
        }
    }

    for provider in provider_lookup.values() {
        if let Some(binding) = default_auth_binding_for_provider(provider) {
            if !state
                .auth_bindings
                .iter()
                .any(|existing| existing.auth_binding_id == binding.auth_binding_id)
            {
                state.auth_bindings.push(binding);
            }
        }
    }

    discovery.retain(|record| !record.provider_id.trim().is_empty());
    for record in &mut discovery {
        record.provider_id = record.provider_id.trim().to_string();
        if let Some(provider) = provider_lookup.get(&record.provider_id) {
            if record.provider_type.trim().is_empty() {
                record.provider_type = normalize_provider_type(Some(&provider.provider_type));
            }
            if record.provider_kind.as_deref().unwrap_or("").trim().is_empty() {
                record.provider_kind = provider.provider_kind.clone();
            }
            if record.endpoint.trim().is_empty() {
                record.endpoint = provider.base_url.clone();
            }
        }
    }

    if !state
        .execution_bindings
        .iter()
        .any(|binding| binding.binding_id == DEFAULT_LLM_BINDING_ID)
    {
        if let Some(bound_provider_id) = migrated_providers
            .iter()
            .find(|provider| normalize_provider_type(Some(&provider.provider_type)) == "Llm")
            .map(|provider| provider.provider_id.clone())
        {
            state.execution_bindings.push(ExecutionBindingRecord {
                binding_id: DEFAULT_LLM_BINDING_ID.to_string(),
                provider_type: "Llm".to_string(),
                bound_provider_id,
                updated_at: Some(now_iso()),
                metadata: BTreeMap::new(),
            });
        }
    }
    state.execution_bindings.retain(|binding| {
        !binding.binding_id.trim().is_empty() && provider_lookup.contains_key(&binding.bound_provider_id)
    });

    state.providers = migrated_providers;
    state.runtime_hosts = runtime_hosts;
    state.discovery = discovery;
    state
}

pub fn resolve_provider_runtime_state() -> ResolvedProviderRuntimeState {
    let env_settings = provider_runtime_settings_from_env();
    let loaded = load_provider_registry_state().ok();
    let stored = loaded
        .as_ref()
        .and_then(|state| {
            preferred_binding_provider_id(state, "Llm").and_then(|provider_id| {
                state
                    .providers
                    .iter()
                    .find(|item| item.provider_id == provider_id)
                    .cloned()
            })
        })
        .unwrap_or(env_settings.clone());
    let host_id = stored.host_id.clone().unwrap_or_else(|| {
        default_runtime_host_id(
            &stored.provider_id,
            &stored.base_url,
            stored.locality_kind.unwrap_or(ProviderLocalityKind::Cloud),
            stored.environment_id.as_deref(),
        )
    });
    let discovered = loaded.as_ref().and_then(|state| {
        state
            .discovery
            .iter()
            .find(|item| item.provider_id == stored.provider_id)
            .cloned()
    });

    let provider_kind = stored
        .provider_kind
        .clone()
        .or_else(|| discovered.as_ref().and_then(|item| item.provider_kind.clone()))
        .or_else(|| env_settings.provider_kind.clone())
        .unwrap_or_else(|| infer_provider_kind(&stored.base_url, &stored.default_model));
    let base_url = normalize_adapter_base_url(
        discovered
            .as_ref()
            .map(|item| item.endpoint.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&stored.base_url),
    );
    let default_model = Some(stored.default_model.clone())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| discovered.as_ref().and_then(|item| item.default_model.clone()))
        .or_else(|| {
            discovered.as_ref().and_then(|item| {
                item.supported_models
                    .iter()
                    .find(|value| !value.trim().is_empty())
                    .cloned()
            })
        })
        .unwrap_or_else(|| env_settings.default_model.clone());
    let device_id = stored
        .device_id
        .clone()
        .or_else(|| env_settings.device_id.clone());
    let environment_id = stored
        .environment_id
        .clone()
        .or_else(|| env_settings.environment_id.clone());
    let locality_kind = discovered
        .as_ref()
        .and_then(|item| item.topology.as_ref())
        .map(|topology| topology.locality_kind)
        .or_else(|| env_settings.locality_kind)
        .unwrap_or_else(|| {
            infer_provider_locality_kind(
                &base_url,
                device_id.as_deref(),
                environment_id.as_deref(),
            )
        });
    let batch_policy = stored
        .batch_policy
        .clone()
        .or_else(|| env_settings.batch_policy.clone());
    let discovery_source = stored
        .discovery_source
        .clone()
        .or_else(|| {
            discovered
                .as_ref()
                .and_then(|item| item.topology.as_ref())
                .and_then(|topology| topology.discovery_source.clone())
        })
        .or_else(|| env_settings.discovery_source.clone())
        .or_else(|| Some("registry".to_string()));
    let runtime_context = discovered
        .as_ref()
        .and_then(|item| item.topology.clone())
        .unwrap_or_else(|| {
            build_runtime_context(
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
                    .unwrap_or_else(now_iso),
            )
        });

    let auth_binding = loaded.as_ref().and_then(|state| {
        stored.auth_binding_id.as_ref().and_then(|binding_id| {
            state
                .auth_bindings
                .iter()
                .find(|item| item.auth_binding_id == *binding_id)
                .cloned()
        })
    });
    let provider_kind_for_auth = stored
        .provider_kind
        .as_deref()
        .unwrap_or(provider_kind.as_str());
    let resolved_auth_type = auth_binding
        .as_ref()
        .map(|item| item.auth_type.clone())
        .unwrap_or_else(|| {
            if provider_requires_key(provider_kind_for_auth, locality_kind) {
                AuthBindingType::ApiKey
            } else {
                AuthBindingType::None
            }
        });
    let has_auth_secret = auth_binding
        .as_ref()
        .map(|item| item.has_secret || !item.secret.trim().is_empty())
        .unwrap_or_else(|| {
            if resolved_auth_type == AuthBindingType::None {
                false
            } else {
                !adapter_api_key_default().trim().is_empty()
            }
        });

    ResolvedProviderRuntimeState {
        provider_id: stored.provider_id,
        name: stored.name,
        provider_kind,
        host_id,
        enabled: stored.enabled,
        base_url,
        default_model,
        adapter_set_ref: stored.adapter_set_ref,
        auth_binding_id: stored.auth_binding_id,
        auth_type: resolved_auth_type,
        has_auth_secret,
        auth_source: auth_binding
            .and_then(|item| item.source)
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
        host_id: Some(resolved.host_id.clone()),
        enabled: resolved.enabled,
        base_url: resolved.base_url.clone(),
        default_model: resolved.default_model.clone(),
        adapter_set_ref: None,
        auth_binding_id: resolved.auth_binding_id.clone(),
        provider_family_id: Some(resolved.runtime_context.family_id.clone()),
        profile_id: resolved.runtime_context.profile_id.clone(),
        instance_id: Some(resolved.runtime_context.instance_id.clone()),
        device_id: resolved.runtime_context.device_id.clone(),
        environment_id: resolved.runtime_context.environment_id.clone(),
        locality_kind: Some(resolved.runtime_context.locality_kind),
        discovery_source: resolved.runtime_context.discovery_source.clone(),
        batch_policy: resolved.batch_policy.clone(),
        updated_at: Some(resolved.runtime_context.last_seen_at.clone()),
        supported_models: Vec::new(),
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

pub fn effective_provider_auth_source(
    binding: Option<&AuthBindingRecord>,
) -> Option<String> {
    binding
        .and_then(|item| item.source.clone())
        .or_else(|| Some("environment".to_string()))
}

pub fn provider_runtime_config_from_env() -> ProviderRuntimeConfig {
    let resolved = resolve_provider_runtime_state();
    let state = load_provider_registry_state().ok();
    let api_key = state
        .as_ref()
        .and_then(|loaded| {
            resolved.auth_binding_id.as_ref().and_then(|binding_id| {
                loaded
                    .auth_bindings
                    .iter()
                    .find(|item| item.auth_binding_id == *binding_id)
                    .map(|item| item.secret.clone())
            })
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(adapter_api_key_default);
    let timeout_secs = parse_u64_env("CORTEX_LLM_REQUEST_TIMEOUT_SECS", 90);
    let max_tool_steps = parse_usize_env("CORTEX_AGENT_MAX_TOOL_STEPS", 8);

    ProviderRuntimeConfig {
        enabled: resolved.enabled,
        base_url: resolved.base_url,
        api_key,
        request_timeout: Duration::from_secs(timeout_secs),
        fail_mode: ProviderRuntimeFailMode::from_env(),
        max_tool_steps,
        default_model: resolved.default_model,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_cleared_provider_env<F: FnOnce()>(f: F) {
        let keys = [
            DEFAULT_PROVIDER_RUNTIME_HOST_ENV,
            DEFAULT_PROVIDER_RUNTIME_PORT_ENV,
            DEFAULT_PROVIDER_RUNTIME_URL_ENV,
            DEFAULT_PROVIDER_RUNTIME_API_KEY_ENV,
            DEFAULT_PROVIDER_RUNTIME_ENABLED_ENV,
            DEFAULT_PROVIDER_RUNTIME_FAIL_MODE_ENV,
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
            assert_eq!(state.provider_id, "ollama_local");
            assert!(!state.base_url.trim().is_empty());
            assert!(!state.default_model.trim().is_empty());
            assert!(!state.runtime_context.family_id.trim().is_empty());
            assert!(!state.runtime_context.instance_id.trim().is_empty());
        });
    }

    #[test]
    fn provider_registry_state_round_trips_runtime_settings_and_auth_bindings() {
        let binding = AuthBindingRecord {
            auth_binding_id: "auth-openai".to_string(),
            target_kind: AuthBindingTargetKind::Provider,
            target_id: "openai_primary".to_string(),
            auth_type: AuthBindingType::ApiKey,
            label: Some("Primary".to_string()),
            source: Some("manual".to_string()),
            secret: "sk-test".to_string(),
            has_secret: true,
            created_at: Some("2026-03-22T00:00:00Z".to_string()),
            updated_at: Some("2026-03-22T00:00:00Z".to_string()),
            metadata: std::collections::BTreeMap::new(),
        };
        let settings = ProviderRuntimeSettings {
            provider_id: "openai_primary".to_string(),
            name: "Primary OpenAI".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("OpenAI".to_string()),
            host_id: Some("host.managed.openai_primary".to_string()),
            enabled: true,
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-5.4".to_string(),
            adapter_set_ref: Some("adapter.primary".to_string()),
            auth_binding_id: Some("auth-openai".to_string()),
            provider_family_id: Some("openai".to_string()),
            profile_id: Some("gpt-5.4".to_string()),
            instance_id: Some("openai_primary__api.openai.com".to_string()),
            device_id: None,
            environment_id: None,
            locality_kind: Some(ProviderLocalityKind::Cloud),
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
            runtime_hosts: Vec::new(),
            auth_bindings: vec![binding],
            execution_bindings: Vec::new(),
            discovery: Vec::new(),
        };

        let encoded = serde_json::to_string(&state).expect("encode");
        let decoded: ProviderRegistryState = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded.providers.len(), 1);
        assert_eq!(decoded.auth_bindings.len(), 1);
        assert_eq!(
            decoded.providers[0].auth_binding_id.as_deref(),
            Some("auth-openai")
        );
        assert!(decoded.providers[0].batch_policy.is_some());
        assert_eq!(
            decoded.auth_bindings[0].source.as_deref(),
            Some("manual")
        );
    }

    #[test]
    fn migrate_provider_registry_state_normalizes_real_provider_records_and_promotes_catalogs_to_discovery() {
        let legacy_provider = ProviderRuntimeSettings {
            provider_id: "openrouter_primary".to_string(),
            name: "OpenRouter x".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("OpenRouter".to_string()),
            host_id: Some("host.managed.openrouter_primary".to_string()),
            enabled: true,
            base_url: "https://openrouter.ai/api".to_string(),
            default_model: "openai/gpt-5.4".to_string(),
            adapter_set_ref: Some("adapter.default".to_string()),
            auth_binding_id: Some("auth_openrouter_primary".to_string()),
            provider_family_id: Some("openrouter".to_string()),
            profile_id: Some("openai/gpt-5.4".to_string()),
            instance_id: Some("openrouter_primary__https_openrouter.ai_api".to_string()),
            device_id: None,
            environment_id: None,
            locality_kind: Some(ProviderLocalityKind::Cloud),
            discovery_source: Some("discovered".to_string()),
            batch_policy: None,
            updated_at: Some("2026-03-24T00:00:00Z".to_string()),
            supported_models: vec!["openai/gpt-5.4".to_string(), "openai/gpt-4.1".to_string()],
            metadata: BTreeMap::from([("customNote".to_string(), "keep me".to_string())]),
        };
        let legacy_auth_binding = AuthBindingRecord {
            auth_binding_id: "auth_openrouter_primary".to_string(),
            target_kind: AuthBindingTargetKind::Provider,
            target_id: "openrouter_primary".to_string(),
            auth_type: AuthBindingType::ApiKey,
            label: Some("OpenRouter x".to_string()),
            source: Some("manual".to_string()),
            secret: "sk-test".to_string(),
            has_secret: true,
            created_at: Some("2026-03-24T00:00:00Z".to_string()),
            updated_at: Some("2026-03-24T00:00:00Z".to_string()),
            metadata: BTreeMap::from([
                ("providerId".to_string(), "openrouter_primary".to_string()),
                ("providerKind".to_string(), "Ollama".to_string()),
            ]),
        };

        let migrated = migrate_provider_registry_state(ProviderRegistryState {
            providers: vec![legacy_provider],
            runtime_hosts: Vec::new(),
            auth_bindings: vec![legacy_auth_binding],
            execution_bindings: Vec::new(),
            discovery: Vec::new(),
        });

        assert_eq!(migrated.providers.len(), 1);
        assert_eq!(migrated.providers[0].provider_id, "openrouter_primary");
        assert!(migrated.providers[0].supported_models.is_empty());
        assert_eq!(migrated.providers[0].adapter_set_ref.as_deref(), Some("adapter.default"));
        assert_eq!(migrated.auth_bindings[0].target_id, "openrouter_primary");
        assert_eq!(
            migrated.auth_bindings[0].metadata.get("providerId"),
            Some(&"openrouter_primary".to_string())
        );
        assert_eq!(
            migrated.auth_bindings[0].metadata.get("providerKind"),
            Some(&"OpenRouter".to_string())
        );
        assert_eq!(migrated.execution_bindings.len(), 1);
        assert_eq!(migrated.execution_bindings[0].binding_id, "llm.default");
        assert_eq!(migrated.execution_bindings[0].bound_provider_id, "openrouter_primary");
        assert_eq!(migrated.discovery.len(), 1);
        assert_eq!(migrated.discovery[0].provider_id, "openrouter_primary");
        assert_eq!(
            migrated.discovery[0].supported_models,
            vec!["openai/gpt-5.4".to_string(), "openai/gpt-4.1".to_string()]
        );
    }

    #[test]
    fn migrate_provider_registry_state_creates_runtime_hosts_auth_bindings_and_default_execution_binding() {
        let openrouter_provider = ProviderRuntimeSettings {
            provider_id: "openrouter_primary".to_string(),
            name: "OpenRouter x".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("OpenRouter".to_string()),
            host_id: None,
            enabled: true,
            base_url: "https://openrouter.ai/api".to_string(),
            default_model: "openai/gpt-5.4".to_string(),
            adapter_set_ref: None,
            auth_binding_id: Some("auth_openrouter_primary".to_string()),
            provider_family_id: Some("openrouter".to_string()),
            profile_id: Some("openai/gpt-5.4".to_string()),
            instance_id: Some("openrouter_primary__https_openrouter.ai_api".to_string()),
            device_id: None,
            environment_id: None,
            locality_kind: Some(ProviderLocalityKind::Cloud),
            discovery_source: Some("registry".to_string()),
            batch_policy: None,
            updated_at: Some("2026-03-25T00:00:00Z".to_string()),
            supported_models: vec!["openai/gpt-5.4".to_string()],
            metadata: BTreeMap::new(),
        };
        let ollama_provider = ProviderRuntimeSettings {
            provider_id: "ollama_local".to_string(),
            name: "Ollama".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("Ollama".to_string()),
            host_id: None,
            enabled: true,
            base_url: "http://127.0.0.1:11434".to_string(),
            default_model: "llama3.1:8b".to_string(),
            adapter_set_ref: None,
            auth_binding_id: None,
            provider_family_id: Some("ollama".to_string()),
            profile_id: Some("llama3.1:8b".to_string()),
            instance_id: Some("ollama_local__http_127.0.0.1_11434".to_string()),
            device_id: Some("macbook-pro".to_string()),
            environment_id: Some("local-dev".to_string()),
            locality_kind: Some(ProviderLocalityKind::Local),
            discovery_source: Some("discovered".to_string()),
            batch_policy: None,
            updated_at: Some("2026-03-25T00:00:00Z".to_string()),
            supported_models: vec!["llama3.1:8b".to_string()],
            metadata: BTreeMap::new(),
        };
        let openrouter_auth = AuthBindingRecord {
            auth_binding_id: "auth_openrouter_primary".to_string(),
            target_kind: AuthBindingTargetKind::Provider,
            target_id: "openrouter_primary".to_string(),
            auth_type: AuthBindingType::ApiKey,
            label: Some("OpenRouter x".to_string()),
            source: Some("manual".to_string()),
            secret: "sk-test".to_string(),
            has_secret: true,
            created_at: Some("2026-03-25T00:00:00Z".to_string()),
            updated_at: Some("2026-03-25T00:00:00Z".to_string()),
            metadata: BTreeMap::new(),
        };

        let migrated = migrate_provider_registry_state(ProviderRegistryState {
            providers: vec![openrouter_provider, ollama_provider],
            runtime_hosts: Vec::new(),
            auth_bindings: vec![openrouter_auth],
            execution_bindings: Vec::new(),
            discovery: Vec::new(),
        });

        assert_eq!(migrated.runtime_hosts.len(), 2);
        assert!(migrated
            .runtime_hosts
            .iter()
            .any(|host| host.host_id == "host.local.primary" && host.host_kind == RuntimeHostKind::Local));
        assert!(migrated
            .runtime_hosts
            .iter()
            .any(|host| host.host_kind == RuntimeHostKind::Managed));
        assert_eq!(migrated.auth_bindings.len(), 2);
        assert!(migrated
            .auth_bindings
            .iter()
            .any(|binding| binding.target_id == "ollama_local" && binding.auth_type == AuthBindingType::None));
        assert_eq!(migrated.execution_bindings.len(), 1);
        assert_eq!(migrated.execution_bindings[0].binding_id, DEFAULT_LLM_BINDING_ID);
        assert_eq!(
            migrated.providers.iter().find(|provider| provider.provider_id == "openrouter_primary").and_then(|provider| provider.host_id.clone()).as_deref(),
            Some("host.managed.openrouter_primary"),
        );
        assert_eq!(
            migrated.providers.iter().find(|provider| provider.provider_id == "ollama_local").and_then(|provider| provider.auth_binding_id.clone()).as_deref(),
            Some("auth.none.ollama_local"),
        );
    }

    #[test]
    fn doubleword_provider_defaults_to_batch_policy() {
        with_cleared_provider_env(|| {
            std::env::set_var(DEFAULT_PROVIDER_RUNTIME_URL_ENV, "https://api.doubleword.ai/v1/batch");
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
                "ollama_local",
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
            assert_eq!(context.instance_id, "ollama_local__http_127.0.0.1_11434");
            assert_eq!(context.locality_kind, ProviderLocalityKind::Local);
            assert_eq!(context.last_seen_at, "2026-03-22T00:00:00Z");
        });
    }
}
