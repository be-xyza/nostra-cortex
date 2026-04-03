use crate::gateway::provider_admin::contracts::SystemProviderRecord;
use crate::services::cortex_ux::now_iso;
use crate::services::provider_runtime::config::{
    AuthBindingRecord, AuthBindingType, ExecutionBindingRecord, ProviderDiscoveryRecord,
    ProviderRegistryState, ProviderRuntimeContext, ProviderRuntimeSettings, infer_provider_kind,
    infer_provider_locality_kind,
};
use crate::services::provider_runtime::discovery::ProviderDiagnostics;
use std::collections::BTreeMap;

pub(crate) fn normalize_provider_type(value: Option<&str>) -> String {
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

fn provider_metadata_from_settings(
    settings: &ProviderRuntimeSettings,
    context: &ProviderRuntimeContext,
) -> BTreeMap<String, String> {
    let mut metadata = settings.metadata.clone();
    metadata.insert("providerId".to_string(), settings.provider_id.clone());
    metadata.insert(
        "providerType".to_string(),
        normalize_provider_type(Some(&settings.provider_type)),
    );
    if let Some(provider_kind) = settings.provider_kind.as_ref() {
        metadata.insert("providerKind".to_string(), provider_kind.clone());
    }
    metadata.insert("familyId".to_string(), context.family_id.clone());
    if let Some(profile_id) = context.profile_id.as_ref() {
        metadata.insert("profileId".to_string(), profile_id.clone());
    }
    metadata.insert("instanceId".to_string(), context.instance_id.clone());
    if let Some(device_id) = context.device_id.as_ref() {
        metadata.insert("deviceId".to_string(), device_id.clone());
    }
    if let Some(environment_id) = context.environment_id.as_ref() {
        metadata.insert("environmentId".to_string(), environment_id.clone());
    }
    metadata.insert(
        "localityKind".to_string(),
        format!("{:?}", context.locality_kind),
    );
    metadata.insert("lastSeenAt".to_string(), context.last_seen_at.clone());
    if let Some(discovery_source) = context.discovery_source.as_ref() {
        metadata.insert("discoverySource".to_string(), discovery_source.clone());
    }
    metadata
}

pub(crate) fn binding_ids_for_provider(
    bindings: &[ExecutionBindingRecord],
    provider_id: &str,
) -> Vec<String> {
    bindings
        .iter()
        .filter(|binding| binding.bound_provider_id == provider_id)
        .map(|binding| binding.binding_id.clone())
        .collect()
}

pub(crate) fn apply_provider_discovery(
    record: &mut SystemProviderRecord,
    discovery: &ProviderDiscoveryRecord,
) {
    if record
        .default_model
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        record.default_model = discovery.default_model.clone();
    }
    if !discovery.supported_models.is_empty() {
        record.supported_models = discovery.supported_models.clone();
    }
    record.adapter_health = discovery.adapter_health.clone();
    record.adapter_health_error = discovery.adapter_health_error.clone();
    record.openapi_paths = discovery.openapi_paths.clone();
    record.upstream_models_error = discovery.upstream_models_error.clone();
    record.fail_mode = discovery.fail_mode.clone();
    if discovery.topology.is_some() {
        record.topology = discovery.topology.clone();
    }
}

