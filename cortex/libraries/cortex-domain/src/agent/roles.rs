use crate::agent::{AgentIntention, AgentState, IntentionStatus};

pub trait AgentRole {
    fn initialize_state(&self, identity: &str) -> AgentState;
}

pub struct DispatcherRole;

impl AgentRole for DispatcherRole {
    fn initialize_state(&self, identity: &str) -> AgentState {
        let mut state = AgentState::new(identity);
        state.set_intention(AgentIntention {
            id: format!("{}_init", identity),
            description: "Monitor workflow engine for new completed instances to route via GovernanceAdapter.".to_string(),
            target_resource: "nostra://workflow_engine/events".to_string(),
            status: IntentionStatus::Pending,
        });
        state
    }
}

pub struct ComplianceOfficerRole;

impl AgentRole for ComplianceOfficerRole {
    fn initialize_state(&self, identity: &str) -> AgentState {
        let mut state = AgentState::new(identity);
        state.set_intention(AgentIntention {
            id: format!("{}_init", identity),
            description: "Audit space Configurations against Organizational Policy.".to_string(),
            target_resource: "nostra://spaces/all".to_string(),
            status: IntentionStatus::Pending,
        });
        state
    }
}

pub struct GardenerRole;

impl AgentRole for GardenerRole {
    fn initialize_state(&self, identity: &str) -> AgentState {
        let mut state = AgentState::new(identity);
        state.set_intention(AgentIntention {
            id: format!("{}_init", identity),
            description: "Query Knowledge Graph for orphaned nodes and generate prune proposals."
                .to_string(),
            target_resource: "nostra://knowledge_graph/nodes".to_string(),
            status: IntentionStatus::Pending,
        });
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_role_initialization() {
        let role = DispatcherRole;
        let state = role.initialize_state("agent_dispatch_01");

        assert_eq!(state.identity, "agent_dispatch_01");
        assert!(state.active_intention.is_some());

        let intention = state.active_intention.unwrap();
        assert_eq!(intention.id, "agent_dispatch_01_init");
        assert_eq!(intention.target_resource, "nostra://workflow_engine/events");
        assert_eq!(intention.status, IntentionStatus::Pending);
    }

    #[test]
    fn test_compliance_officer_role_initialization() {
        let role = ComplianceOfficerRole;
        let state = role.initialize_state("agent_compliance_02");

        assert_eq!(state.identity, "agent_compliance_02");
        assert!(state.active_intention.is_some());

        let intention = state.active_intention.unwrap();
        assert_eq!(intention.id, "agent_compliance_02_init");
        assert_eq!(intention.target_resource, "nostra://spaces/all");
        assert_eq!(intention.status, IntentionStatus::Pending);
    }

    #[test]
    fn test_gardener_role_initialization() {
        let role = GardenerRole;
        let state = role.initialize_state("agent_gardener_03");

        assert_eq!(state.identity, "agent_gardener_03");
        assert!(state.active_intention.is_some());

        let intention = state.active_intention.unwrap();
        assert_eq!(intention.id, "agent_gardener_03_init");
        assert_eq!(intention.target_resource, "nostra://knowledge_graph/nodes");
        assert_eq!(intention.status, IntentionStatus::Pending);
    }
}
