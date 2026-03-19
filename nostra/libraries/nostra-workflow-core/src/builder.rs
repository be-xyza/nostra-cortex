//! Workflow Builder Module
//!
//! Provides tools for creating and loading workflow definitions:
//! - YAML/JSON parsing from files or strings
//! - In-memory workflow registry
//! - Template expansion

use crate::primitives::{
    Action, AsyncProviderStrategy, AsyncRetryPolicy, Step, Transition,
};
use crate::types::{StepId, WorkflowDefinition};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- YAML Schema Types ---
// These match the CNCF Serverless Workflow Spec patterns

/// A workflow definition in YAML format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowYaml {
    /// Unique identifier for the workflow.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
    /// The starting step ID.
    pub start: String,
    /// List of workflow steps.
    pub steps: Vec<StepYaml>,
}

/// A step definition in YAML format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepYaml {
    /// Unique step identifier.
    pub id: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// The action to perform (enum tag).
    pub action: ActionYaml,
    /// Transitions to next steps.
    #[serde(default)]
    pub transitions: Vec<TransitionYaml>,
    /// Optional compensation step for Saga pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensated_by: Option<String>,
}

/// Action definition in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionYaml {
    /// User task requiring interaction.
    UserTask {
        description: String,
        #[serde(default)]
        roles: Vec<String>,
        #[serde(default)]
        users: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        a2ui_schema: Option<String>,
    },
    /// System operation (automated).
    SystemOp { op_type: String, payload: String },
    /// Async external operation (agent calls).
    AsyncOp {
        target: String,
        input: String,
        #[serde(default = "default_timeout")]
        timeout_secs: u64,
        #[serde(default)]
        retry_policy: AsyncRetryPolicy,
        #[serde(default)]
        provider_strategy: AsyncProviderStrategy,
    },
    /// No-op (pass-through).
    None,
}

fn default_timeout() -> u64 {
    300
}

/// Transition definition in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionYaml {
    /// Target step ID.
    pub to: String,
    /// Optional condition expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

// --- Parser ---

/// Parses workflow definitions from YAML or JSON.
pub struct WorkflowParser;

impl WorkflowParser {
    /// Parse a workflow definition from YAML string.
    pub fn from_yaml(yaml: &str) -> Result<WorkflowDefinition> {
        let parsed: WorkflowYaml =
            serde_yaml::from_str(yaml).context("Failed to parse workflow YAML")?;
        Self::convert(parsed)
    }

    /// Parse a workflow definition from JSON string.
    pub fn from_json(json: &str) -> Result<WorkflowDefinition> {
        let parsed: WorkflowYaml =
            serde_json::from_str(json).context("Failed to parse workflow JSON")?;
        Self::convert(parsed)
    }

    /// Convert parsed YAML to WorkflowDefinition.
    fn convert(yaml: WorkflowYaml) -> Result<WorkflowDefinition> {
        let mut steps: HashMap<StepId, Step> = HashMap::new();

        for step_yaml in yaml.steps {
            let action = match step_yaml.action {
                ActionYaml::UserTask {
                    description,
                    roles,
                    users,
                    a2ui_schema,
                } => Action::UserTask {
                    description,
                    candidate_roles: roles,
                    candidate_users: users,
                    a2ui_schema,
                },
                ActionYaml::SystemOp { op_type, payload } => Action::SystemOp { op_type, payload },
                ActionYaml::AsyncOp {
                    target,
                    input,
                    timeout_secs,
                    retry_policy,
                    provider_strategy,
                } => Action::AsyncExternalOp {
                    target,
                    input,
                    timeout_secs,
                    retry_policy,
                    provider_strategy,
                },
                ActionYaml::None => Action::None,
            };

            let transitions: Vec<Transition> = step_yaml
                .transitions
                .into_iter()
                .map(|t| Transition::to(t.to))
                .collect();

            let step = Step {
                id: step_yaml.id.clone(),
                description: step_yaml.description,
                action,
                transitions,
                compensated_by: step_yaml.compensated_by,
            };

            steps.insert(step_yaml.id, step);
        }

        Ok(WorkflowDefinition {
            id: yaml.id,
            steps,
            start_step_id: yaml.start,
        })
    }
}

