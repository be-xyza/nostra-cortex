use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceKind {
    ProviderKey,
    Credential,
    PiiField,
    SealedDocument,
    IdentityClaim,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceSensitivityClass {
    Secret,
    RestrictedPii,
    SealedContent,
    ProtectedMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceStorageMode {
    ExternalVault,
    OperatorEnv,
    SealedStore,
    DerivedReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceLocationClass {
    NostraAuthority,
    CortexHost,
    ExternalSecretManager,
    UserDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceStewardRole {
    User,
    SpaceSteward,
    SystemsSteward,
    Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceRenderMode {
    StatusOnly,
    Fingerprint,
    RedactedPreview,
    SealedOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceGrantStatus {
    Requested,
    Approved,
    Denied,
    Expired,
    Revoked,
    Used,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SealedToolKind {
    SealedProviderInvoke,
    SealedDocumentRender,
    RedactedConfigInspect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SealedToolInputMode {
    AgentRequest,
    OperatorRequest,
    WorkflowAdapter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceTrustedBoundary {
    CortexProviderTransport,
    CortexSealedRenderer,
    CortexRedactedInspector,
    ExternalSecretManager,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceUseResultStatus {
    Success,
    Denied,
    Expired,
    Revoked,
    BoundaryError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedResourceEmittedField {
    Status,
    Fingerprint,
    RedactedPreview,
    SealedArtifactRef,
    AuditRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceAuthority {
    pub owner: String,
    pub steward_role: ProtectedResourceStewardRole,
    pub governance_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceSensitivity {
    pub class: ProtectedResourceSensitivityClass,
    pub contains_pii: bool,
    pub rotation_required_on_exposure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceStorage {
    pub mode: ProtectedResourceStorageMode,
    pub location_class: ProtectedResourceLocationClass,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourcePolicy {
    pub allowed_purposes: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub default_render_mode: ProtectedResourceRenderMode,
    pub audit_required: bool,
    pub raw_value_export: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceLineage {
    pub source_ref: String,
    pub created_by: String,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResource {
    pub schema_version: String,
    pub resource_id: String,
    pub kind: ProtectedResourceKind,
    pub space_id: String,
    pub authority: ProtectedResourceAuthority,
    pub sensitivity: ProtectedResourceSensitivity,
    pub storage: ProtectedResourceStorage,
    pub policy: ProtectedResourcePolicy,
    pub lineage: ProtectedResourceLineage,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretRef {
    pub schema_version: String,
    pub secret_ref: String,
    pub resource_id: String,
    pub kind: ProtectedResourceKind,
    pub space_id: String,
    pub fingerprint: String,
    pub render_mode: ProtectedResourceRenderMode,
    pub expires_at: String,
    pub issued_at: String,
    pub issued_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceGrant {
    pub schema_version: String,
    pub grant_id: String,
    pub secret_ref: String,
    pub resource_id: String,
    pub space_id: String,
    pub purpose: String,
    pub tool: String,
    pub render_mode: ProtectedResourceRenderMode,
    pub status: ProtectedResourceGrantStatus,
    pub requested_by: String,
    pub approved_by: String,
    pub issued_at: String,
    pub expires_at: String,
    pub audit_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SealedToolInvocation {
    pub schema_version: String,
    pub invocation_id: String,
    pub tool: SealedToolKind,
    pub space_id: String,
    pub purpose: String,
    pub grant_id: String,
    pub secret_refs: Vec<String>,
    pub input_mode: SealedToolInputMode,
    pub requested_render_mode: ProtectedResourceRenderMode,
    pub requested_by: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedResourceUsed {
    pub schema_version: String,
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub space_id: String,
    pub resource_id: String,
    pub secret_ref: String,
    pub grant_id: String,
    pub purpose: String,
    pub tool: String,
    pub trusted_boundary: ProtectedResourceTrustedBoundary,
    pub render_mode: ProtectedResourceRenderMode,
    pub result_status: ProtectedResourceUseResultStatus,
    pub emitted_fields: Vec<ProtectedResourceEmittedField>,
    pub fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redacted_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sealed_artifact_ref: Option<String>,
}

pub fn validate_protected_resource(resource: &ProtectedResource) -> Result<(), String> {
    expect_schema_version(&resource.schema_version)?;
    expect_prefix(&resource.resource_id, "pr_", "resourceId")?;
    expect_fingerprint(&resource.storage.fingerprint)?;
    expect_non_empty("spaceId", &resource.space_id)?;
    expect_non_empty("authority.owner", &resource.authority.owner)?;
    expect_non_empty(
        "authority.governanceRef",
        &resource.authority.governance_ref,
    )?;
    expect_non_empty("lineage.sourceRef", &resource.lineage.source_ref)?;
    expect_non_empty("lineage.createdBy", &resource.lineage.created_by)?;
    expect_non_empty("lineage.decisionRef", &resource.lineage.decision_ref)?;
    if resource.policy.allowed_purposes.is_empty() {
        return Err("policy.allowedPurposes must not be empty".to_string());
    }
    if resource.policy.allowed_tools.is_empty() {
        return Err("policy.allowedTools must not be empty".to_string());
    }
    if !resource.policy.audit_required {
        return Err("policy.auditRequired must be true".to_string());
    }
    if resource.policy.raw_value_export != "forbidden" {
        return Err("policy.rawValueExport must be forbidden".to_string());
    }
    Ok(())
}

pub fn validate_secret_ref(secret_ref: &SecretRef) -> Result<(), String> {
    expect_schema_version(&secret_ref.schema_version)?;
    expect_prefix(&secret_ref.secret_ref, "secretref_", "secretRef")?;
    expect_prefix(&secret_ref.resource_id, "pr_", "resourceId")?;
    expect_fingerprint(&secret_ref.fingerprint)?;
    expect_non_empty("spaceId", &secret_ref.space_id)?;
    expect_non_empty("issuedBy", &secret_ref.issued_by)?;
    Ok(())
}

pub fn validate_grant(grant: &ProtectedResourceGrant) -> Result<(), String> {
    expect_schema_version(&grant.schema_version)?;
    expect_prefix(&grant.grant_id, "grant_", "grantId")?;
    expect_prefix(&grant.secret_ref, "secretref_", "secretRef")?;
    expect_prefix(&grant.resource_id, "pr_", "resourceId")?;
    expect_non_empty("spaceId", &grant.space_id)?;
    expect_non_empty("purpose", &grant.purpose)?;
    expect_non_empty("tool", &grant.tool)?;
    expect_non_empty("requestedBy", &grant.requested_by)?;
    expect_non_empty("approvedBy", &grant.approved_by)?;
    if !grant.audit_required {
        return Err("auditRequired must be true".to_string());
    }
    Ok(())
}

pub fn validate_sealed_invocation(invocation: &SealedToolInvocation) -> Result<(), String> {
    expect_schema_version(&invocation.schema_version)?;
    expect_prefix(&invocation.invocation_id, "sealedinv_", "invocationId")?;
    expect_prefix(&invocation.grant_id, "grant_", "grantId")?;
    expect_non_empty("spaceId", &invocation.space_id)?;
    expect_non_empty("purpose", &invocation.purpose)?;
    expect_non_empty("requestedBy", &invocation.requested_by)?;
    if invocation.secret_refs.is_empty() {
        return Err("secretRefs must not be empty".to_string());
    }
    for secret_ref in &invocation.secret_refs {
        expect_prefix(secret_ref, "secretref_", "secretRefs[]")?;
    }
    Ok(())
}

pub fn validate_used_event(event: &ProtectedResourceUsed) -> Result<(), String> {
    expect_schema_version(&event.schema_version)?;
    expect_prefix(&event.event_id, "prevt_", "eventId")?;
    expect_prefix(&event.resource_id, "pr_", "resourceId")?;
    expect_prefix(&event.secret_ref, "secretref_", "secretRef")?;
    expect_prefix(&event.grant_id, "grant_", "grantId")?;
    expect_fingerprint(&event.fingerprint)?;
    expect_non_empty("spaceId", &event.space_id)?;
    expect_non_empty("purpose", &event.purpose)?;
    expect_non_empty("tool", &event.tool)?;
    if event.event_type != "ProtectedResourceUsedV1" {
        return Err("eventType must be ProtectedResourceUsedV1".to_string());
    }
    if event.emitted_fields.is_empty() {
        return Err("emittedFields must not be empty".to_string());
    }
    Ok(())
}

fn expect_schema_version(value: &str) -> Result<(), String> {
    if value == "1.0.0" {
        Ok(())
    } else {
        Err("schemaVersion must be 1.0.0".to_string())
    }
}

fn expect_prefix(value: &str, prefix: &str, field: &str) -> Result<(), String> {
    if value.starts_with(prefix) {
        Ok(())
    } else {
        Err(format!("{field} must start with {prefix}"))
    }
}

fn expect_non_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

fn expect_fingerprint(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("fingerprint must start with sha256:".to_string());
    };
    if hex.len() < 12 || hex.len() > 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("fingerprint must be sha256: plus 12-64 hex characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource() -> ProtectedResource {
        ProtectedResource {
            schema_version: "1.0.0".to_string(),
            resource_id: "pr_openrouter_provider_key".to_string(),
            kind: ProtectedResourceKind::ProviderKey,
            space_id: "space:nostra-governance-v0".to_string(),
            authority: ProtectedResourceAuthority {
                owner: "Systems Steward".to_string(),
                steward_role: ProtectedResourceStewardRole::Operator,
                governance_ref:
                    "research/138-protected-resources-and-secret-egress/DECISIONS.md#dec-138-009"
                        .to_string(),
            },
            sensitivity: ProtectedResourceSensitivity {
                class: ProtectedResourceSensitivityClass::Secret,
                contains_pii: false,
                rotation_required_on_exposure: true,
            },
            storage: ProtectedResourceStorage {
                mode: ProtectedResourceStorageMode::OperatorEnv,
                location_class: ProtectedResourceLocationClass::CortexHost,
                fingerprint: "sha256:4ace5fcc4269".to_string(),
            },
            policy: ProtectedResourcePolicy {
                allowed_purposes: vec!["sealed-provider-invocation".to_string()],
                allowed_tools: vec!["sealed_provider_invoke".to_string()],
                default_render_mode: ProtectedResourceRenderMode::StatusOnly,
                audit_required: true,
                raw_value_export: "forbidden".to_string(),
            },
            lineage: ProtectedResourceLineage {
                source_ref: "operator-env:NOSTRA_LLM_API_KEY".to_string(),
                created_by: "Systems Steward".to_string(),
                decision_ref: "DEC-138-009".to_string(),
            },
            created_at: "2026-05-02T00:00:00Z".to_string(),
            updated_at: "2026-05-02T00:00:00Z".to_string(),
        }
    }

    fn secret_ref() -> SecretRef {
        SecretRef {
            schema_version: "1.0.0".to_string(),
            secret_ref: "secretref_openrouter_provider_key_001".to_string(),
            resource_id: "pr_openrouter_provider_key".to_string(),
            kind: ProtectedResourceKind::ProviderKey,
            space_id: "space:nostra-governance-v0".to_string(),
            fingerprint: "sha256:4ace5fcc4269".to_string(),
            render_mode: ProtectedResourceRenderMode::StatusOnly,
            expires_at: "2026-05-02T01:00:00Z".to_string(),
            issued_at: "2026-05-02T00:00:00Z".to_string(),
            issued_by: "Systems Steward".to_string(),
        }
    }

    fn grant() -> ProtectedResourceGrant {
        ProtectedResourceGrant {
            schema_version: "1.0.0".to_string(),
            grant_id: "grant_openrouter_provider_key_001".to_string(),
            secret_ref: "secretref_openrouter_provider_key_001".to_string(),
            resource_id: "pr_openrouter_provider_key".to_string(),
            space_id: "space:nostra-governance-v0".to_string(),
            purpose: "sealed-provider-invocation".to_string(),
            tool: "sealed_provider_invoke".to_string(),
            render_mode: ProtectedResourceRenderMode::StatusOnly,
            status: ProtectedResourceGrantStatus::Approved,
            requested_by: "agent:eudaemon-alpha-01".to_string(),
            approved_by: "Systems Steward".to_string(),
            issued_at: "2026-05-02T00:00:00Z".to_string(),
            expires_at: "2026-05-02T01:00:00Z".to_string(),
            audit_required: true,
        }
    }

    #[test]
    fn protected_resource_contracts_validate() {
        validate_protected_resource(&resource()).unwrap();
        validate_secret_ref(&secret_ref()).unwrap();
        validate_grant(&grant()).unwrap();

        let invocation = SealedToolInvocation {
            schema_version: "1.0.0".to_string(),
            invocation_id: "sealedinv_openrouter_provider_key_001".to_string(),
            tool: SealedToolKind::SealedProviderInvoke,
            space_id: "space:nostra-governance-v0".to_string(),
            purpose: "sealed-provider-invocation".to_string(),
            grant_id: "grant_openrouter_provider_key_001".to_string(),
            secret_refs: vec!["secretref_openrouter_provider_key_001".to_string()],
            input_mode: SealedToolInputMode::WorkflowAdapter,
            requested_render_mode: ProtectedResourceRenderMode::StatusOnly,
            requested_by: "agent:eudaemon-alpha-01".to_string(),
            requested_at: "2026-05-02T00:00:00Z".to_string(),
        };
        validate_sealed_invocation(&invocation).unwrap();

        let event = ProtectedResourceUsed {
            schema_version: "1.0.0".to_string(),
            event_id: "prevt_openrouter_provider_key_001".to_string(),
            event_type: "ProtectedResourceUsedV1".to_string(),
            occurred_at: "2026-05-02T00:00:01Z".to_string(),
            space_id: "space:nostra-governance-v0".to_string(),
            resource_id: "pr_openrouter_provider_key".to_string(),
            secret_ref: "secretref_openrouter_provider_key_001".to_string(),
            grant_id: "grant_openrouter_provider_key_001".to_string(),
            purpose: "sealed-provider-invocation".to_string(),
            tool: "sealed_provider_invoke".to_string(),
            trusted_boundary: ProtectedResourceTrustedBoundary::CortexProviderTransport,
            render_mode: ProtectedResourceRenderMode::StatusOnly,
            result_status: ProtectedResourceUseResultStatus::Success,
            emitted_fields: vec![
                ProtectedResourceEmittedField::Status,
                ProtectedResourceEmittedField::Fingerprint,
                ProtectedResourceEmittedField::AuditRef,
            ],
            fingerprint: "sha256:4ace5fcc4269".to_string(),
            redacted_preview: None,
            sealed_artifact_ref: None,
        };
        validate_used_event(&event).unwrap();
    }

    #[test]
    fn protected_resource_serialization_does_not_require_raw_values() {
        let serialized = serde_json::to_string(&resource()).unwrap();
        assert!(serialized.contains("\"rawValueExport\":\"forbidden\""));
        assert!(!serialized.contains("NOSTRA_LLM_API_KEY="));
        assert!(!serialized.contains("sk-or-v1-"));
    }

    #[test]
    fn protected_resource_policy_rejects_raw_value_export() {
        let mut resource = resource();
        resource.policy.raw_value_export = "allowed".to_string();
        let error = validate_protected_resource(&resource).unwrap_err();
        assert!(error.contains("rawValueExport"));
    }

    #[test]
    fn protected_resource_grant_requires_audit() {
        let mut grant = grant();
        grant.audit_required = false;
        let error = validate_grant(&grant).unwrap_err();
        assert!(error.contains("auditRequired"));
    }
}
