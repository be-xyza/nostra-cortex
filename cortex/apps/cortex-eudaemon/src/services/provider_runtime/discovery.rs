use crate::services::cortex_ux::now_iso;
use crate::services::provider_runtime::client::ProviderRuntimeClient;
use crate::services::provider_runtime::config::{
    AuthBindingRecord, AuthBindingTargetKind, AuthBindingType, ProviderDiscoveryRecord,
    ProviderRegistryState, ProviderRuntimeConfig, ProviderRuntimeContext, ProviderRuntimeFailMode,
    ProviderRuntimeSettings, RuntimeHostKind, RuntimeHostRecord, infer_provider_kind,
    infer_provider_locality_kind,
};
use crate::services::provider_runtime::policy::runtime_host_allows_remote_discovery;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub(crate) struct ProviderDiagnostics {
    pub(crate) default_model: String,
    pub(crate) supported_models: Vec<String>,
    pub(crate) adapter_health: Option<Value>,
    pub(crate) adapter_health_error: Option<String>,
    pub(crate) openapi_paths: Vec<String>,
    pub(crate) upstream_models_error: Option<String>,
    pub(crate) fail_mode: Option<String>,
}

fn normalize_provider_type(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Llm".to_string())
}

fn provider_runtime_context_from_settings(
    settings: &ProviderRuntimeSettings,
) -> ProviderRuntimeContext {
    let provider_type = normalize_provider_type(Some(&settings.provider_type));
    let provider_kind = settings
        .provider_kind
        .clone()
        .unwrap_or_else(|| infer_provider_kind(&settings.base_url, &settings.default_model));
    let family_id = settings
        .provider_family_id
        .clone()
        .unwrap_or_else(|| provider_type.to_ascii_lowercase());
    let profile_id = settings.profile_id.clone().or_else(|| {
        Some(settings.default_model.trim().to_string()).filter(|value| !value.is_empty())
    });
    let instance_id = settings.instance_id.clone().unwrap_or_else(|| {
        format!(
            "{}_{}",
            settings.provider_id,
            provider_type.to_ascii_lowercase()
        )
    });
    let locality_kind = infer_provider_locality_kind(
        &settings.base_url,
        settings.device_id.as_deref(),
        settings.environment_id.as_deref(),
    );

    ProviderRuntimeContext {
        family_id,
        profile_id: profile_id.or_else(|| Some(provider_kind)),
        instance_id,
        device_id: settings.device_id.clone(),
        environment_id: settings.environment_id.clone(),
        locality_kind,
        last_seen_at: settings.updated_at.clone().unwrap_or_else(now_iso),
        discovery_source: settings
            .discovery_source
            .clone()
            .or_else(|| Some("registry".to_string())),
    }
}

fn discovery_record_from_provider_settings(
    settings: &ProviderRuntimeSettings,
    diagnostics: Option<&ProviderDiagnostics>,
) -> ProviderDiscoveryRecord {
    let provider_type = normalize_provider_type(Some(&settings.provider_type));
    let topology = Some(provider_runtime_context_from_settings(settings));
    let default_model =
        Some(settings.default_model.clone()).filter(|value| !value.trim().is_empty());
    let mut supported_models = diagnostics
        .map(|item| item.supported_models.clone())
        .filter(|models| !models.is_empty())
        .unwrap_or_else(|| settings.supported_models.clone());
    if supported_models.is_empty() {
        if let Some(model) = default_model.clone() {
            supported_models.push(model);
        }
    }
    ProviderDiscoveryRecord {
        provider_id: settings.provider_id.clone(),
        provider_type,
        provider_kind: settings.provider_kind.clone().or_else(|| {
            Some(infer_provider_kind(
                &settings.base_url,
                &settings.default_model,
            ))
        }),
        endpoint: settings.base_url.clone(),
        default_model,
        supported_models,
        adapter_health: diagnostics.and_then(|item| item.adapter_health.clone()),
        adapter_health_error: diagnostics.and_then(|item| item.adapter_health_error.clone()),
        openapi_paths: diagnostics
            .map(|item| item.openapi_paths.clone())
            .unwrap_or_default(),
        upstream_models_error: diagnostics.and_then(|item| item.upstream_models_error.clone()),
        fail_mode: diagnostics.and_then(|item| item.fail_mode.clone()),
        topology,
        updated_at: settings.updated_at.clone().or_else(|| Some(now_iso())),
        metadata: BTreeMap::new(),
    }
}

