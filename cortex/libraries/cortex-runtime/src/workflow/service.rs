use crate::RuntimeError;
use crate::ports::TimeProvider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRuntimeConfig {
    pub default_actor_principal: String,
    pub default_actor_role: String,
    pub active_space_id: String,
    #[serde(default)]
    pub decision_signing_secret: Option<String>,
}

impl Default for WorkflowRuntimeConfig {
    fn default() -> Self {
        Self {
            default_actor_principal: "2vxsx-fae".to_string(),
            default_actor_role: "operator".to_string(),
            active_space_id: "space-default".to_string(),
            decision_signing_secret: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DecisionSignatureHeaders {
    pub actor_principal: String,
    pub actor_role: String,
    pub signed_at: Option<u64>,
    pub signature: Option<String>,
}

pub fn normalize_actor_role(role: &str) -> String {
    let normalized = role.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        "operator".to_string()
    } else {
        normalized
    }
}

pub fn resolve_space_id(
    config: &WorkflowRuntimeConfig,
    requested_space_id: Option<&str>,
) -> String {
    requested_space_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(config.active_space_id.as_str())
        .to_string()
}

pub fn signature_material(
    principal: &str,
    role: &str,
    decision_gate_id: &str,
    mutation_id: &str,
    action_target: &str,
    signed_at: u64,
) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        principal, role, decision_gate_id, mutation_id, action_target, signed_at
    )
}

pub fn decision_signature(secret: &str, material: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"|");
    hasher.update(material.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn build_decision_headers(
    config: &WorkflowRuntimeConfig,
    time: &dyn TimeProvider,
    decision_gate_id: &str,
    mutation_id: &str,
    action_target: &str,
) -> Result<DecisionSignatureHeaders, RuntimeError> {
    let actor_principal = config.default_actor_principal.trim().to_string();
    let actor_role = normalize_actor_role(&config.default_actor_role);

    if actor_principal.is_empty() {
        return Err(RuntimeError::Domain(
            "workflow actor principal must not be empty".to_string(),
        ));
    }

    let signing_secret = config
        .decision_signing_secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let (signed_at, signature) = if let Some(secret) = signing_secret {
        let ts = time.now_unix_secs();
        let material = signature_material(
            &actor_principal,
            &actor_role,
            decision_gate_id,
            mutation_id,
            action_target,
            ts,
        );
        (Some(ts), Some(decision_signature(secret, &material)))
    } else {
        (None, None)
    };

    Ok(DecisionSignatureHeaders {
        actor_principal,
        actor_role,
        signed_at,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedTimeProvider;

    impl TimeProvider for FixedTimeProvider {
        fn now_unix_secs(&self) -> u64 {
            1_700_000_000
        }

        fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
            Ok(format!("ts-{unix_secs}"))
        }
    }

    #[test]
    fn resolve_space_prefers_request_then_config_default() {
        let config = WorkflowRuntimeConfig::default();
        assert_eq!(resolve_space_id(&config, Some("space-a")), "space-a");
        assert_eq!(resolve_space_id(&config, Some("   ")), "space-default");
        assert_eq!(resolve_space_id(&config, None), "space-default");
    }

    #[test]
    fn decision_headers_are_signed_when_secret_is_present() {
        let config = WorkflowRuntimeConfig {
            decision_signing_secret: Some("secret".to_string()),
            ..WorkflowRuntimeConfig::default()
        };
        let headers = build_decision_headers(
            &config,
            &FixedTimeProvider,
            "gate-1",
            "mutation-1",
            "governance:decision",
        )
        .expect("headers");
        assert_eq!(headers.actor_principal, "2vxsx-fae");
        assert_eq!(headers.actor_role, "operator");
        assert_eq!(headers.signed_at, Some(1_700_000_000));
        assert!(headers.signature.is_some());
    }
}
