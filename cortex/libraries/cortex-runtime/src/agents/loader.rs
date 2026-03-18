use crate::RuntimeError;
use cortex_domain::agent::{AgentIntention, AgentState, IntentionStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub name: String,
    pub role: String,
    pub goal: String,
    pub target_resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepManifest {
    pub id: String,
    pub step_type: String,
    pub agent_role: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowManifest {
    pub id: String,
    pub name: String,
    pub trigger: Option<String>,
    pub steps: Vec<WorkflowStepManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub agents: Option<Vec<AgentManifest>>,
    pub workflows: Option<Vec<WorkflowManifest>>,
}

pub fn parse_module_yaml(yaml: &str) -> Result<ModuleManifest, RuntimeError> {
    serde_yaml::from_str(yaml)
        .map_err(|e| RuntimeError::Serialization(format!("Failed to parse module YAML: {}", e)))
}

pub fn build_agent_states(manifest: &ModuleManifest) -> Vec<AgentState> {
    manifest
        .agents
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|agent_manifest| {
            let mut state = AgentState::new(&agent_manifest.name);
            state.role = Some(agent_manifest.role.clone());
            state.set_intention(AgentIntention {
                id: format!("{}_boot", agent_manifest.name),
                description: agent_manifest.goal.clone(),
                target_resource: agent_manifest
                    .target_resource
                    .clone()
                    .unwrap_or_else(|| "nostra://default".to_string()),
                status: IntentionStatus::Pending,
            });
            state
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module_yaml() {
        let yaml = r#"
id: "nostra.modules.research"
version: "1.0.0"
name: "Research Synthesis"
description: "Turn user feedback into concrete research initiatives."
agents:
  - name: "Sam the Sentiment Analyst"
    role: "Analyst"
    goal: "Detect patterns in feedback."
    target_resource: "nostra://graph/feedback"
workflows:
  - id: "feedback-synthesis"
    name: "Synthesize Feedback to Research"
    trigger: "schedule"
    steps:
      - id: "analyze_clusters"
        step_type: "AsyncExternalOp"
        agent_role: "Analyst"
        instruction: "Cluster this feedback into topics."
"#;
        let manifest = parse_module_yaml(yaml).unwrap();

        assert_eq!(manifest.id, "nostra.modules.research");
        assert_eq!(manifest.version, "1.0.0");

        let agents = manifest.agents.as_ref().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "Sam the Sentiment Analyst");
        assert_eq!(agents[0].role, "Analyst");

        let states = build_agent_states(&manifest);
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].role.as_deref(), Some("Analyst"));
        assert!(states[0].active_intention.is_some());
    }
}
