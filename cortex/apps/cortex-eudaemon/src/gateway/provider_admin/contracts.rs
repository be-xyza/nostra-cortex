use crate::services::provider_runtime::config::{
    AuthBindingRecord, ExecutionBindingRecord, ProviderBatchPolicy, ProviderDiscoveryRecord,
    ProviderRuntimeContext, RuntimeHostRecord,
};
use crate::services::secret_redaction::{
    redact_json_value, redact_metadata_map, redact_runtime_text,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemProviderRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_family: Option<String>,
    pub(crate) host_id: String,
    pub(crate) endpoint: String,
    pub(crate) is_active: bool,
    pub(crate) priority: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) default_model: Option<String>,
    #[serde(default)]
    pub(crate) supported_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health_error: Option<String>,
    #[serde(default)]
    pub(crate) openapi_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) upstream_models_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fail_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_mode: Option<String>,
    pub(crate) auth_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_type: Option<String>,
    #[serde(default)]
    pub(crate) binding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) topology: Option<ProviderRuntimeContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) batch_policy: Option<ProviderBatchPolicy>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemAuthBindingResponse {
    pub(crate) auth_binding_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) auth_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<String>,
    pub(crate) has_secret: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    pub(crate) updated_at: String,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemProviderBindingResponse {
    pub(crate) binding_id: String,
    pub(crate) provider_type: String,
    pub(crate) bound_provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) updated_at: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemProviderDiscoveryResponse {
    pub(crate) provider_id: String,
    pub(crate) provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_kind: Option<String>,
    pub(crate) endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) default_model: Option<String>,
    #[serde(default)]
    pub(crate) supported_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health_error: Option<String>,
    #[serde(default)]
    pub(crate) openapi_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) upstream_models_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fail_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) topology: Option<ProviderRuntimeContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) updated_at: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemRuntimeHostResponse {
    pub(crate) host_id: String,
    pub(crate) name: String,
    pub(crate) host_kind: String,
    pub(crate) endpoint: String,
    pub(crate) locality_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) environment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) health: Option<Value>,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    pub(crate) remote_discovery_enabled: bool,
    pub(crate) execution_routable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) updated_at: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemProvidersResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) providers: Vec<SystemProviderRecord>,
    #[serde(default)]
    pub(crate) runtime_hosts: Vec<SystemRuntimeHostResponse>,
    #[serde(default)]
    pub(crate) auth_bindings: Vec<SystemAuthBindingResponse>,
    #[serde(default)]
    pub(crate) execution_bindings: Vec<SystemProviderBindingResponse>,
    #[serde(default)]
    pub(crate) discovery_records: Vec<SystemProviderDiscoveryResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OperatorProviderInventoryResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) providers: Vec<SystemProviderRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeHostInventoryResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) runtime_hosts: Vec<SystemRuntimeHostResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthBindingInventoryResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) auth_bindings: Vec<SystemAuthBindingResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExecutionBindingStatusResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) execution_bindings: Vec<SystemProviderBindingResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderDiscoveryInventoryResponse {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) discovery_records: Vec<SystemProviderDiscoveryResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderRuntimeStatusResponse {
    pub(crate) enabled: bool,
    pub(crate) base_url: String,
    pub(crate) binding_id: String,
    pub(crate) fail_mode: String,
    pub(crate) model: String,
    pub(crate) provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adapter_health_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) openapi_paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) openapi_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) upstream_models: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) upstream_models_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PutSystemProviderBindingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_type: Option<String>,
    pub(crate) bound_provider_id: String,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PutSystemProviderRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) set_as_default_llm: Option<bool>,
    #[serde(default)]
    pub(crate) supported_models: Vec<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAuthBindingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateAuthBindingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target_kind: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
}

pub(crate) fn map_auth_binding_response(binding: &AuthBindingRecord) -> SystemAuthBindingResponse {
    SystemAuthBindingResponse {
        auth_binding_id: binding.auth_binding_id.clone(),
        target_kind: match binding.target_kind {
            crate::services::provider_runtime::config::AuthBindingTargetKind::Provider => {
                "provider".to_string()
            }
            crate::services::provider_runtime::config::AuthBindingTargetKind::Host => {
                "host".to_string()
            }
        },
        target_id: binding.target_id.clone(),
        auth_type: match binding.auth_type {
            crate::services::provider_runtime::config::AuthBindingType::None => "none".to_string(),
            crate::services::provider_runtime::config::AuthBindingType::ApiKey => {
                "api_key".to_string()
            }
            crate::services::provider_runtime::config::AuthBindingType::BearerToken => {
                "bearer_token".to_string()
            }
            crate::services::provider_runtime::config::AuthBindingType::Pat => "pat".to_string(),
            crate::services::provider_runtime::config::AuthBindingType::SshKey => {
                "ssh_key".to_string()
            }
            crate::services::provider_runtime::config::AuthBindingType::SshPassword => {
                "ssh_password".to_string()
            }
        },
        label: binding.label.clone(),
        has_secret: binding.has_secret || !binding.secret.trim().is_empty(),
        source: binding.source.clone(),
        updated_at: binding
            .updated_at
            .clone()
            .unwrap_or_else(crate::gateway::server::now_iso),
        metadata: redact_metadata_map(&binding.metadata),
    }
}

pub(crate) fn map_execution_binding_response(
    binding: &ExecutionBindingRecord,
) -> SystemProviderBindingResponse {
    SystemProviderBindingResponse {
        binding_id: binding.binding_id.clone(),
        provider_type: binding.provider_type.clone(),
        bound_provider_id: binding.bound_provider_id.clone(),
        updated_at: binding.updated_at.clone(),
        metadata: redact_metadata_map(&binding.metadata),
    }
}

pub(crate) fn map_runtime_host_response(host: &RuntimeHostRecord) -> SystemRuntimeHostResponse {
    SystemRuntimeHostResponse {
        host_id: host.host_id.clone(),
        name: host.name.clone(),
        host_kind: format!("{:?}", host.host_kind).to_ascii_lowercase(),
        endpoint: host.endpoint.clone(),
        locality_kind: format!("{:?}", host.locality_kind),
        device_id: host.device_id.clone(),
        environment_id: host.environment_id.clone(),
        health: host.health.as_ref().map(redact_json_value),
        capabilities: host.capabilities.clone(),
        remote_discovery_enabled: host.remote_discovery_enabled,
        execution_routable: host.execution_routable,
        updated_at: host.updated_at.clone(),
        metadata: redact_metadata_map(&host.metadata),
    }
}

pub(crate) fn map_discovery_response(
    record: &ProviderDiscoveryRecord,
) -> SystemProviderDiscoveryResponse {
    SystemProviderDiscoveryResponse {
        provider_id: record.provider_id.clone(),
        provider_type: record.provider_type.clone(),
        provider_kind: record.provider_kind.clone(),
        endpoint: record.endpoint.clone(),
        default_model: record.default_model.clone(),
        supported_models: record.supported_models.clone(),
        adapter_health: record.adapter_health.as_ref().map(redact_json_value),
        adapter_health_error: record
            .adapter_health_error
            .as_ref()
            .map(|value| redact_runtime_text(value)),
        openapi_paths: record.openapi_paths.clone(),
        upstream_models_error: record
            .upstream_models_error
            .as_ref()
            .map(|value| redact_runtime_text(value)),
        fail_mode: record.fail_mode.clone(),
        topology: record.topology.clone(),
        updated_at: record.updated_at.clone(),
        metadata: redact_metadata_map(&record.metadata),
    }
}
