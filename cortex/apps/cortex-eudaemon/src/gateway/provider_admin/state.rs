use crate::gateway::provider_admin::records::upsert_provider_record_in_state;
use crate::services::cortex_ux::now_iso;
use crate::services::provider_runtime::config::{
    ProviderRegistryState, provider_runtime_settings_from_resolved_state,
    resolve_provider_runtime_state,
};
use std::collections::BTreeMap;

pub(crate) fn resolve_provider_probe_api_key(
    explicit_api_key: Option<&str>,
    use_stored_auth: bool,
    provider_id: Option<&str>,
    auth_binding_id: Option<&str>,
    provider_bindings: &BTreeMap<String, String>,
    auth_secrets: &BTreeMap<String, String>,
) -> Option<String> {
    let explicit_api_key = explicit_api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if explicit_api_key.is_some() {
        return explicit_api_key;
    }
    if !use_stored_auth {
        return None;
    }

    let resolved_binding_id = auth_binding_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            provider_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .and_then(|provider_id| provider_bindings.get(provider_id).cloned())
        });

    resolved_binding_id
        .as_ref()
        .and_then(|binding_id| auth_secrets.get(binding_id).cloned())
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn link_provider_auth_binding(
    state: &mut ProviderRegistryState,
    provider_id: &str,
    auth_binding_id: &str,
) -> bool {
    let resolved_provider_id = provider_id.trim().to_string();
    if let Some(existing) = state
        .providers
        .iter_mut()
        .find(|item| item.provider_id == resolved_provider_id)
    {
        existing.auth_binding_id = Some(auth_binding_id.to_string());
        existing.updated_at = Some(now_iso());
        return true;
    }

    let resolved_runtime = resolve_provider_runtime_state();
    if resolved_provider_id == resolved_runtime.provider_id {
        let mut default_settings = provider_runtime_settings_from_resolved_state(&resolved_runtime);
        default_settings.provider_id = resolved_provider_id;
        default_settings.auth_binding_id = Some(auth_binding_id.to_string());
        default_settings.updated_at = Some(now_iso());
        upsert_provider_record_in_state(state, default_settings);
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_provider_probe_api_key_prefers_explicit_key() {
        let provider_bindings =
            BTreeMap::from([("provider_alpha".to_string(), "cred_alpha".to_string())]);
        let credential_keys = BTreeMap::from([("cred_alpha".to_string(), "saved-key".to_string())]);

        let resolved = resolve_provider_probe_api_key(
            Some("typed-key"),
            true,
            Some("provider_alpha"),
            None,
            &provider_bindings,
            &credential_keys,
        );

        assert_eq!(resolved.as_deref(), Some("typed-key"));
    }

    #[test]
    fn resolve_provider_probe_api_key_uses_provider_binding_when_requested() {
        let provider_bindings =
            BTreeMap::from([("provider_alpha".to_string(), "cred_alpha".to_string())]);
        let credential_keys = BTreeMap::from([("cred_alpha".to_string(), "saved-key".to_string())]);

        let resolved = resolve_provider_probe_api_key(
            None,
            true,
            Some("provider_alpha"),
            None,
            &provider_bindings,
            &credential_keys,
        );

        assert_eq!(resolved.as_deref(), Some("saved-key"));
    }
}
