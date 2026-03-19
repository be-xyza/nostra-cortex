use cortex_domain::agent::contracts::{ActionTarget, AgentIntent};
use cortex_domain::simulation::session::SimulationAction;

/// Translates a high-level agent `ActionTarget` into a deterministic
/// structural mutation intended for GSMS deep evaluation.
pub fn translate_action_target(action_target_json: &str) -> Result<SimulationAction, String> {
    if let Ok(intent) = serde_json::from_str::<AgentIntent>(action_target_json) {
        return Ok(match intent {
            AgentIntent::CreateContextNode { node_id, content } => {
                let mut attributes = std::collections::BTreeMap::new();
                attributes.insert("content".to_string(), content);
                SimulationAction::AddNode {
                    node_id,
                    node_type: "context_node".to_string(),
                    attributes,
                }
            }
            AgentIntent::ProposeSchemaMutation { schema_json } => {
                let mut attributes = std::collections::BTreeMap::new();
                attributes.insert("schema_json".to_string(), schema_json);
                SimulationAction::AddNode {
                    node_id: format!("schema_mutation_{}", short_hash(action_target_json)),
                    node_type: "schema_mutation_proposal".to_string(),
                    attributes,
                }
            }
            AgentIntent::ExecuteSimulation { scenario_id } => SimulationAction::SubmitProposal {
                proposal_type: "execute_simulation".to_string(),
                payload: serde_json::json!({ "scenarioId": scenario_id }).to_string(),
            },
            AgentIntent::ApplyActionTarget { action_target } => {
                simulation_action_from_action_target(action_target)
            }
        });
    }

    let target: ActionTarget = serde_json::from_str(action_target_json)
        .map_err(|e| format!("Failed to parse ActionTarget: {}", e))?;
    Ok(simulation_action_from_action_target(target))
}

fn simulation_action_from_action_target(target: ActionTarget) -> SimulationAction {
    match target.method.as_str() {
        "create_context_node" => {
            let payload_json = payload_json_value(&target.payload);
            let node_id = payload_json
                .get("nodeId")
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("context_node_from_action")
                .to_string();
            let content = payload_json
                .get("content")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let mut attributes = std::collections::BTreeMap::new();
            attributes.insert("content".to_string(), content);
            SimulationAction::AddNode {
                node_id,
                node_type: "context_node".to_string(),
                attributes,
            }
        }
        "propose_schema_mutation" => {
            let payload_json = payload_json_value(&target.payload);
            let schema_json = payload_json
                .get("schemaJson")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let mut attributes = std::collections::BTreeMap::new();
            attributes.insert("schema_json".to_string(), schema_json);
            SimulationAction::AddNode {
                node_id: format!("schema_mutation_{}", short_hash(&target.address)),
                node_type: "schema_mutation_proposal".to_string(),
                attributes,
            }
        }
        "execute_simulation" => SimulationAction::SubmitProposal {
            proposal_type: "execute_simulation".to_string(),
            payload: String::from_utf8_lossy(&target.payload).to_string(),
        },
        _ => {
            let serialized = serde_json::json!({
                "protocol": target.protocol,
                "address": target.address,
                "method": target.method,
                "payload": payload_json_value(&target.payload),
            });
            SimulationAction::AgentAction {
                target: serialized.to_string(),
            }
        }
    }
}

fn payload_json_value(payload: &[u8]) -> serde_json::Value {
    serde_json::from_slice(payload).unwrap_or_else(|_| serde_json::json!({}))
}

fn short_hash(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)[..12].to_string()
}
