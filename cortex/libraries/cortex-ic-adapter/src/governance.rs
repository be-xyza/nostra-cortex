use crate::ic::resolve_canister_id;
use async_trait::async_trait;
use candid::{CandidType, Decode, Encode, Principal};
use cortex_runtime::RuntimeError;
use cortex_runtime::ports::{
    ActorRoleBinding, GovernanceAdapter, GovernanceScopeEvaluation, GovernanceScopeRequest,
};
use ic_agent::identity::AnonymousIdentity;
use ic_agent::Agent;
use serde::Serialize;

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct ActionScopeEvaluationCandid {
    allowed: bool,
    reason: String,
    effective_weight: f64,
    requires_review: bool,
    gate_decision: String,
    required_actions: Vec<String>,
    policy_ref: Option<String>,
    policy_version: u64,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct ActorRoleBindingCandid {
    space_id: String,
    principal: Principal,
    role: String,
    source_ref: Option<String>,
    updated_at: u64,
}

#[derive(Clone, Debug)]
pub struct GovernanceCanisterClient {
    host: String,
    canister_id: Principal,
}

impl GovernanceCanisterClient {
    pub async fn from_env() -> Result<Self, RuntimeError> {
        let host = std::env::var("NOSTRA_IC_HOST")
            .or_else(|_| std::env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
        let canister_id_text = resolve_canister_id("CANISTER_ID_GOVERNANCE", "governance")
            .await
            .map_err(RuntimeError::Network)?;
        let canister_id = Principal::from_text(canister_id_text)
            .map_err(|err| RuntimeError::Network(format!("invalid governance principal: {err}")))?;
        Ok(Self { host, canister_id })
    }

    async fn agent(&self) -> Result<Agent, RuntimeError> {
        let agent = Agent::builder()
            .with_url(self.host.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|err| RuntimeError::Network(format!("failed to build ic-agent: {err}")))?;

        if self.host.contains("127.0.0.1") || self.host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|err| RuntimeError::Network(format!("failed to fetch root key: {err}")))?;
        }

        Ok(agent)
    }
}

#[async_trait]
impl GovernanceAdapter for GovernanceCanisterClient {
    async fn evaluate_action_scope(
        &self,
        request: GovernanceScopeRequest,
    ) -> Result<GovernanceScopeEvaluation, RuntimeError> {
        let agent = self.agent().await?;
        let bytes = if let Some(actor_principal) = request.actor_principal.as_deref() {
            let principal = Principal::from_text(actor_principal).map_err(|err| {
                RuntimeError::Domain(format!("invalid actor principal for governance: {err}"))
            })?;
            let arg = Encode!(
                &request.space_id,
                &request.action_target,
                &request.domain_mode,
                &request.gate_level,
                &principal
            )
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
            agent
                .query(&self.canister_id, "evaluate_action_scope_with_actor")
                .with_arg(arg)
                .call()
                .await
                .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?
        } else {
            let arg = Encode!(
                &request.space_id,
                &request.action_target,
                &request.domain_mode,
                &request.gate_level
            )
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
            agent
                .query(&self.canister_id, "evaluate_action_scope_with_gate")
                .with_arg(arg)
                .call()
                .await
                .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?
        };

        let decoded = Decode!(&bytes, ActionScopeEvaluationCandid)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(GovernanceScopeEvaluation {
            allowed: decoded.allowed,
            reason: decoded.reason,
            effective_weight: decoded.effective_weight,
            requires_review: decoded.requires_review,
            gate_decision: decoded.gate_decision,
            required_actions: decoded.required_actions,
            policy_ref: decoded.policy_ref,
            policy_version: decoded.policy_version,
        })
    }

    async fn get_actor_role_binding(
        &self,
        space_id: &str,
        principal: &str,
    ) -> Result<Option<ActorRoleBinding>, RuntimeError> {
        let principal = Principal::from_text(principal)
            .map_err(|err| RuntimeError::Domain(format!("invalid actor principal: {err}")))?;
        let agent = self.agent().await?;
        let arg = Encode!(&space_id.to_string(), &principal)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_actor_role_binding")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;

        let decoded = Decode!(&bytes, Option<ActorRoleBindingCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;

        Ok(decoded.map(|binding| ActorRoleBinding {
            space_id: binding.space_id,
            principal: binding.principal.to_text(),
            role: binding.role,
            source_ref: binding.source_ref,
            updated_at: binding.updated_at,
        }))
    }
}