fn extract_supported_model_name(entry: &Value) -> Option<String> {
    if let Some(model) = entry.as_str() {
        let trimmed = model.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    entry
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            entry
                .get("name")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn extract_supported_models(payload: &Value) -> Vec<String> {
    if let Some(rows) = payload.as_array() {
        return rows
            .iter()
            .filter_map(extract_supported_model_name)
            .collect();
    }

    payload
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(extract_supported_model_name)
        .collect()
}

pub(crate) async fn resolve_provider_supported_models(
    cfg: &ProviderRuntimeConfig,
) -> (
    Vec<String>,
    Option<Value>,
    Option<String>,
    Vec<String>,
    Option<String>,
) {
    let mut adapter_health = None;
    let mut adapter_health_error = None;
    let mut openapi_paths = Vec::new();
    let mut upstream_models_error = None;
    if !cfg.enabled {
        return (
            vec![cfg.default_model.clone()],
            None,
            None,
            Vec::new(),
            None,
        );
    }
    let Ok(client) = ProviderRuntimeClient::new(cfg.clone()) else {
        return (
            vec![cfg.default_model.clone()],
            None,
            Some("provider_runtime_client_init_failed".to_string()),
            Vec::new(),
            None,
        );
    };

    match client.health_adapter().await {
        Ok(value) => adapter_health = Some(value),
        Err(err) => adapter_health_error = Some(err),
    }
    match client.openapi_paths().await {
        Ok(value) => openapi_paths = value,
        Err(err) => {
            if adapter_health_error.is_none() {
                adapter_health_error = Some(err);
            }
        }
    }
    let supported_models = match client.health_upstream_models().await {
        Ok(value) => {
            let parsed = extract_supported_models(&value);
            if parsed.is_empty() {
                vec![cfg.default_model.clone()]
            } else {
                parsed
            }
        }
        Err(err) => {
            upstream_models_error = Some(err);
            vec![cfg.default_model.clone()]
        }
    };

    (
        supported_models,
        adapter_health,
        adapter_health_error,
        openapi_paths,
        upstream_models_error,
    )
}

fn provider_client_config(
    settings: &ProviderRuntimeSettings,
    auth_binding_lookup: &BTreeMap<String, AuthBindingRecord>,
) -> ProviderRuntimeConfig {
    let api_key = settings
        .auth_binding_id
        .as_ref()
        .and_then(|binding_id| auth_binding_lookup.get(binding_id))
        .map(|binding| binding.secret.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_default();
    ProviderRuntimeConfig {
        enabled: settings.enabled,
        base_url: settings.base_url.clone(),
        api_key,
        request_timeout: std::time::Duration::from_secs(15),
        fail_mode: ProviderRuntimeFailMode::Fallback,
        max_tool_steps: 1,
        default_model: settings.default_model.clone(),
    }
}

fn candidate_local_providers(state: &ProviderRegistryState) -> Vec<ProviderRuntimeSettings> {
    let mut providers = state
        .providers
        .iter()
        .filter(|provider| {
            let provider_kind = provider.provider_kind.as_deref().unwrap_or("");
            let endpoint = provider.base_url.to_ascii_lowercase();
            provider_kind.eq_ignore_ascii_case("ollama")
                || endpoint.contains("127.0.0.1:11434")
                || endpoint.contains("localhost:11434")
        })
        .cloned()
        .collect::<Vec<_>>();
    if !providers
        .iter()
        .any(|provider| provider.provider_id == "ollama_local")
    {
        providers.push(ProviderRuntimeSettings {
            provider_id: "ollama_local".to_string(),
            name: "Ollama Local".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("Ollama".to_string()),
            host_id: Some("host.local.primary".to_string()),
            enabled: true,
            base_url: "http://127.0.0.1:11434".to_string(),
            default_model: String::new(),
            adapter_set_ref: None,
            auth_binding_id: Some("auth.none.ollama_local".to_string()),
            provider_family_id: Some("ollama".to_string()),
            profile_id: None,
            instance_id: Some("ollama_local__http_127.0.0.1_11434".to_string()),
            device_id: None,
            environment_id: None,
            locality_kind: Some(
                crate::services::provider_runtime::config::ProviderLocalityKind::Local,
            ),
            discovery_source: Some("auto".to_string()),
            batch_policy: None,
            updated_at: Some(now_iso()),
            supported_models: Vec::new(),
            metadata: BTreeMap::new(),
        });
    }
    providers
}

pub(crate) fn upsert_discovery_record(
    records: &mut Vec<ProviderDiscoveryRecord>,
    record: ProviderDiscoveryRecord,
) {
    if let Some(existing) = records
        .iter_mut()
        .find(|item| item.provider_id == record.provider_id)
    {
        *existing = record;
    } else {
        records.push(record);
    }
}

pub(crate) async fn collect_live_discovery_records(
    state: &ProviderRegistryState,
    active_provider_id: &str,
    active_provider_settings: &ProviderRuntimeSettings,
    active_diagnostics: &ProviderDiagnostics,
    auth_binding_lookup: &BTreeMap<String, AuthBindingRecord>,
) -> Vec<ProviderDiscoveryRecord> {
    let mut records = state.discovery.clone();
    upsert_discovery_record(
        &mut records,
        discovery_record_from_provider_settings(active_provider_settings, Some(active_diagnostics)),
    );

    for provider in candidate_local_providers(state) {
        if provider.provider_id == active_provider_id {
            continue;
        }
        let cfg = provider_client_config(&provider, auth_binding_lookup);
        let (
            supported_models,
            adapter_health,
            adapter_health_error,
            openapi_paths,
            upstream_models_error,
        ) = resolve_provider_supported_models(&cfg).await;
        if supported_models.is_empty()
            && upstream_models_error.is_some()
            && adapter_health.is_none()
        {
            continue;
        }
        let diagnostics = ProviderDiagnostics {
            default_model: provider.default_model.clone(),
            supported_models,
            adapter_health,
            adapter_health_error,
            openapi_paths,
            upstream_models_error,
            fail_mode: Some("fallback".to_string()),
        };
        upsert_discovery_record(
            &mut records,
            discovery_record_from_provider_settings(&provider, Some(&diagnostics)),
        );
    }

    records
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteHostRuntimeProbe {
    status: String,
    detail: Option<String>,
    provider_kind: Option<String>,
    supported_models: Vec<String>,
    default_model: Option<String>,
}

fn sanitize_runtime_identifier(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut last_was_separator = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            output.push('_');
            last_was_separator = true;
        }
    }

    let collapsed = output.trim_matches('_').to_string();
    if collapsed.is_empty() {
        "runtime".to_string()
    } else {
        collapsed
    }
}

fn provider_id_for_runtime_host(provider_kind: &str, host: &RuntimeHostRecord) -> String {
    let prefix = sanitize_runtime_identifier(provider_kind);
    let host_suffix = match host.host_kind {
        RuntimeHostKind::Local => "local".to_string(),
        RuntimeHostKind::Vps => "vps_primary".to_string(),
        RuntimeHostKind::Tunnel => "tunnel_primary".to_string(),
        RuntimeHostKind::Managed => sanitize_runtime_identifier(&host.host_id),
    };
    format!("{prefix}_{host_suffix}")
}

fn parse_remote_ollama_probe_output(output: &str) -> RemoteHostRuntimeProbe {
    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed == "NO_OLLAMA_RUNTIME" || trimmed == "NO_OLLAMA_BIN" {
        return RemoteHostRuntimeProbe {
            status: "no_runtime".to_string(),
            detail: Some("No provider runtime detected over SSH.".to_string()),
            provider_kind: Some("Ollama".to_string()),
            supported_models: Vec::new(),
            default_model: None,
        };
    }

    let parsed = serde_json::from_str::<Value>(trimmed).or_else(|_| {
        let json_start = trimmed.find('{').or_else(|| trimmed.find('['));
        match json_start {
            Some(index) => serde_json::from_str::<Value>(&trimmed[index..]),
            None => Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing json payload",
            ))),
        }
    });

    match parsed {
        Ok(value) => {
            let mut supported_models = extract_supported_models(&value);
            if supported_models.is_empty() {
                supported_models = value
                    .get("models")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| {
                        entry
                            .get("name")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|name| !name.is_empty())
                            .map(str::to_string)
                            .or_else(|| {
                                entry
                                    .get("model")
                                    .and_then(Value::as_str)
                                    .map(str::trim)
                                    .filter(|name| !name.is_empty())
                                    .map(str::to_string)
                            })
                    })
                    .collect();
            }
            if supported_models.is_empty() {
                RemoteHostRuntimeProbe {
                    status: "no_runtime".to_string(),
                    detail: Some("No provider runtime detected over SSH.".to_string()),
                    provider_kind: Some("Ollama".to_string()),
                    supported_models,
                    default_model: None,
                }
            } else {
                RemoteHostRuntimeProbe {
                    status: "available".to_string(),
                    detail: Some(format!(
                        "Discovered {} model{} over SSH.",
                        supported_models.len(),
                        if supported_models.len() == 1 { "" } else { "s" }
                    )),
                    provider_kind: Some("Ollama".to_string()),
                    default_model: supported_models.first().cloned(),
                    supported_models,
                }
            }
        }
        Err(err) => RemoteHostRuntimeProbe {
            status: "probe_failed".to_string(),
            detail: Some(format!(
                "Remote runtime probe returned unreadable data: {err}"
            )),
            provider_kind: Some("Ollama".to_string()),
            supported_models: Vec::new(),
            default_model: None,
        },
    }
}

