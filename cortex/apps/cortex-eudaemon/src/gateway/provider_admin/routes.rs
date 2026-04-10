use crate::gateway::provider_admin::contracts::{
    CreateAuthBindingRequest, PutSystemProviderBindingRequest, PutSystemProviderRequest,
    UpdateAuthBindingRequest,
};
use crate::gateway::provider_admin::records::{
    blank_provider_runtime_settings, ephemeral_provider_from_discovery, normalize_provider_type,
    upsert_provider_record_in_state,
};
use crate::gateway::provider_admin::service::{
    build_auth_binding_inventory_response, build_execution_binding_status_response,
    build_provider_discovery_inventory_response, build_provider_inventory_response,
    build_provider_runtime_status_response, build_runtime_host_inventory_response,
    build_system_providers_response,
};
use crate::gateway::provider_admin::state::{
    link_provider_auth_binding, resolve_provider_probe_api_key,
};
use crate::services::provider_probe::{ProviderProbeRequest, validate_provider_probe};
use crate::services::provider_runtime::config::{
    AuthBindingRecord, AuthBindingTargetKind, AuthBindingType, DEFAULT_LLM_BINDING_ID,
    ExecutionBindingRecord, ProviderDiscoveryRecord, ProviderRegistryState,
    ProviderRuntimeFailMode, ProviderRuntimeSettings, infer_provider_kind,
    infer_provider_locality_kind, load_provider_registry_state, provider_runtime_config_from_env,
    provider_runtime_settings_from_resolved_state, resolve_provider_runtime_state,
    save_provider_registry_state,
};
use crate::services::provider_runtime::discovery::{
    ProviderDiagnostics, collect_live_discovery_records, probe_remote_runtime_hosts,
    resolve_provider_supported_models, upsert_discovery_record,
};
use crate::services::provider_runtime::policy::provider_execution_eligibility;
use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;
use std::collections::BTreeMap;

async fn enforce_provider_operator_read(
    headers: &HeaderMap,
    route_id: &'static str,
    capability: &'static str,
    error_code: &'static str,
    message: &'static str,
) -> Result<(), axum::response::Response> {
    crate::gateway::server::enforce_role_authorization(
        headers,
        route_id,
        None,
        capability,
        "read",
        "operator",
        vec![],
        false,
        vec![],
        error_code,
        message,
    )
    .await
    .map(|_| ())
}

pub(crate) async fn get_system_providers(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_providers",
        "capability:system_provider_read",
        "SYSTEM_PROVIDERS_READ_FORBIDDEN",
        "Operator role or higher is required to inspect provider inventory.",
    )
    .await
    {
        return response;
    }

    Json(build_system_providers_response().await).into_response()
}

pub(crate) async fn get_system_provider_inventory(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_provider_inventory",
        "capability:system_provider_read",
        "SYSTEM_PROVIDER_INVENTORY_READ_FORBIDDEN",
        "Operator role or higher is required to inspect provider inventory.",
    )
    .await
    {
        return response;
    }

    Json(build_provider_inventory_response().await).into_response()
}

pub(crate) async fn get_system_runtime_hosts(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_runtime_hosts",
        "capability:system_provider_read",
        "SYSTEM_RUNTIME_HOSTS_READ_FORBIDDEN",
        "Operator role or higher is required to inspect runtime host inventory.",
    )
    .await
    {
        return response;
    }

    Json(build_runtime_host_inventory_response().await).into_response()
}

pub(crate) async fn get_system_auth_bindings(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_auth_bindings",
        "capability:system_provider_read",
        "SYSTEM_AUTH_BINDINGS_READ_FORBIDDEN",
        "Operator role or higher is required to inspect provider auth bindings.",
    )
    .await
    {
        return response;
    }

    Json(build_auth_binding_inventory_response().await).into_response()
}

pub(crate) async fn get_system_execution_bindings(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_execution_bindings",
        "capability:system_provider_read",
        "SYSTEM_EXECUTION_BINDINGS_READ_FORBIDDEN",
        "Operator role or higher is required to inspect execution bindings.",
    )
    .await
    {
        return response;
    }

    Json(build_execution_binding_status_response().await).into_response()
}

