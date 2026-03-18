use cortex_domain::agent::contracts::{ActionTarget, AgentIntent};
use serde_json::Value;

#[derive(Debug)]
pub enum ToolError {
    ParseError(String),
    MissingCapability(String),
}

/// The core Tool Execution mapping traits.
pub trait CortexTool: Send + Sync {
    /// The canonical name of the tool, matching what the LLM will see.
    fn name(&self) -> &'static str;

    /// The standard JSON schema definition expected by OpenAI / Anthropic APIs
    fn json_schema(&self) -> Value;

    /// Core capability declaration based on Section 8.1 of Research 122
    fn requires_http(&self) -> bool {
        false
    }
    fn requires_secrets(&self) -> bool {
        false
    }
    fn requires_tool_invoke(&self) -> bool {
        false
    }

    /// Parses raw tool args into a canonical host `ActionTarget`.
    fn parse(&self, input: &Value) -> Result<ActionTarget, ToolError>;
}

pub fn intent_to_action_target(intent: AgentIntent) -> ActionTarget {
    match intent {
        AgentIntent::CreateContextNode { node_id, content } => ActionTarget {
            protocol: "ic".to_string(),
            address: "kg-canister".to_string(),
            method: "create_context_node".to_string(),
            payload: serde_json::to_vec(&serde_json::json!({
                "nodeId": node_id,
                "content": content
            }))
            .unwrap_or_default(),
        },
        AgentIntent::ProposeSchemaMutation { schema_json } => ActionTarget {
            protocol: "ic".to_string(),
            address: "kg-canister".to_string(),
            method: "propose_schema_mutation".to_string(),
            payload: serde_json::to_vec(&serde_json::json!({
                "schemaJson": schema_json
            }))
            .unwrap_or_default(),
        },
        AgentIntent::ExecuteSimulation { scenario_id } => ActionTarget {
            protocol: "ic".to_string(),
            address: "simulation-canister".to_string(),
            method: "execute_simulation".to_string(),
            payload: serde_json::to_vec(&serde_json::json!({
                "scenarioId": scenario_id
            }))
            .unwrap_or_default(),
        },
        AgentIntent::ApplyActionTarget { action_target } => action_target,
    }
}