fn ssh_target_from_runtime_host(host: &RuntimeHostRecord) -> Option<String> {
    host.endpoint
        .trim()
        .strip_prefix("ssh://")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

async fn probe_remote_runtime_host(host: &RuntimeHostRecord) -> Option<RemoteHostRuntimeProbe> {
    let target = ssh_target_from_runtime_host(host)?;
    let output = tokio::task::spawn_blocking(move || {
        Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg(target)
            .arg("body=$(curl -fsS http://127.0.0.1:11434/api/tags 2>/dev/null || true); if [ -n \"$body\" ]; then printf '%s' \"$body\"; else printf 'NO_OLLAMA_RUNTIME'; fi")
            .output()
    })
    .await
    .ok()?;

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).to_string();
            Some(parse_remote_ollama_probe_output(&stdout))
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            Some(RemoteHostRuntimeProbe {
                status: "probe_failed".to_string(),
                detail: Some(if stderr.is_empty() {
                    "Remote runtime probe could not complete over SSH.".to_string()
                } else {
                    format!("Remote runtime probe could not complete over SSH: {stderr}")
                }),
                provider_kind: Some("Ollama".to_string()),
                supported_models: Vec::new(),
                default_model: None,
            })
        }
        Err(err) => Some(RemoteHostRuntimeProbe {
            status: "probe_failed".to_string(),
            detail: Some(format!("Remote runtime probe could not start: {err}")),
            provider_kind: Some("Ollama".to_string()),
            supported_models: Vec::new(),
            default_model: None,
        }),
    }
}