// --- Registry ---

/// In-memory workflow definition registry.
#[derive(Default)]
pub struct WorkflowRegistry {
    definitions: HashMap<String, WorkflowDefinition>,
}

impl WorkflowRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a workflow definition.
    pub fn register(&mut self, definition: WorkflowDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Get a workflow definition by ID.
    pub fn get(&self, id: &str) -> Option<&WorkflowDefinition> {
        self.definitions.get(id)
    }

    /// List all workflow IDs.
    pub fn list(&self) -> Vec<&str> {
        self.definitions.keys().map(|s| s.as_str()).collect()
    }

    /// Load a workflow from YAML and register it.
    pub fn load_yaml(&mut self, yaml: &str) -> Result<String> {
        let def = WorkflowParser::from_yaml(yaml)?;
        let id = def.id.clone();
        self.register(def);
        Ok(id)
    }
}

// --- Common Templates ---

/// Predefined workflow templates.
pub struct WorkflowTemplates;

impl WorkflowTemplates {
    /// Approval workflow template.
    pub fn approval() -> &'static str {
        r#"
id: approval_workflow
name: Simple Approval
description: A basic two-step approval workflow.
start: submit

steps:
  - id: submit
    description: Submit request for approval
    action:
      type: user_task
      description: "Submit your request"
      roles: ["requester"]
    transitions:
      - to: approve

  - id: approve
    description: Approve or reject the request
    action:
      type: user_task
      description: "Review and approve"
      roles: ["approver"]
    transitions:
      - to: complete

  - id: complete
    description: Workflow complete
    action:
      type: none
"#
    }

    /// Governance voting workflow template.
    pub fn governance_vote() -> &'static str {
        r#"
id: governance_vote
name: Governance Proposal
description: Multi-step governance proposal and voting.
start: draft

steps:
  - id: draft
    description: Draft the proposal
    action:
      type: user_task
      description: "Write your proposal"
      roles: ["proposer"]
    transitions:
      - to: review

  - id: review
    description: Community review period
    action:
      type: user_task
      description: "Review proposal and provide feedback"
      roles: ["reviewer"]
    transitions:
      - to: vote

  - id: vote
    description: Community voting
    action:
      type: user_task
      description: "Cast your vote"
      roles: ["voter"]
      a2ui_schema: |
        {"id": "vote_form", "root": {"id": "c1", "type": "Column", "children": [
          {"id": "h", "type": "Heading", "text": "Cast Your Vote"},
          {"id": "y", "type": "Button", "label": "Vote Yes", "action": "submit"},
          {"id": "n", "type": "Button", "label": "Vote No", "action": "reject"}
        ]}}
    transitions:
      - to: execute

  - id: execute
    description: Execute approved proposal
    action:
      type: system_op
      op_type: "Governance.Execute"
      payload: "{}"
    transitions:
      - to: complete

  - id: complete
    description: Governance complete
    action:
      type: none
"#
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_approval() {
        let yaml = WorkflowTemplates::approval();
        let result = WorkflowParser::from_yaml(yaml);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let def = result.unwrap();
        assert_eq!(def.id, "approval_workflow");
        assert_eq!(def.start_step_id, "submit");
        assert_eq!(def.steps.len(), 3);
    }

    #[test]
    fn test_parse_yaml_governance() {
        let yaml = WorkflowTemplates::governance_vote();
        let result = WorkflowParser::from_yaml(yaml);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let def = result.unwrap();
        assert_eq!(def.id, "governance_vote");
        assert_eq!(def.steps.len(), 5);
    }

    #[test]
    fn test_registry() {
        let mut registry = WorkflowRegistry::new();
        registry.load_yaml(WorkflowTemplates::approval()).unwrap();
        registry
            .load_yaml(WorkflowTemplates::governance_vote())
            .unwrap();

        assert_eq!(registry.list().len(), 2);
        assert!(registry.get("approval_workflow").is_some());
        assert!(registry.get("governance_vote").is_some());
    }
}
