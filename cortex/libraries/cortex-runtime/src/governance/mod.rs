use crate::RuntimeError;
use crate::ports::{
    ActorRoleBinding, GovernanceAdapter, GovernanceScopeEvaluation, GovernanceScopeRequest,
};

pub struct GovernanceRuntime<A>
where
    A: GovernanceAdapter,
{
    adapter: A,
}

impl<A> GovernanceRuntime<A>
where
    A: GovernanceAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub async fn evaluate_action_scope(
        &self,
        request: GovernanceScopeRequest,
    ) -> Result<GovernanceScopeEvaluation, RuntimeError> {
        self.adapter.evaluate_action_scope(request).await
    }

    pub async fn get_actor_role_binding(
        &self,
        space_id: &str,
        principal: &str,
    ) -> Result<Option<ActorRoleBinding>, RuntimeError> {
        self.adapter
            .get_actor_role_binding(space_id, principal)
            .await
    }
}
