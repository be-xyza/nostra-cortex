use crate::gateway::provider_admin::contracts::{
    AuthBindingInventoryResponse, ExecutionBindingStatusResponse,
    OperatorProviderInventoryResponse, ProviderDiscoveryInventoryResponse,
    ProviderRuntimeStatusResponse, RuntimeHostInventoryResponse, SystemProviderRecord,
    SystemProvidersResponse, SystemRuntimeHostResponse, map_auth_binding_response,
    map_discovery_response, map_execution_binding_response, map_runtime_host_response,
};
use crate::gateway::provider_admin::records::{
    apply_provider_discovery, binding_ids_for_provider, ephemeral_provider_from_discovery,
    provider_record_from_settings,
};
use crate::services::provider_runtime::client::ProviderRuntimeClient;
use crate::services::provider_runtime::config::{
    DEFAULT_LLM_BINDING_ID, ProviderRegistryState, ProviderRuntimeFailMode,
    provider_runtime_config_from_env, provider_runtime_settings_from_resolved_state,
    resolve_provider_runtime_state,
};
use crate::services::provider_runtime::discovery::{
    ProviderDiagnostics, collect_live_discovery_records, resolve_provider_supported_models,
};
use crate::services::secret_redaction::{redact_json_value, redact_runtime_text};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderAdminSnapshot {
    pub(crate) providers: Vec<SystemProviderRecord>,
    pub(crate) runtime_hosts: Vec<SystemRuntimeHostResponse>,
    pub(crate) auth_bindings:
        Vec<crate::gateway::provider_admin::contracts::SystemAuthBindingResponse>,
    pub(crate) execution_bindings:
        Vec<crate::gateway::provider_admin::contracts::SystemProviderBindingResponse>,
    pub(crate) discovery_records:
        Vec<crate::gateway::provider_admin::contracts::SystemProviderDiscoveryResponse>,
}

pub(crate) async fn build_provider_admin_snapshot() -> ProviderAdminSnapshot {
    let cfg = provider_runtime_config_from_env();
    let resolved = resolve_provider_runtime_state();
    let state = crate::services::provider_runtime::config::load_provider_registry_state()
        .unwrap_or_else(|_| ProviderRegistryState::default());
    let auth_binding_lookup = state
        .auth_bindings
        .iter()
        .cloned()
        .map(|binding| (binding.auth_binding_id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let (
        supported_models,
        adapter_health,
        adapter_health_error,
        openapi_paths,
        upstream_models_error,
    ) = resolve_provider_supported_models(&cfg).await;
    let diagnostics = ProviderDiagnostics {
        default_model: cfg.default_model.clone(),
        supported_models,
        adapter_health,
        adapter_health_error,
        openapi_paths,
        upstream_models_error,
        fail_mode: Some(match cfg.fail_mode {
            ProviderRuntimeFailMode::Fallback => "fallback".to_string(),
            ProviderRuntimeFailMode::FailClosed => "fail_closed".to_string(),
        }),
    };
    let active_provider_id = resolved.provider_id.clone();
    let active_settings = state
        .providers
        .iter()
        .find(|item| item.provider_id == active_provider_id)
        .cloned()
        .unwrap_or_else(|| provider_runtime_settings_from_resolved_state(&resolved));
    let live_discovery_records = collect_live_discovery_records(
        &state,
        &active_provider_id,
        &active_settings,
        &diagnostics,
        &auth_binding_lookup,
    )
    .await;
    let discovery_lookup = live_discovery_records
        .iter()
        .cloned()
        .map(|record| (record.provider_id.clone(), record))
        .collect::<BTreeMap<_, _>>();

    let mut providers = state
        .providers
        .iter()
        .map(|settings| provider_record_from_settings(settings, &auth_binding_lookup, None))
        .collect::<Vec<_>>();
    if !providers
        .iter()
        .any(|provider| provider.id == active_provider_id)
    {
        providers.push(provider_record_from_settings(
            &active_settings,
            &auth_binding_lookup,
            None,
        ));
    }
    for discovery in &live_discovery_records {
        if !providers
            .iter()
            .any(|provider| provider.id == discovery.provider_id)
        {
            providers.push(provider_record_from_settings(
                &ephemeral_provider_from_discovery(discovery),
                &auth_binding_lookup,
                None,
            ));
        }
    }

    for provider in &mut providers {
        if let Some(discovery) = discovery_lookup.get(&provider.id) {
            apply_provider_discovery(provider, discovery);
        }
        provider.binding_ids = binding_ids_for_provider(&state.execution_bindings, &provider.id);
        if provider.id == active_provider_id {
            provider.auth_source = resolved.auth_source.clone();
            provider.auth_binding_id = resolved.auth_binding_id.clone();
            provider.auth_type = Some(match resolved.auth_type {
                crate::services::provider_runtime::config::AuthBindingType::None => {
                    "none".to_string()
                }
                crate::services::provider_runtime::config::AuthBindingType::ApiKey => {
                    "api_key".to_string()
                }
                crate::services::provider_runtime::config::AuthBindingType::BearerToken => {
                    "bearer_token".to_string()
                }
                crate::services::provider_runtime::config::AuthBindingType::Pat => {
                    "pat".to_string()
                }
                crate::services::provider_runtime::config::AuthBindingType::SshKey => {
                    "ssh_key".to_string()
                }
                crate::services::provider_runtime::config::AuthBindingType::SshPassword => {
                    "ssh_password".to_string()
                }
            });
            provider.auth_state = match resolved.auth_type {
                crate::services::provider_runtime::config::AuthBindingType::None => {
                    "not_required".to_string()
                }
                _ if resolved.has_auth_secret => "linked".to_string(),
                _ if resolved.auth_binding_id.is_some() => "missing".to_string(),
                _ => "inherited".to_string(),
            };
        }
    }

    providers.sort_by(|left, right| {
        left.provider_type
            .cmp(&right.provider_type)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut runtime_hosts = state
        .runtime_hosts
        .iter()
        .map(map_runtime_host_response)
        .collect::<Vec<_>>();
    runtime_hosts.sort_by(|left, right| left.host_id.cmp(&right.host_id));
    let mut auth_bindings = state
        .auth_bindings
        .iter()
        .map(map_auth_binding_response)
        .collect::<Vec<_>>();
    auth_bindings.sort_by(|left, right| left.label.cmp(&right.label));
    let mut execution_bindings = state
        .execution_bindings
        .iter()
        .map(map_execution_binding_response)
        .collect::<Vec<_>>();
    execution_bindings.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    let mut discovery_records = live_discovery_records
        .iter()
        .map(map_discovery_response)
        .collect::<Vec<_>>();
    discovery_records.sort_by(|left, right| left.provider_id.cmp(&right.provider_id));

    ProviderAdminSnapshot {
        providers,
        runtime_hosts,
        auth_bindings,
        execution_bindings,
        discovery_records,
    }
}

pub(crate) async fn build_system_providers_response() -> SystemProvidersResponse {
    let snapshot = build_provider_admin_snapshot().await;
    SystemProvidersResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        providers: snapshot.providers,
        runtime_hosts: snapshot.runtime_hosts,
        auth_bindings: snapshot.auth_bindings,
        execution_bindings: snapshot.execution_bindings,
        discovery_records: snapshot.discovery_records,
    }
}

pub(crate) async fn build_provider_inventory_response() -> OperatorProviderInventoryResponse {
    let snapshot = build_provider_admin_snapshot().await;
    OperatorProviderInventoryResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        providers: snapshot.providers,
    }
}

pub(crate) async fn build_runtime_host_inventory_response() -> RuntimeHostInventoryResponse {
    let snapshot = build_provider_admin_snapshot().await;
    RuntimeHostInventoryResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        runtime_hosts: snapshot.runtime_hosts,
    }
}

pub(crate) async fn build_auth_binding_inventory_response() -> AuthBindingInventoryResponse {
    let snapshot = build_provider_admin_snapshot().await;
    AuthBindingInventoryResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        auth_bindings: snapshot.auth_bindings,
    }
}