pub(crate) async fn get_system_provider_discovery(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = enforce_provider_operator_read(
        &headers,
        "get_system_provider_discovery",
        "capability:system_provider_read",
        "SYSTEM_PROVIDER_DISCOVERY_READ_FORBIDDEN",
        "Operator role or higher is required to inspect provider discovery records.",
    )
    .await
    {
        return response;
    }

    Json(build_provider_discovery_inventory_response().await).into_response()
}

pub(crate) async fn post_system_providers_discover(headers: HeaderMap) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "post_system_providers_discover",
        None,
        "capability:system_provider_update",
        "mutate",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_PROVIDER_DISCOVERY_FORBIDDEN",
        "Operator role or higher is required to discover providers.",
    )
    .await
    {
        return response;
    }

    let mut state =
        load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let resolved = resolve_provider_runtime_state();
    let cfg = provider_runtime_config_from_env();
    let auth_binding_lookup = state
        .auth_bindings
        .iter()
        .cloned()
        .map(|binding| (binding.auth_binding_id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let active_provider_id = state
        .execution_bindings
        .iter()
        .find(|binding| binding.binding_id == DEFAULT_LLM_BINDING_ID)
        .map(|binding| binding.bound_provider_id.clone())
        .unwrap_or_else(|| resolved.provider_id.clone());
    let active_settings = state
        .providers
        .iter()
        .find(|item| item.provider_id == active_provider_id)
        .cloned()
        .unwrap_or_else(|| provider_runtime_settings_from_resolved_state(&resolved));
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
    let discovery_records = collect_live_discovery_records(
        &state,
        &active_provider_id,
        &active_settings,
        &diagnostics,
        &auth_binding_lookup,
    )
    .await;
    let (probed_runtime_hosts, remote_host_discovery_records) =
        probe_remote_runtime_hosts(&state).await;
    let mut merged_discovery_records = discovery_records.clone();
    for record in remote_host_discovery_records {
        upsert_discovery_record(&mut merged_discovery_records, record);
    }
    state.runtime_hosts = probed_runtime_hosts;
    state.discovery = merged_discovery_records.clone();
    for discovery in merged_discovery_records {
        if !state
            .providers
            .iter()
            .any(|provider| provider.provider_id == discovery.provider_id)
        {
            upsert_provider_record_in_state(
                &mut state,
                ephemeral_provider_from_discovery(&discovery),
            );
        }
    }

    if let Err(err) = save_provider_registry_state(&state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSTEM_PROVIDER_DISCOVERY_PERSIST_FAILED",
            "Unable to persist discovered providers.",
            Some(json!({ "reason": err })),
        );
    }

    Json(build_system_providers_response().await).into_response()
}

