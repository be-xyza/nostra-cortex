use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::temporal::Activity;
use crate::translator::translate_action_target;
use cortex_domain::graph::{EdgeKind, Graph, Node};
use cortex_domain::integrity::{
    Constraint, Direction, EdgeSelector, IntegrityPredicate, IntegrityRule, IntegrityScope,
    NodeSelector, Severity,
};
use cortex_domain::simulation::scenario::{
    ScenarioConstraints, ScenarioDefinition, ScenarioMetadata, ScenarioRound, ScenarioRoundAction,
};
use cortex_domain::simulation::session::{SimulationAction, run_deterministic_session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePlanInput {
    pub scenario_id: String,
    pub action_targets_json: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePlanOutput {
    pub success: bool,
    pub violation_count: usize,
    pub risk_score: usize,
    pub siqs_score: f32,
    pub session_id: String,
    pub structural_diff_summary: String,
}

pub struct EvaluateSimulationPlanActivity;

#[async_trait]
impl Activity<EvaluatePlanInput, EvaluatePlanOutput> for EvaluateSimulationPlanActivity {
    const NAME: &'static str = "EvaluateSimulationPlan";

    async fn execute(&self, input: EvaluatePlanInput) -> Result<EvaluatePlanOutput, String> {
        let translated_actions = input
            .action_targets_json
            .iter()
            .map(|target| translate_action_target(target))
            .collect::<Result<Vec<_>, _>>()?;

        let scenario = build_scenario(&input.scenario_id, &translated_actions);
        let base_graph = build_base_graph(&input.scenario_id);
        let rules = default_rules();
        let session = run_deterministic_session(&base_graph, &rules, &scenario);
        let result = session
            .result
            .clone()
            .ok_or_else(|| "Simulation session returned no result".to_string())?;
        let summary = &result.violation_summary;
        let risk_score = summary.risk_score;
        let siqs_score = (100.0_f32 - (risk_score as f32 * 4.0)).clamp(0.0, 100.0);
        let structural_diff_summary = format!(
            "nodes_added={} nodes_removed={} edges_added={} edges_removed={} attrs_changed={}",
            result.structural_diff.nodes_added.len(),
            result.structural_diff.nodes_removed.len(),
            result.structural_diff.edges_added.len(),
            result.structural_diff.edges_removed.len(),
            result.structural_diff.attributes_changed.len(),
        );

        Ok(EvaluatePlanOutput {
            success: !result.aborted && summary.critical == 0 && summary.violation == 0,
            violation_count: result.violations.len(),
            risk_score,
            siqs_score,
            session_id: session.session_id,
            structural_diff_summary,
        })
    }
}

fn build_base_graph(scenario_id: &str) -> Graph {
    let mut graph = Graph::default();
    graph.add_node(Node {
        id: format!("space:{scenario_id}"),
        node_type: "space".to_string(),
        attributes: std::collections::BTreeMap::new(),
    });
    graph
}

fn default_rules() -> Vec<IntegrityRule> {
    vec![
        IntegrityRule {
            id: "rule-space-exists".to_string(),
            name: "Space root must exist".to_string(),
            description: "Simulation graph must include a space root node.".to_string(),
            scope: IntegrityScope::Global,
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("space".to_string()),
                    tags: None,
                },
                relation: None,
                constraint: Constraint::MustExist,
            },
            severity: Severity::Critical,
            remediation_hint: Some("Ensure scenario bootstraps a space node.".to_string()),
        },
        IntegrityRule {
            id: "rule-context-node-has-dependency".to_string(),
            name: "Context nodes should link to dependencies".to_string(),
            description: "Context nodes are expected to declare at least one dependency edge."
                .to_string(),
            scope: IntegrityScope::Global,
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("context_node".to_string()),
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::MinCount(1),
            },
            severity: Severity::Warning,
            remediation_hint: Some(
                "Attach depends_on edges for newly introduced context nodes.".to_string(),
            ),
        },
    ]
}