pub(crate) fn ephemeral_provider_from_discovery(
    record: &ProviderDiscoveryRecord,
) -> ProviderRuntimeSettings {
    let provider_type = normalize_provider_type(Some(&record.provider_type));
    let provider_kind = record.provider_kind.clone();
    let default_model = record
        .default_model
        .clone()
        .or_else(|| record.supported_models.first().cloned())
        .unwrap_or_default();
    let topology = record.topology.clone();
    let display_name = record
        .metadata
        .get("displayName")
        .cloned()
        .or_else(|| record.provider_kind.clone())
        .unwrap_or_else(|| provider_type.clone());
    let host_id = record.metadata.get("hostId").cloned().or_else(|| {
        topology.as_ref().map(|item| {
            crate::services::provider_runtime::config::default_runtime_host_id(
                &record.provider_id,
                &record.endpoint,
                item.locality_kind,
                item.environment_id.as_deref(),
            )
        })
    });
    let execution_ready = record
        .metadata
        .get("executionReady")
        .map(|value| !value.eq_ignore_ascii_case("false"))
        .unwrap_or_else(|| !record.endpoint.trim().starts_with("ssh://"));
    ProviderRuntimeSettings {
        provider_id: record.provider_id.clone(),
        name: display_name,
        provider_type: provider_type.clone(),
        provider_kind,
        host_id,
        enabled: execution_ready,
        base_url: record.endpoint.clone(),
        default_model,
        adapter_set_ref: None,
        auth_binding_id: None,
        provider_family_id: topology.as_ref().map(|item| item.family_id.clone()),
        profile_id: topology.as_ref().and_then(|item| item.profile_id.clone()),
        instance_id: topology.as_ref().map(|item| item.instance_id.clone()),
        device_id: topology.as_ref().and_then(|item| item.device_id.clone()),
        environment_id: topology
            .as_ref()
            .and_then(|item| item.environment_id.clone()),
        locality_kind: topology.as_ref().map(|item| item.locality_kind),
        discovery_source: topology
            .as_ref()
            .and_then(|item| item.discovery_source.clone()),
        batch_policy: None,
        updated_at: record.updated_at.clone(),
        supported_models: Vec::new(),
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn upsert_provider_record_in_state(
    state: &mut ProviderRegistryState,
    settings: ProviderRuntimeSettings,
) {
    if let Some(existing) = state
        .providers
        .iter_mut()
        .find(|item| item.provider_id == settings.provider_id)
    {
        *existing = settings;
    } else {
        state.providers.push(settings);
    }
}

pub(crate) fn blank_provider_runtime_settings(
    provider_id: &str,
    provider_type: &str,
) -> ProviderRuntimeSettings {
    let normalized_type = normalize_provider_type(Some(provider_type));
    ProviderRuntimeSettings {
        provider_id: provider_id.to_string(),
        name: String::new(),
        provider_type: normalized_type.clone(),
        provider_kind: None,
        host_id: None,
        enabled: true,
        base_url: String::new(),
        default_model: String::new(),
        adapter_set_ref: None,
        auth_binding_id: None,
        provider_family_id: Some(normalized_type.to_ascii_lowercase()),
        profile_id: None,
        instance_id: None,
        device_id: None,
        environment_id: None,
        locality_kind: None,
        discovery_source: Some("registry".to_string()),
        batch_policy: None,
        updated_at: Some(now_iso()),
        supported_models: Vec::new(),
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn provider_record_from_settings(
    settings: &ProviderRuntimeSettings,
    auth_bindings: &BTreeMap<String, AuthBindingRecord>,
    diagnostics: Option<&ProviderDiagnostics>,
) -> SystemProviderRecord {
    let provider_type = normalize_provider_type(Some(&settings.provider_type));
    let context = provider_runtime_context_from_settings(settings);
    let auth_binding = settings
        .auth_binding_id
        .as_ref()
        .and_then(|auth_binding_id| auth_bindings.get(auth_binding_id));
    let provider_family = settings.provider_kind.clone();
    let host_id = settings.host_id.clone().unwrap_or_else(|| {
        let locality = context.locality_kind;
        format!(
            "{}",
            crate::services::provider_runtime::config::default_runtime_host_id(
                &settings.provider_id,
                &settings.base_url,
                locality,
                settings.environment_id.as_deref(),
            )
        )
    });
    let auth_type = auth_binding
        .map(|binding| binding.auth_type.clone())
        .unwrap_or_else(|| {
            if settings
                .provider_kind
                .as_deref()
                .map(|kind| kind.eq_ignore_ascii_case("ollama"))
                .unwrap_or(false)
            {
                AuthBindingType::None
            } else {
                AuthBindingType::ApiKey
            }
        });
    let has_auth_secret = auth_binding
        .map(|binding| binding.has_secret || !binding.secret.trim().is_empty())
        .unwrap_or(false);
    let auth_state = match auth_type {
        AuthBindingType::None => "not_required".to_string(),
        _ if has_auth_secret => "linked".to_string(),
        _ if settings
            .auth_binding_id
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty() =>
        {
            "inherited".to_string()
        }
        _ => "missing".to_string(),
    };
    let default_model = if provider_type == "Llm" {
        if settings.default_model.trim().is_empty() {
            diagnostics
                .map(|item| item.default_model.clone())
                .filter(|value| !value.trim().is_empty())
        } else {
            Some(settings.default_model.clone())
        }
    } else {
        Some(settings.default_model.clone()).filter(|value| !value.trim().is_empty())
    };
    let mut supported_models = if !settings.supported_models.is_empty() {
        settings.supported_models.clone()
    } else if provider_type == "Llm" {
        diagnostics
            .map(|item| item.supported_models.clone())
            .unwrap_or_default()
    } else {
        settings.supported_models.clone()
    };
    if supported_models.is_empty() {
        if let Some(model) = default_model.clone() {
            supported_models.push(model);
        }
    }
    let is_llm_provider = provider_type == "Llm";

    SystemProviderRecord {
        id: settings.provider_id.clone(),
        name: settings.name.clone(),
        provider_type,
        provider_family,
        host_id,
        endpoint: settings.base_url.clone(),
        is_active: settings.enabled,
        priority: 1,
        default_model,
        supported_models,
        adapter_health: if is_llm_provider {
            diagnostics.and_then(|item| item.adapter_health.clone())
        } else {
            None
        },
        adapter_health_error: if is_llm_provider {
            diagnostics.and_then(|item| item.adapter_health_error.clone())
        } else {
            None
        },
        openapi_paths: if is_llm_provider {
            diagnostics
                .map(|item| item.openapi_paths.clone())
                .unwrap_or_default()
        } else {
            Vec::new()
        },
        upstream_models_error: if is_llm_provider {
            diagnostics.and_then(|item| item.upstream_models_error.clone())
        } else {
            None
        },
        fail_mode: if is_llm_provider {
            diagnostics.and_then(|item| item.fail_mode.clone())
        } else {
            None
        },
        auth_mode: Some(match auth_type {
            AuthBindingType::None => "none".to_string(),
            _ => "api_key".to_string(),
        }),
        auth_state,
        auth_source: auth_binding.and_then(|binding| binding.source.clone()),
        auth_binding_id: settings.auth_binding_id.clone(),
        auth_type: Some(match auth_type {
            AuthBindingType::None => "none".to_string(),
            AuthBindingType::ApiKey => "api_key".to_string(),
            AuthBindingType::BearerToken => "bearer_token".to_string(),
            AuthBindingType::Pat => "pat".to_string(),
            AuthBindingType::SshKey => "ssh_key".to_string(),
            AuthBindingType::SshPassword => "ssh_password".to_string(),
        }),
        binding_ids: Vec::new(),
        topology: Some(context.clone()),
        batch_policy: settings.batch_policy.clone(),
        metadata: provider_metadata_from_settings(settings, &context),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_provider_type_defaults_to_llm() {
        assert_eq!(normalize_provider_type(None), "Llm");
        assert_eq!(normalize_provider_type(Some("  ")), "Llm");
        assert_eq!(normalize_provider_type(Some("OpenAI")), "OpenAI");
    }
}