pub(crate) async fn post_system_provider_validate(
    headers: HeaderMap,
    Json(mut payload): Json<ProviderProbeRequest>,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "post_system_provider_validate",
        None,
        "capability:system_provider_update",
        "read",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_PROVIDER_VALIDATE_FORBIDDEN",
        "Operator role or higher is required to validate provider settings.",
    )
    .await
    {
        return response;
    }

    if payload.base_url.trim().is_empty() {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_VALIDATE_INVALID",
            "baseUrl is required.",
            None,
        );
    }
    let state = load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let mut provider_bindings = state
        .providers
        .iter()
        .filter_map(|provider| {
            provider
                .auth_binding_id
                .as_ref()
                .map(|binding_id| (provider.provider_id.clone(), binding_id.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let resolved_runtime = resolve_provider_runtime_state();
    if let Some(binding_id) = resolved_runtime.auth_binding_id.clone() {
        provider_bindings.insert(resolved_runtime.provider_id.clone(), binding_id);
    }
    let auth_secrets = state
        .auth_bindings
        .iter()
        .map(|binding| (binding.auth_binding_id.clone(), binding.secret.clone()))
        .collect::<BTreeMap<_, _>>();
    let resolved_api_key = resolve_provider_probe_api_key(
        Some(payload.api_key.as_str()),
        payload.use_stored_auth,
        payload.provider_id.as_deref(),
        payload.auth_binding_id.as_deref(),
        &provider_bindings,
        &auth_secrets,
    )
    .or_else(|| {
        if payload.use_stored_auth {
            let env_key = provider_runtime_config_from_env().api_key;
            if env_key.trim().is_empty() {
                None
            } else {
                Some(env_key)
            }
        } else {
            None
        }
    });

    if resolved_api_key.as_deref().unwrap_or("").trim().is_empty() {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_VALIDATE_INVALID",
            "apiKey is required unless a saved auth binding can be reused.",
            None,
        );
    }
    payload.api_key = resolved_api_key.unwrap_or_default();

    Json(validate_provider_probe(payload).await).into_response()
}

pub(crate) async fn put_system_provider(
    headers: HeaderMap,
    Path(provider_id): Path<String>,
    Json(payload): Json<PutSystemProviderRequest>,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "put_system_provider",
        None,
        "capability:system_provider_update",
        "mutate",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_PROVIDER_UPDATE_FORBIDDEN",
        "Operator role or higher is required to update provider settings.",
    )
    .await
    {
        return response;
    }

    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_INVALID",
            "providerId is required.",
            None,
        );
    }
    if payload.name.as_deref().unwrap_or("").trim().is_empty()
        || payload.endpoint.as_deref().unwrap_or("").trim().is_empty()
    {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_INVALID",
            "name and endpoint are required.",
            None,
        );
    }

    let mut state =
        load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let resolved_provider_id = provider_id.trim().to_string();
    let existing = state
        .providers
        .iter()
        .find(|item| item.provider_id == resolved_provider_id)
        .cloned()
        .unwrap_or_else(|| {
            blank_provider_runtime_settings(
                &resolved_provider_id,
                payload.provider_type.as_deref().unwrap_or("Llm"),
            )
        });
    let provider_type = normalize_provider_type(
        payload
            .provider_type
            .as_deref()
            .or(Some(existing.provider_type.as_str())),
    );
    let base_url = payload
        .endpoint
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| existing.base_url.clone());
    let default_model = payload
        .default_model
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| existing.default_model.clone());
    let provider_kind = payload
        .provider_kind
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            if provider_type == "Llm" {
                Some(infer_provider_kind(&base_url, &default_model))
            } else {
                existing.provider_kind.clone()
            }
        });
    let metadata = if payload.metadata.is_empty() {
        existing.metadata.clone()
    } else {
        payload.metadata.clone()
    };
    let updated_at = Some(crate::gateway::server::now_iso());
    let locality_kind = existing.locality_kind.unwrap_or_else(|| {
        infer_provider_locality_kind(
            &base_url,
            existing.device_id.as_deref(),
            existing.environment_id.as_deref(),
        )
    });
    let provider_family_id = existing
        .provider_family_id
        .clone()
        .or_else(|| Some(provider_type.to_ascii_lowercase()));
    let instance_id = existing
        .instance_id
        .clone()
        .unwrap_or_else(|| format!("{}_{}", provider_id, provider_type.to_ascii_lowercase()));
    let profile_id = existing
        .profile_id
        .clone()
        .or_else(|| Some(default_model.clone()).filter(|value| !value.trim().is_empty()));
    let discovery_source = existing
        .discovery_source
        .clone()
        .or_else(|| Some("registry".to_string()));
    let host_id = payload
        .host_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            existing.host_id.clone().or_else(|| {
                Some(
                    crate::services::provider_runtime::config::default_runtime_host_id(
                        &resolved_provider_id,
                        &base_url,
                        locality_kind,
                        existing.environment_id.as_deref(),
                    ),
                )
            })
        });

    let record = ProviderRuntimeSettings {
        provider_id: resolved_provider_id.to_string(),
        name: payload
            .name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(existing.name),
        provider_type: provider_type.clone(),
        provider_kind,
        host_id: host_id.clone(),
        enabled: payload.enabled.unwrap_or(existing.enabled),
        base_url,
        default_model,
        adapter_set_ref: None,
        auth_binding_id: payload
            .auth_binding_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or(existing.auth_binding_id),
        provider_family_id,
        profile_id,
        instance_id: Some(instance_id),
        device_id: existing.device_id.clone(),
        environment_id: existing.environment_id.clone(),
        locality_kind: Some(locality_kind),
        discovery_source,
        batch_policy: existing.batch_policy.clone(),
        updated_at,
        supported_models: Vec::new(),
        metadata,
    };

    upsert_provider_record_in_state(&mut state, record);
    if payload.set_as_default_llm.unwrap_or(false) && provider_type == "Llm" {
        let Some(provider) = state
            .providers
            .iter()
            .find(|provider| provider.provider_id == resolved_provider_id)
        else {
            return crate::gateway::server::cortex_ux_error(
                StatusCode::NOT_FOUND,
                "SYSTEM_PROVIDER_BINDING_TARGET_NOT_FOUND",
                "The provider bound to this adapter slot was not found.",
                Some(json!({ "boundProviderId": resolved_provider_id })),
            );
        };
        if let Err(err) = provider_execution_eligibility(provider, &state) {
            return crate::gateway::server::cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "SYSTEM_PROVIDER_BINDING_NOT_EXECUTABLE",
                "The selected provider cannot be used as an execution binding.",
                Some(json!({ "reason": err.message, "reasonCode": err.code })),
            );
        }
        let record = ExecutionBindingRecord {
            binding_id: DEFAULT_LLM_BINDING_ID.to_string(),
            provider_type: provider_type.clone(),
            bound_provider_id: resolved_provider_id.to_string(),
            updated_at: Some(crate::gateway::server::now_iso()),
            metadata: BTreeMap::new(),
        };
        if let Some(existing) = state
            .execution_bindings
            .iter_mut()
            .find(|binding| binding.binding_id == record.binding_id)
        {
            *existing = record;
        } else {
            state.execution_bindings.push(record);
        }
    }
    if !payload.supported_models.is_empty() {
        upsert_discovery_record(
            &mut state.discovery,
            ProviderDiscoveryRecord {
                provider_id: resolved_provider_id.to_string(),
                provider_type: provider_type.clone(),
                provider_kind: payload
                    .provider_kind
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .or(existing.provider_kind.clone()),
                endpoint: payload
                    .endpoint
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(existing.base_url.clone()),
                default_model: payload
                    .default_model
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .or_else(|| {
                        Some(existing.default_model.clone())
                            .filter(|value| !value.trim().is_empty())
                    }),
                supported_models: payload.supported_models.clone(),
                adapter_health: None,
                adapter_health_error: None,
                openapi_paths: Vec::new(),
                upstream_models_error: None,
                fail_mode: None,
                topology: None,
                updated_at: Some(crate::gateway::server::now_iso()),
                metadata: BTreeMap::new(),
            },
        );
    }

    if let Err(err) = save_provider_registry_state(&state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSTEM_PROVIDER_PERSIST_FAILED",
            "Unable to persist provider settings.",
            Some(json!({ "reason": err })),
        );
    }

    Json(build_system_providers_response().await).into_response()
}

