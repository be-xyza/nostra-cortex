use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Represents a running workflow instance
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub current_state: String,
    pub data: HashMap<String, String>, // Serialized JSON strings
    pub history: Vec<StateTransition>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub timestamp: u64,
    pub trigger: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum ActionType {
    RenderForm {
        fields: Vec<FormField>,
    },
    TemplateHydrate {
        template_id: String,
        data: HashMap<String, String>,
    },
    EditorOpen {
        path: String,
    },
    TaskCreate {
        title: String,
        assignee: String,
        description: String,
    },
    NotificationSend {
        recipients: Vec<String>,
        message: String,
    },
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

pub struct StateMachine {
    instances: HashMap<String, WorkflowInstance>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn create_instance(&mut self, workflow_id: String, initial_state: String) -> String {
        let id = format!("wf_{}", uuid::Uuid::new_v4());
        let instance = WorkflowInstance {
            id: id.clone(),
            workflow_id,
            current_state: initial_state,
            data: HashMap::new(),
            history: Vec::new(),
        };
        self.instances.insert(id.clone(), instance);
        id
    }

    pub fn transition(
        &mut self,
        instance_id: &str,
        trigger: &str,
        data: HashMap<String, String>,
    ) -> Result<String, String> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;

        // Store form data
        instance.data.extend(data);

        // Simplified transition logic - in real implementation would parse workflow YAML
        let next_state = match instance.current_state.as_str() {
            "RENDER_FORM" => "GENERATE_ARTIFACT",
            "GENERATE_ARTIFACT" => "OPEN_EDITOR",
            "OPEN_EDITOR" => "COMPLETE",
            _ => "COMPLETE",
        };

        instance.history.push(StateTransition {
            from_state: instance.current_state.clone(),
            to_state: next_state.to_string(),
            timestamp: 0, // TODO: get current timestamp
            trigger: trigger.to_string(),
        });

        instance.current_state = next_state.to_string();
        Ok(next_state.to_string())
    }

    pub fn get_instance(&self, instance_id: &str) -> Option<&WorkflowInstance> {
        self.instances.get(instance_id)
    }
}