fn build_scenario(scenario_id: &str, actions: &[SimulationAction]) -> ScenarioDefinition {
    let scenario_actions = actions
        .iter()
        .map(action_to_round_action)
        .collect::<Vec<_>>();
    ScenarioDefinition {
        scenario: ScenarioMetadata {
            id: scenario_id.to_string(),
            name: format!("Evaluated plan for {scenario_id}"),
            seed: deterministic_seed(scenario_id),
            commons_version: "nostra-core-v0".to_string(),
            siqs_version: "1.0.0".to_string(),
        },
        constraints: ScenarioConstraints {
            max_mutations: actions.len().max(1),
            max_rounds: 1,
            max_runtime_ms: 1_000,
        },
        rounds: vec![ScenarioRound {
            round: 1,
            actions: scenario_actions,
        }],
    }
}

fn action_to_round_action(action: &SimulationAction) -> ScenarioRoundAction {
    match action {
        SimulationAction::AddNode {
            node_id,
            node_type,
            attributes,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "add_node".to_string(),
            node_id: Some(node_id.clone()),
            node_type: Some(node_type.clone()),
            attributes: Some(attributes.clone()),
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: None,
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: None,
        },
        SimulationAction::RemoveNode { node_id } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "remove_node".to_string(),
            node_id: Some(node_id.clone()),
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: None,
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: None,
        },
        SimulationAction::AddEdge {
            source,
            target,
            edge_kind,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "add_edge".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: Some(source.clone()),
            target: Some(target.clone()),
            edge_kind: Some(match edge_kind {
                EdgeKind::DependsOn => "depends_on".to_string(),
                EdgeKind::Contradicts => "contradicts".to_string(),
                EdgeKind::Supersedes => "supersedes".to_string(),
                EdgeKind::Implements => "implements".to_string(),
                EdgeKind::Invalidates => "invalidates".to_string(),
                EdgeKind::Requires => "requires".to_string(),
                EdgeKind::Assumes => "assumes".to_string(),
                EdgeKind::ConstitutionalBasis => "constitutional_basis".to_string(),
                EdgeKind::DerivesFrom => "derives_from".to_string(),
                EdgeKind::ForkedInto => "forked_into".to_string(),
                EdgeKind::Governs => "governs".to_string(),
                EdgeKind::Produces => "produces".to_string(),
                EdgeKind::References => "references".to_string(),
                EdgeKind::Custom(value) => value.clone(),
            }),
            proposal_type: None,
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: None,
        },
        SimulationAction::RemoveEdge {
            source,
            target,
            edge_kind,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "remove_edge".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: Some(source.clone()),
            target: Some(target.clone()),
            edge_kind: Some(match edge_kind {
                EdgeKind::DependsOn => "depends_on".to_string(),
                EdgeKind::Contradicts => "contradicts".to_string(),
                EdgeKind::Supersedes => "supersedes".to_string(),
                EdgeKind::Implements => "implements".to_string(),
                EdgeKind::Invalidates => "invalidates".to_string(),
                EdgeKind::Requires => "requires".to_string(),
                EdgeKind::Assumes => "assumes".to_string(),
                EdgeKind::ConstitutionalBasis => "constitutional_basis".to_string(),
                EdgeKind::DerivesFrom => "derives_from".to_string(),
                EdgeKind::ForkedInto => "forked_into".to_string(),
                EdgeKind::Governs => "governs".to_string(),
                EdgeKind::Produces => "produces".to_string(),
                EdgeKind::References => "references".to_string(),
                EdgeKind::Custom(value) => value.clone(),
            }),
            proposal_type: None,
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: None,
        },
        SimulationAction::SubmitProposal {
            proposal_type,
            payload,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "submit_proposal".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: Some(proposal_type.clone()),
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: serde_json::from_str(payload).ok(),
        },
        SimulationAction::CastVote {
            proposal_id,
            choice,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "cast_vote".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: None,
            proposal: Some(proposal_id.clone()),
            choice: Some(choice.clone()),
            key: None,
            value: None,
            payload: None,
        },
        SimulationAction::ModifyAttribute {
            node_id,
            key,
            value,
        } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "modify_attribute".to_string(),
            node_id: Some(node_id.clone()),
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: None,
            proposal: None,
            choice: None,
            key: Some(key.clone()),
            value: Some(value.clone()),
            payload: None,
        },
        SimulationAction::AgentAction { target } => ScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "submit_proposal".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: Some("agent_action".to_string()),
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: serde_json::from_str(target).ok(),
        },
    }
}

fn deterministic_seed(value: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    u64::from_be_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ])
}
