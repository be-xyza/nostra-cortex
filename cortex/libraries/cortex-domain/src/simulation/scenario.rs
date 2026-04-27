use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ScenarioDefinition {
    pub scenario: ScenarioMetadata,
    pub constraints: ScenarioConstraints,
    #[serde(default)]
    pub rounds: Vec<ScenarioRound>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ScenarioMetadata {
    pub id: String,
    pub name: String,
    pub seed: u64,
    pub commons_version: String,
    pub siqs_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ScenarioConstraints {
    pub max_mutations: usize,
    pub max_rounds: usize,
    pub max_runtime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ScenarioRound {
    pub round: usize,
    #[serde(default)]
    pub actions: Vec<ScenarioRoundAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ScenarioRoundAction {
    pub actor: String,
    pub action: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub node_type: Option<String>,
    #[serde(default)]
    pub attributes: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub edge_kind: Option<String>,
    #[serde(default)]
    pub proposal_type: Option<String>,
    #[serde(default)]
    pub proposal: Option<String>,
    #[serde(default)]
    pub choice: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CanonicalScenarioAction {
    pub round: usize,
    pub actor: String,
    pub action: ScenarioRoundAction,
}

pub fn parse_scenario_yaml(raw: &str) -> Result<ScenarioDefinition, String> {
    serde_yaml::from_str::<ScenarioDefinition>(raw).map_err(|err| err.to_string())
}

pub fn canonical_actions(definition: &ScenarioDefinition) -> Vec<CanonicalScenarioAction> {
    let mut rounds = definition.rounds.clone();
    rounds.sort_by_key(|round| round.round);

    let mut output = Vec::new();
    for round in rounds {
        for action in round.actions {
            output.push(CanonicalScenarioAction {
                round: round.round,
                actor: action.actor.clone(),
                action,
            });
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_canonicalize_yaml_scenario() {
        let raw = r#"
scenario:
  id: "scenario-1"
  name: "Test"
  seed: 42
  commons_version: "nostra-core-v0"
  siqs_version: "1.0.0"
constraints:
  max_mutations: 10
  max_rounds: 5
  max_runtime_ms: 100
rounds:
  - round: 2
    actions:
      - actor: "moderator"
        action: "cast_vote"
        proposal: "proposal-1"
        choice: "yes"
  - round: 1
    actions:
      - actor: "author"
        action: "add_node"
        node_id: "n1"
        node_type: "proposal"
"#;

        let parsed = parse_scenario_yaml(raw).expect("scenario should parse");
        let canonical = canonical_actions(&parsed);
        assert_eq!(canonical[0].round, 1);
        assert_eq!(canonical[1].round, 2);
    }
}