pub(crate) async fn put_system_provider_binding(
    headers: HeaderMap,
    Path(binding_id): Path<String>,
    Json(payload): Json<PutSystemProviderBindingRequest>,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "put_system_provider_binding",
        None,
        "capability:system_provider_update",
        "mutate",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_PROVIDER_BINDING_UPDATE_FORBIDDEN",
        "Operator role or higher is required to update provider bindings.",
    )
    .await
    {
        return response;
    }

    if binding_id.trim().is_empty() || payload.bound_provider_id.trim().is_empty() {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_BINDING_INVALID",
            "bindingId and boundProviderId are required.",
            None,
        );
    }

    let mut state =
        load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let resolved_provider_id = payload.bound_provider_id.trim().to_string();
    let Some(provider) = state
        .providers
        .iter()
        .find(|provider| provider.provider_id == resolved_provider_id)
    else {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::NOT_FOUND,
            "SYSTEM_PROVIDER_BINDING_TARGET_NOT_FOUND",
            "The provider bound to this adapter slot was not found.",
            Some(json!({ "boundProviderId": resolved_provider_id })),
        );
    };
    if let Err(err) = provider_execution_eligibility(provider, &state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_PROVIDER_BINDING_NOT_EXECUTABLE",
            "The selected provider cannot be used as an execution binding.",
            Some(json!({ "reason": err.message, "reasonCode": err.code })),
        );
    }

    let provider_type = payload
        .provider_type
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Llm".to_string());
    let record = ExecutionBindingRecord {
        binding_id: binding_id.trim().to_string(),
        provider_type: provider_type.clone(),
        bound_provider_id: resolved_provider_id,
        updated_at: Some(crate::gateway::server::now_iso()),
        metadata: payload.metadata.clone(),
    };
    if let Some(existing) = state
        .execution_bindings
        .iter_mut()
        .find(|binding| binding.binding_id == record.binding_id)
    {
        *existing = record;
    } else {
        state.execution_bindings.push(record);
    }

    if let Err(err) = save_provider_registry_state(&state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSTEM_PROVIDER_BINDING_PERSIST_FAILED",
            "Unable to persist provider binding.",
            Some(json!({ "reason": err })),
        );
    }

    Json(build_system_providers_response().await).into_response()
}

