use crate::services::provider_runtime::config::{
    AuthBindingRecord, AuthBindingType, ProviderLocalityKind, ProviderRegistryState,
    ProviderRuntimeSettings, RuntimeHostRecord, adapter_api_key_from_env, infer_provider_kind,
    infer_provider_locality_kind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderExecutionEligibilityError {
    pub code: &'static str,
    pub message: String,
}

fn provider_type_matches(candidate: &str, expected: &str) -> bool {
    candidate.trim().eq_ignore_ascii_case(expected.trim())
}

pub fn runtime_endpoint_supports_execution(endpoint: &str) -> bool {
    let normalized = endpoint.trim().to_ascii_lowercase();
    normalized.starts_with("http://") || normalized.starts_with("https://")
}

pub fn default_remote_discovery_enabled(endpoint: &str) -> bool {
    endpoint.trim().to_ascii_lowercase().starts_with("ssh://")
}

pub fn runtime_host_allows_remote_discovery(host: &RuntimeHostRecord) -> bool {
    host.remote_discovery_enabled
        && host
            .endpoint
            .trim()
            .to_ascii_lowercase()
            .starts_with("ssh://")
}

pub fn runtime_host_allows_execution(host: &RuntimeHostRecord) -> bool {
    host.execution_routable && runtime_endpoint_supports_execution(&host.endpoint)
}

fn provider_auth_is_satisfiable(
    provider: &ProviderRuntimeSettings,
    auth_binding: Option<&AuthBindingRecord>,
) -> bool {
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
    let auth_type = auth_binding
        .map(|binding| binding.auth_type.clone())
        .unwrap_or_else(|| {
            if locality_kind == ProviderLocalityKind::Local
                && provider_kind.eq_ignore_ascii_case("ollama")
            {
                AuthBindingType::None
            } else if provider_kind.eq_ignore_ascii_case("ollama") {
                AuthBindingType::None
            } else {
                AuthBindingType::ApiKey
            }
        });
    if auth_type == AuthBindingType::None {
        return true;
    }

    auth_binding
        .map(|binding| binding.has_secret || !binding.secret.trim().is_empty())
        .unwrap_or(false)
        || adapter_api_key_from_env().is_some()
}

pub fn provider_execution_eligibility(
    provider: &ProviderRuntimeSettings,
    state: &ProviderRegistryState,
) -> Result<(), ProviderExecutionEligibilityError> {
    if !provider.enabled {
        return Err(ProviderExecutionEligibilityError {
            code: "provider_disabled",
            message: format!("Provider {} is disabled.", provider.provider_id),
        });
    }

    if !runtime_endpoint_supports_execution(&provider.base_url) {
        return Err(ProviderExecutionEligibilityError {
            code: "provider_transport_unsupported",
            message: format!(
                "Provider {} uses an execution transport the runtime cannot call directly.",
                provider.provider_id
            ),
        });
    }

    if let Some(host_id) = provider
        .host_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        let Some(host) = state
            .runtime_hosts
            .iter()
            .find(|host| host.host_id == *host_id)
        else {
            return Err(ProviderExecutionEligibilityError {
                code: "provider_host_missing",
                message: format!(
                    "Provider {} references an unknown runtime host.",
                    provider.provider_id
                ),
            });
        };
        if !runtime_host_allows_execution(host) {
            return Err(ProviderExecutionEligibilityError {
                code: "provider_host_not_routable",
                message: format!(
                    "Provider {} is attached to a runtime host that is not execution-routable.",
                    provider.provider_id
                ),
            });
        }
    }

    let auth_binding = provider.auth_binding_id.as_ref().and_then(|binding_id| {
        state
            .auth_bindings
            .iter()
            .find(|binding| binding.auth_binding_id == *binding_id)
    });
    if !provider_auth_is_satisfiable(provider, auth_binding) {
        return Err(ProviderExecutionEligibilityError {
            code: "provider_auth_unsatisfied",
            message: format!(
                "Provider {} does not have satisfiable authentication for execution.",
                provider.provider_id
            ),
        });
    }

    Ok(())
}

pub fn provider_by_id_if_executable<'a>(
    state: &'a ProviderRegistryState,
    provider_id: &str,
) -> Option<&'a ProviderRuntimeSettings> {
    state
        .providers
        .iter()
        .find(|provider| provider.provider_id == provider_id)
        .filter(|provider| provider_execution_eligibility(provider, state).is_ok())
}

pub fn preferred_executable_provider_for_type<'a>(
    state: &'a ProviderRegistryState,
    provider_type: &str,
) -> Option<&'a ProviderRuntimeSettings> {
    state
        .execution_bindings
        .iter()
        .find(|binding| provider_type_matches(&binding.provider_type, provider_type))
        .and_then(|binding| provider_by_id_if_executable(state, &binding.bound_provider_id))
        .or_else(|| {
            state.providers.iter().find(|provider| {
                provider_type_matches(&provider.provider_type, provider_type)
                    && provider_execution_eligibility(provider, state).is_ok()
            })
        })
}