fn apply_remote_probe_to_runtime_host(
    host: &RuntimeHostRecord,
    probe: &RemoteHostRuntimeProbe,
) -> RuntimeHostRecord {
    let mut next = host.clone();
    next.updated_at = Some(now_iso());
    next.health = Some(serde_json::json!({
        "status": probe.status,
        "detail": probe.detail,
        "providerKind": probe.provider_kind,
        "modelCount": probe.supported_models.len(),
    }));
    next.metadata
        .insert("remoteDiscoveryStatus".to_string(), probe.status.clone());
    if let Some(detail) = probe.detail.as_ref() {
        next.metadata
            .insert("remoteDiscoveryDetail".to_string(), detail.clone());
    }
    if let Some(provider_kind) = probe.provider_kind.as_ref() {
        next.metadata.insert(
            "remoteDiscoveryProviderKind".to_string(),
            provider_kind.clone(),
        );
    }
    if let Some(default_model) = probe.default_model.as_ref() {
        next.metadata.insert(
            "remoteDiscoveryDefaultModel".to_string(),
            default_model.clone(),
        );
    }
    if probe.status == "available"
        && !next
            .capabilities
            .iter()
            .any(|capability| capability.eq_ignore_ascii_case("ollama"))
    {
        next.capabilities.push("ollama".to_string());
    }
    next
}