pub(crate) async fn post_system_auth_binding(
    headers: HeaderMap,
    Json(payload): Json<CreateAuthBindingRequest>,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "post_system_auth_binding",
        None,
        "capability:system_auth_binding_create",
        "mutate",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_AUTH_BINDING_CREATE_FORBIDDEN",
        "Operator role or higher is required to add auth bindings.",
    )
    .await
    {
        return response;
    }

    if payload.api_key.as_deref().unwrap_or("").trim().is_empty() {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SYSTEM_AUTH_BINDING_INVALID",
            "apiKey is required.",
            None,
        );
    }

    let mut state =
        load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let target_kind = match payload.target_kind.as_deref() {
        Some("host") => AuthBindingTargetKind::Host,
        _ => AuthBindingTargetKind::Provider,
    };
    let target_id = if payload.target_id.as_deref().unwrap_or("").trim().is_empty() {
        if target_kind == AuthBindingTargetKind::Host {
            return crate::gateway::server::cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "SYSTEM_AUTH_BINDING_INVALID",
                "targetId is required for host auth bindings.",
                None,
            );
        }
        resolve_provider_runtime_state().provider_id
    } else {
        payload
            .target_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let target_id = target_id.trim().to_string();
    let auth_binding_id = payload
        .auth_binding_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "auth_{}_{}",
                crate::gateway::server::sanitize_fs_component(&target_id),
                Utc::now().timestamp_millis()
            )
        });
    let binding = AuthBindingRecord {
        auth_binding_id: auth_binding_id.clone(),
        target_kind,
        target_id: target_id.clone(),
        auth_type: match payload.auth_type.as_deref() {
            Some("none") => AuthBindingType::None,
            Some("bearer_token") => AuthBindingType::BearerToken,
            Some("pat") => AuthBindingType::Pat,
            Some("ssh_key") => AuthBindingType::SshKey,
            Some("ssh_password") => AuthBindingType::SshPassword,
            _ => AuthBindingType::ApiKey,
        },
        label: payload.label.clone(),
        secret: payload.api_key.unwrap_or_default(),
        has_secret: true,
        created_at: Some(crate::gateway::server::now_iso()),
        updated_at: Some(crate::gateway::server::now_iso()),
        source: payload
            .source
            .clone()
            .or_else(|| Some("manual".to_string())),
        metadata: payload.metadata.clone(),
    };

    if let Some(existing) = state
        .auth_bindings
        .iter_mut()
        .find(|entry| entry.auth_binding_id == auth_binding_id)
    {
        *existing = binding.clone();
    } else {
        state.auth_bindings.push(binding.clone());
    }

    if binding.target_kind == AuthBindingTargetKind::Provider {
        if !link_provider_auth_binding(&mut state, &target_id, &auth_binding_id) {
            let mut default_settings =
                provider_runtime_settings_from_resolved_state(&resolve_provider_runtime_state());
            default_settings.provider_id = target_id.clone();
            default_settings.auth_binding_id = Some(auth_binding_id.clone());
            default_settings.updated_at = Some(crate::gateway::server::now_iso());
            upsert_provider_record_in_state(&mut state, default_settings);
        }
        if let Some(discovery) = state
            .discovery
            .iter_mut()
            .find(|record| record.provider_id == target_id)
        {
            discovery.updated_at = Some(crate::gateway::server::now_iso());
        }
    }

    if let Err(err) = save_provider_registry_state(&state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSTEM_AUTH_BINDING_PERSIST_FAILED",
            "Unable to persist auth binding.",
            Some(json!({ "reason": err })),
        );
    }

    Json(crate::gateway::provider_admin::contracts::map_auth_binding_response(&binding))
        .into_response()
}

