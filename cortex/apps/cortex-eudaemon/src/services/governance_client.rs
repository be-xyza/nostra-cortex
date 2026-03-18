use candid::Principal;
use cortex_ic_adapter::governance::GovernanceCanisterClient;
use cortex_runtime::ports::{
    ActorRoleBinding as RuntimeActorRoleBinding, GovernanceAdapter, GovernanceScopeEvaluation,
    GovernanceScopeRequest,
};

pub type ActionScopeEvaluation = GovernanceScopeEvaluation;

#[derive(Clone, Debug, PartialEq)]
pub struct ActorRoleBinding {
    pub space_id: String,
    pub principal: Principal,
    pub role: String,
    pub source_ref: Option<String>,
    pub updated_at: u64,
}

#[derive(Clone, Debug)]
pub struct GovernanceClient {
    inner: GovernanceCanisterClient,
}

impl GovernanceClient {
    pub async fn from_env() -> Result<Self, String> {
        let inner = GovernanceCanisterClient::from_env().await.map_err(|err| err.to_string())?;
        Ok(Self { inner })
    }

    pub async fn evaluate_action_scope_with_actor(
        &self,
        space_id: &str,
        action_target: &str,
        domain_mode: &str,
        gate_level: &str,
        actor_principal: &Principal,
    ) -> Result<ActionScopeEvaluation, String> {
        self.inner
            .evaluate_action_scope(GovernanceScopeRequest {
                space_id: space_id.to_string(),
                action_target: action_target.to_string(),
                domain_mode: domain_mode.to_string(),
                gate_level: gate_level.to_string(),
                actor_principal: Some(actor_principal.to_text()),
            })
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn get_actor_role_binding(
        &self,
        space_id: &str,
        principal: &Principal,
    ) -> Result<Option<ActorRoleBinding>, String> {
        self.inner
            .get_actor_role_binding(space_id, &principal.to_text())
            .await
            .map(|binding| binding.map(map_binding))
            .map_err(|err| err.to_string())
    }
}

fn map_binding(source: RuntimeActorRoleBinding) -> ActorRoleBinding {
    let principal =
        Principal::from_text(&source.principal).unwrap_or_else(|_| Principal::anonymous());
    ActorRoleBinding {
        space_id: source.space_id,
        principal,
        role: source.role,
        source_ref: source.source_ref,
        updated_at: source.updated_at,
    }
}