fn remote_discovery_record_from_runtime_host(
    host: &RuntimeHostRecord,
    probe: &RemoteHostRuntimeProbe,
) -> ProviderDiscoveryRecord {
    let provider_id =
        provider_id_for_runtime_host(probe.provider_kind.as_deref().unwrap_or("runtime"), host);
    let display_name = format!(
        "{} · {}",
        probe.provider_kind.as_deref().unwrap_or("Runtime"),
        host.name,
    );
    let mut metadata = BTreeMap::new();
    metadata.insert("displayName".to_string(), display_name);
    metadata.insert("hostId".to_string(), host.host_id.clone());
    metadata.insert("executionTransport".to_string(), "ssh".to_string());
    metadata.insert("executionReady".to_string(), "false".to_string());
    metadata.insert("remoteDiscoveryStatus".to_string(), probe.status.clone());
    if let Some(detail) = probe.detail.as_ref() {
        metadata.insert("remoteDiscoveryDetail".to_string(), detail.clone());
    }

    ProviderDiscoveryRecord {
        provider_id: provider_id.clone(),
        provider_type: "Llm".to_string(),
        provider_kind: probe.provider_kind.clone(),
        endpoint: host.endpoint.clone(),
        default_model: probe.default_model.clone(),
        supported_models: probe.supported_models.clone(),
        adapter_health: None,
        adapter_health_error: None,
        openapi_paths: Vec::new(),
        upstream_models_error: None,
        fail_mode: None,
        topology: Some(ProviderRuntimeContext {
            family_id: "ollama".to_string(),
            profile_id: probe.default_model.clone(),
            instance_id: format!(
                "{}__{}",
                provider_id,
                sanitize_runtime_identifier(&host.endpoint)
            ),
            device_id: host.device_id.clone(),
            environment_id: host.environment_id.clone(),
            locality_kind: host.locality_kind,
            last_seen_at: now_iso(),
            discovery_source: Some("ssh_host_probe".to_string()),
        }),
        updated_at: Some(now_iso()),
        metadata,
    }
}