pub(crate) async fn put_system_auth_binding(
    headers: HeaderMap,
    Path(auth_binding_id): Path<String>,
    Json(payload): Json<UpdateAuthBindingRequest>,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "put_system_auth_binding",
        None,
        "capability:system_auth_binding_update",
        "mutate",
        "operator",
        vec![],
        true,
        vec![],
        "SYSTEM_AUTH_BINDING_UPDATE_FORBIDDEN",
        "Operator role or higher is required to update auth bindings.",
    )
    .await
    {
        return response;
    }

    let mut state =
        load_provider_registry_state().unwrap_or_else(|_| ProviderRegistryState::default());
    let resolved_payload_provider_id = payload
        .target_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .map(|provider_id| provider_id.trim().to_string());
    let resolved_payload_target_kind = payload
        .target_kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|target_kind| {
            if target_kind.eq_ignore_ascii_case("host") {
                AuthBindingTargetKind::Host
            } else {
                AuthBindingTargetKind::Provider
            }
        });
    let Some(binding) = state
        .auth_bindings
        .iter_mut()
        .find(|entry| entry.auth_binding_id == auth_binding_id.trim())
    else {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::NOT_FOUND,
            "SYSTEM_AUTH_BINDING_NOT_FOUND",
            "Auth binding was not found.",
            Some(json!({ "authBindingId": auth_binding_id })),
        );
    };

    if let Some(label) = payload.label.filter(|value| !value.trim().is_empty()) {
        binding.label = Some(label);
    }
    if let Some(api_key) = payload.api_key.filter(|value| !value.trim().is_empty()) {
        binding.secret = api_key;
        binding.has_secret = true;
    }
    if let Some(source) = payload.source.filter(|value| !value.trim().is_empty()) {
        binding.source = Some(source);
    }
    if let Some(provider_id) = resolved_payload_provider_id {
        binding.target_id = provider_id;
    }
    if let Some(target_kind) = resolved_payload_target_kind {
        binding.target_kind = target_kind;
    }
    binding.updated_at = Some(crate::gateway::server::now_iso());
    let response = crate::gateway::provider_admin::contracts::map_auth_binding_response(binding);

    if let Err(err) = save_provider_registry_state(&state) {
        return crate::gateway::server::cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSTEM_AUTH_BINDING_PERSIST_FAILED",
            "Unable to persist auth binding.",
            Some(json!({ "reason": err })),
        );
    }

    Json(response).into_response()
}

pub(crate) async fn get_system_provider_runtime_status(
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err(response) = crate::gateway::server::enforce_role_authorization(
        &headers,
        "get_system_provider_runtime_status",
        None,
        "capability:system_provider_runtime_status",
        "read",
        "operator",
        vec![],
        false,
        vec![],
        "SYSTEM_PROVIDER_RUNTIME_STATUS_FORBIDDEN",
        "Operator role or higher is required to inspect runtime status.",
    )
    .await
    {
        return response;
    }

    Json(build_provider_runtime_status_response().await).into_response()
}