pub(crate) async fn build_execution_binding_status_response() -> ExecutionBindingStatusResponse {
    let snapshot = build_provider_admin_snapshot().await;
    ExecutionBindingStatusResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        execution_bindings: snapshot.execution_bindings,
    }
}

pub(crate) async fn build_provider_discovery_inventory_response()
-> ProviderDiscoveryInventoryResponse {
    let snapshot = build_provider_admin_snapshot().await;
    ProviderDiscoveryInventoryResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::gateway::server::now_iso(),
        discovery_records: snapshot.discovery_records,
    }
}

pub(crate) async fn build_provider_runtime_status_response() -> ProviderRuntimeStatusResponse {
    let cfg = provider_runtime_config_from_env();
    let resolved = resolve_provider_runtime_state();
    let fail_mode = match cfg.fail_mode {
        ProviderRuntimeFailMode::Fallback => "fallback".to_string(),
        ProviderRuntimeFailMode::FailClosed => "fail_closed".to_string(),
    };

    if !cfg.enabled {
        return ProviderRuntimeStatusResponse {
            enabled: false,
            base_url: cfg.base_url,
            binding_id: DEFAULT_LLM_BINDING_ID.to_string(),
            fail_mode,
            model: cfg.default_model,
            provider_id: resolved.provider_id,
            ..ProviderRuntimeStatusResponse::default()
        };
    }

    let client = match ProviderRuntimeClient::new(cfg.clone()) {
        Ok(client) => client,
        Err(err) => {
            return ProviderRuntimeStatusResponse {
                enabled: true,
                base_url: cfg.base_url,
                binding_id: DEFAULT_LLM_BINDING_ID.to_string(),
                fail_mode,
                model: cfg.default_model,
                provider_id: resolved.provider_id,
                adapter_health_error: Some(redact_runtime_text(&err)),
                ..ProviderRuntimeStatusResponse::default()
            };
        }
    };

    let (adapter_health, adapter_health_error) = match client.health_adapter().await {
        Ok(value) => (Some(redact_json_value(&value)), None),
        Err(err) => (None, Some(redact_runtime_text(&err))),
    };
    let (openapi_paths, openapi_error) = match client.openapi_paths().await {
        Ok(value) => (Some(value), None),
        Err(err) => (None, Some(redact_runtime_text(&err))),
    };
    let (upstream_models, upstream_models_error) = match client.health_upstream_models().await {
        Ok(value) => (Some(redact_json_value(&value)), None),
        Err(err) => (None, Some(redact_runtime_text(&err))),
    };

    ProviderRuntimeStatusResponse {
        enabled: true,
        base_url: cfg.base_url,
        binding_id: DEFAULT_LLM_BINDING_ID.to_string(),
        fail_mode,
        model: cfg.default_model,
        provider_id: resolved.provider_id,
        adapter_health,
        adapter_health_error,
        openapi_paths,
        openapi_error,
        upstream_models,
        upstream_models_error,
    }
}