pub(crate) async fn probe_remote_runtime_hosts(
    state: &ProviderRegistryState,
) -> (Vec<RuntimeHostRecord>, Vec<ProviderDiscoveryRecord>) {
    let mut runtime_hosts = Vec::with_capacity(state.runtime_hosts.len());
    let mut discovery_records = Vec::new();

    for host in &state.runtime_hosts {
        let host_auth = state.auth_bindings.iter().find(|binding| {
            binding.target_kind == AuthBindingTargetKind::Host && binding.target_id == host.host_id
        });
        let is_ssh_host = matches!(
            host_auth.map(|binding| &binding.auth_type),
            Some(AuthBindingType::SshKey) | Some(AuthBindingType::SshPassword)
        ) || host.endpoint.trim().starts_with("ssh://");

        if !is_ssh_host || !runtime_host_allows_remote_discovery(host) {
            runtime_hosts.push(host.clone());
            continue;
        }

        match probe_remote_runtime_host(host).await {
            Some(probe) => {
                let updated_host = apply_remote_probe_to_runtime_host(host, &probe);
                if probe.status == "available" {
                    discovery_records.push(remote_discovery_record_from_runtime_host(
                        &updated_host,
                        &probe,
                    ));
                }
                runtime_hosts.push(updated_host);
            }
            None => runtime_hosts.push(host.clone()),
        }
    }

    (runtime_hosts, discovery_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::provider_runtime::config::{
        AuthBindingRecord, AuthBindingTargetKind, AuthBindingType, ProviderLocalityKind,
        ProviderRuntimeConfig, ProviderRuntimeSettings, RuntimeHostKind, RuntimeHostRecord,
    };
    use std::collections::BTreeMap;

    #[test]
    fn provider_client_config_uses_bound_secret() {
        let settings = ProviderRuntimeSettings {
            provider_id: "provider.example".to_string(),
            name: "Provider Example".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("Example".to_string()),
            host_id: None,
            enabled: true,
            base_url: "https://example.invalid".to_string(),
            default_model: "model-a".to_string(),
            adapter_set_ref: None,
            auth_binding_id: Some("auth.example".to_string()),
            provider_family_id: None,
            profile_id: None,
            instance_id: None,
            device_id: None,
            environment_id: None,
            locality_kind: None,
            discovery_source: None,
            batch_policy: None,
            updated_at: None,
            supported_models: Vec::new(),
            metadata: BTreeMap::new(),
        };
        let mut lookup = BTreeMap::new();
        lookup.insert(
            "auth.example".to_string(),
            AuthBindingRecord {
                auth_binding_id: "auth.example".to_string(),
                target_kind: AuthBindingTargetKind::Provider,
                target_id: "provider.example".to_string(),
                auth_type: AuthBindingType::ApiKey,
                label: None,
                source: None,
                secret: "super-secret".to_string(),
                has_secret: true,
                created_at: None,
                updated_at: None,
                metadata: BTreeMap::new(),
            },
        );

        let cfg = provider_client_config(&settings, &lookup);
        assert_eq!(cfg.api_key, "super-secret");
        assert!(cfg.enabled);
        assert_eq!(cfg.default_model, "model-a");
    }

    #[tokio::test]
    async fn resolve_provider_supported_models_returns_default_when_disabled() {
        let cfg = ProviderRuntimeConfig {
            enabled: false,
            base_url: "https://example.invalid".to_string(),
            api_key: String::new(),
            request_timeout: std::time::Duration::from_secs(1),
            fail_mode: ProviderRuntimeFailMode::Fallback,
            max_tool_steps: 1,
            default_model: "fallback-model".to_string(),
        };

        let (supported_models, adapter_health, adapter_health_error, openapi_paths, upstream_err) =
            resolve_provider_supported_models(&cfg).await;

        assert_eq!(supported_models, vec!["fallback-model".to_string()]);
        assert!(adapter_health.is_none());
        assert!(adapter_health_error.is_none());
        assert!(openapi_paths.is_empty());
        assert!(upstream_err.is_none());
    }

    #[test]
    fn parse_remote_ollama_probe_output_reports_missing_runtime() {
        let probe = parse_remote_ollama_probe_output("NO_OLLAMA_RUNTIME");

        assert_eq!(probe.status, "no_runtime");
        assert!(probe.supported_models.is_empty());
        assert_eq!(
            probe.detail.as_deref(),
            Some("No provider runtime detected over SSH.")
        );
    }

    #[test]
    fn parse_remote_ollama_probe_output_extracts_models_when_present() {
        let probe = parse_remote_ollama_probe_output(
            r#"{"models":[{"name":"llama3.1:8b"},{"name":"qwen3:latest"}]}"#,
        );

        assert_eq!(probe.status, "available");
        assert_eq!(
            probe.supported_models,
            vec!["llama3.1:8b".to_string(), "qwen3:latest".to_string()]
        );
        assert_eq!(probe.default_model.as_deref(), Some("llama3.1:8b"));
    }

    #[test]
    fn remote_discovery_record_from_runtime_host_uses_host_scoped_provider_identity() {
        let host = RuntimeHostRecord {
            host_id: "host.vps.primary".to_string(),
            name: "Eudaemon Alpha VPS".to_string(),
            host_kind: RuntimeHostKind::Vps,
            endpoint: "ssh://root@204.168.175.150".to_string(),
            locality_kind: ProviderLocalityKind::Cloud,
            device_id: None,
            environment_id: Some("eudaemon-alpha".to_string()),
            health: None,
            capabilities: vec!["ssh".to_string()],
            remote_discovery_enabled: true,
            execution_routable: false,
            updated_at: None,
            metadata: BTreeMap::new(),
        };
        let probe = RemoteHostRuntimeProbe {
            status: "available".to_string(),
            detail: Some("Discovered 1 model over SSH.".to_string()),
            provider_kind: Some("Ollama".to_string()),
            supported_models: vec!["llama3.1:8b".to_string()],
            default_model: Some("llama3.1:8b".to_string()),
        };

        let discovery = remote_discovery_record_from_runtime_host(&host, &probe);

        assert_eq!(discovery.provider_id, "ollama_vps_primary");
        assert_eq!(discovery.endpoint, "ssh://root@204.168.175.150");
        assert_eq!(
            discovery.metadata.get("executionReady"),
            Some(&"false".to_string())
        );
        assert_eq!(
            discovery.metadata.get("hostId"),
            Some(&"host.vps.primary".to_string())
        );
    }

    #[test]
    fn discovery_record_preserves_adapter_error_when_default_model_is_used_as_fallback() {
        let settings = ProviderRuntimeSettings {
            provider_id: "ollama_local".to_string(),
            name: "Ollama Local".to_string(),
            provider_type: "Llm".to_string(),
            provider_kind: Some("Ollama".to_string()),
            host_id: Some("host.local.primary".to_string()),
            enabled: true,
            base_url: "http://127.0.0.1:11434".to_string(),
            default_model: "llama3.1:8b".to_string(),
            adapter_set_ref: None,
            auth_binding_id: None,
            provider_family_id: Some("ollama".to_string()),
            profile_id: None,
            instance_id: None,
            device_id: None,
            environment_id: None,
            locality_kind: Some(ProviderLocalityKind::Local),
            discovery_source: Some("auto".to_string()),
            batch_policy: None,
            updated_at: Some("2026-04-01T00:00:00Z".to_string()),
            supported_models: Vec::new(),
            metadata: BTreeMap::new(),
        };
        let diagnostics = ProviderDiagnostics {
            default_model: "llama3.1:8b".to_string(),
            supported_models: vec!["llama3.1:8b".to_string()],
            adapter_health: None,
            adapter_health_error: Some("provider_runtime_client_init_failed".to_string()),
            openapi_paths: Vec::new(),
            upstream_models_error: None,
            fail_mode: Some("fallback".to_string()),
        };

        let record = discovery_record_from_provider_settings(&settings, Some(&diagnostics));

        assert_eq!(
            record.adapter_health_error.as_deref(),
            Some("provider_runtime_client_init_failed")
        );
        assert_eq!(record.supported_models, vec!["llama3.1:8b".to_string()]);
    }
}
