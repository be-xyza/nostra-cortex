use super::scenario::{canonical_actions, ScenarioDefinition};
use crate::graph::diff::{structural_graph, StructuralDiff};
use crate::graph::{Edge, EdgeKind, Graph, Node};
use crate::integrity::{evaluate_all, IntegrityRule, IntegrityViolation, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationSession {
    pub session_id: String,
    pub scenario_id: String,
    pub seed: u64,
    pub graph_root_hash: String,
    pub commons_version: String,
    pub siqs_version: String,
    pub constraints: SimulationConstraints,
    pub mutation_log: Vec<SimulationMutation>,
    pub result: Option<SimulationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationConstraints {
    pub max_mutations: usize,
    pub max_rounds: usize,
    pub max_runtime_ms: u64,
    pub deterministic_seed_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationMutation {
    pub round: usize,
    pub actor_id: String,
    pub action: SimulationAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SimulationAction {
    AddNode {
        node_id: String,
        node_type: String,
        attributes: BTreeMap<String, String>,
    },
    RemoveNode {
        node_id: String,
    },
    AddEdge {
        source: String,
        target: String,
        edge_kind: EdgeKind,
    },
    RemoveEdge {
        source: String,
        target: String,
        edge_kind: EdgeKind,
    },
    SubmitProposal {
        proposal_type: String,
        payload: String,
    },
    CastVote {
        proposal_id: String,
        choice: String,
    },
    ModifyAttribute {
        node_id: String,
        key: String,
        value: String,
    },
    // Allows the agent's ActionTarget payload to be integrated directly
    AgentAction {
        target: String, // Serialize the action target json representation
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationResult {
    pub mutation_count: usize,
    pub structural_diff: StructuralDiff,
    pub violations: Vec<IntegrityViolation>,
    pub violation_summary: ViolationSummary,
    pub aborted: bool,
    pub abort_reason: Option<String>,
    pub final_graph_hash: String,
    pub bench_metrics: Option<BenchMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViolationSummary {
    pub critical: usize,
    pub violation: usize,
    pub warning: usize,
    pub info: usize,
    pub risk_score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchMetrics {
    pub violation_counts: ViolationSummary,
    pub capture_probability: f64,
    pub deadlock_rate: f64,
    pub minority_suppression_index: f64,
    pub churn_index: f64,
    pub governance_throughput: f64,
}

pub fn run_deterministic_session(
    base_graph: &Graph,
    rules: &[IntegrityRule],
    scenario: &ScenarioDefinition,
) -> SimulationSession {
    let constraints = SimulationConstraints {
        max_mutations: scenario.constraints.max_mutations,
        max_rounds: scenario.constraints.max_rounds,
        max_runtime_ms: scenario.constraints.max_runtime_ms,
        deterministic_seed_required: true,
    };

    let mut session = SimulationSession {
        session_id: format!("{}:{}", scenario.scenario.id, scenario.scenario.seed),
        scenario_id: scenario.scenario.id.clone(),
        seed: scenario.scenario.seed,
        graph_root_hash: base_graph.root_hash_hex(),
        commons_version: scenario.scenario.commons_version.clone(),
        siqs_version: scenario.scenario.siqs_version.clone(),
        constraints,
        mutation_log: Vec::new(),
        result: None,
    };

    let mut graph = base_graph.clone();
    let canonical_actions = canonical_actions(scenario);
    let mut aborted = false;
    let mut abort_reason = None;

    if scenario.rounds.len() > session.constraints.max_rounds {
        aborted = true;
        abort_reason = Some(format!(
            "max_rounds_exceeded:{}>{}",
            scenario.rounds.len(),
            session.constraints.max_rounds
        ));
    }

    if !aborted {
        for (index, canonical) in canonical_actions.into_iter().enumerate() {
            if index >= session.constraints.max_mutations {
                aborted = true;
                abort_reason = Some(format!(
                    "max_mutations_exceeded:{}>{}",
                    index, session.constraints.max_mutations
                ));
                break;
            }
            if (index as u64) > session.constraints.max_runtime_ms {
                aborted = true;
                abort_reason = Some(format!(
                    "deterministic_runtime_budget_exceeded:{}>{}",
                    index, session.constraints.max_runtime_ms
                ));
                break;
            }

            let action = to_simulation_action(&canonical.action, canonical.round, index);
            apply_action(&mut graph, &action);
            session.mutation_log.push(SimulationMutation {
                round: canonical.round,
                actor_id: canonical.actor,
                action,
            });
        }
    }

    let violations = evaluate_all(rules, &graph);
    let violation_summary = summarize_violations(&violations);
    let structural_diff = structural_graph(base_graph, &graph);

    session.result = Some(SimulationResult {
        mutation_count: session.mutation_log.len(),
        structural_diff,
        violations,
        violation_summary: violation_summary.clone(),
        aborted,
        abort_reason,
        final_graph_hash: graph.root_hash_hex(),
        bench_metrics: Some(BenchMetrics {
            violation_counts: violation_summary,
            capture_probability: 0.0,
            deadlock_rate: 0.0,
            minority_suppression_index: 0.0,
            churn_index: 0.0,
            governance_throughput: 0.0,
        }),
    });

    session
}

fn to_simulation_action(
    raw: &super::scenario::ScenarioRoundAction,
    round: usize,
    index: usize,
) -> SimulationAction {
    match raw.action.as_str() {
        "add_node" => SimulationAction::AddNode {
            node_id: raw
                .node_id
                .clone()
                .unwrap_or_else(|| format!("node:{round}:{index}")),
            node_type: raw
                .node_type
                .clone()
                .unwrap_or_else(|| "generic".to_string()),
            attributes: raw.attributes.clone().unwrap_or_default(),
        },
        "remove_node" => SimulationAction::RemoveNode {
            node_id: raw
                .node_id
                .clone()
                .unwrap_or_else(|| format!("node:{round}:{index}")),
        },
        "add_edge" => SimulationAction::AddEdge {
            source: raw.source.clone().unwrap_or_default(),
            target: raw.target.clone().unwrap_or_default(),
            edge_kind: parse_edge_kind(raw.edge_kind.as_deref().unwrap_or("depends_on")),
        },
        "remove_edge" => SimulationAction::RemoveEdge {
            source: raw.source.clone().unwrap_or_default(),
            target: raw.target.clone().unwrap_or_default(),
            edge_kind: parse_edge_kind(raw.edge_kind.as_deref().unwrap_or("depends_on")),
        },
        "submit_proposal" => SimulationAction::SubmitProposal {
            proposal_type: raw
                .proposal_type
                .clone()
                .unwrap_or_else(|| "generic".to_string()),
            payload: raw
                .payload
                .as_ref()
                .and_then(|value| serde_json::to_string(value).ok())
                .unwrap_or_else(|| "{}".to_string()),
        },
        "cast_vote" => SimulationAction::CastVote {
            proposal_id: raw
                .proposal
                .clone()
                .unwrap_or_else(|| format!("proposal:{round}:{index}")),
            choice: raw.choice.clone().unwrap_or_else(|| "abstain".to_string()),
        },
        "modify_attribute" => SimulationAction::ModifyAttribute {
            node_id: raw
                .node_id
                .clone()
                .unwrap_or_else(|| format!("node:{round}:{index}")),
            key: raw.key.clone().unwrap_or_else(|| "key".to_string()),
            value: raw.value.clone().unwrap_or_default(),
        },
        _ => SimulationAction::AddNode {
            node_id: format!("fallback:{round}:{index}"),
            node_type: "unknown_action".to_string(),
            attributes: BTreeMap::from([("action".to_string(), raw.action.clone())]),
        },
    }
}

fn apply_action(graph: &mut Graph, action: &SimulationAction) {
    match action {
        SimulationAction::AddNode {
            node_id,
            node_type,
            attributes,
        } => graph.add_node(Node {
            id: node_id.clone(),
            node_type: node_type.clone(),
            attributes: attributes.clone(),
        }),
        SimulationAction::RemoveNode { node_id } => graph.remove_node(node_id),
        SimulationAction::AddEdge {
            source,
            target,
            edge_kind,
        } => graph.add_edge(Edge {
            from: source.clone(),
            to: target.clone(),
            kind: edge_kind.clone(),
        }),
        SimulationAction::RemoveEdge {
            source,
            target,
            edge_kind,
        } => graph.remove_edge(&Edge {
            from: source.clone(),
            to: target.clone(),
            kind: edge_kind.clone(),
        }),
        SimulationAction::SubmitProposal {
            proposal_type,
            payload,
        } => {
            let proposal_id = format!("proposal:{}", graph.nodes.len() + 1);
            graph.add_node(Node {
                id: proposal_id,
                node_type: "proposal".to_string(),
                attributes: BTreeMap::from([
                    ("proposal_type".to_string(), proposal_type.clone()),
                    ("payload".to_string(), payload.clone()),
                ]),
            });
        }
        SimulationAction::CastVote {
            proposal_id,
            choice,
        } => {
            let vote_node_id = format!("vote:{}:{}", proposal_id, graph.edges.len() + 1);
            graph.add_node(Node {
                id: vote_node_id.clone(),
                node_type: "vote".to_string(),
                attributes: BTreeMap::from([
                    ("proposal_id".to_string(), proposal_id.clone()),
                    ("choice".to_string(), choice.clone()),
                ]),
            });
            graph.add_edge(Edge {
                from: vote_node_id,
                to: proposal_id.clone(),
                kind: EdgeKind::Custom("cast_vote".to_string()),
            });
        }
        SimulationAction::ModifyAttribute {
            node_id,
            key,
            value,
        } => {
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.attributes.insert(key.clone(), value.clone());
            }
        }
        SimulationAction::AgentAction { target: _ } => {
            // Future integration: deserialize the target and map it to specific graph mutations
            // For now, this acts as a declarative placeholder indicating the agent intent exists
            // in the simulation log, without mutating the dummy GSMS graph structure yet.
        }
    }
}

fn parse_edge_kind(raw: &str) -> EdgeKind {
    match raw {
        "depends_on" => EdgeKind::DependsOn,
        "contradicts" => EdgeKind::Contradicts,
        "supersedes" => EdgeKind::Supersedes,
        "implements" => EdgeKind::Implements,
        "invalidates" => EdgeKind::Invalidates,
        "requires" => EdgeKind::Requires,
        "assumes" => EdgeKind::Assumes,
        "constitutional_basis" => EdgeKind::ConstitutionalBasis,
        "derives_from" => EdgeKind::DerivesFrom,
        "forked_into" => EdgeKind::ForkedInto,
        "governs" => EdgeKind::Governs,
        other => EdgeKind::Custom(other.to_string()),
    }
}

fn summarize_violations(violations: &[IntegrityViolation]) -> ViolationSummary {
    let mut summary = ViolationSummary {
        critical: 0,
        violation: 0,
        warning: 0,
        info: 0,
        risk_score: 0,
    };

    for violation in violations {
        match violation.severity {
            Severity::Critical => summary.critical += 1,
            Severity::Violation => summary.violation += 1,
            Severity::Warning => summary.warning += 1,
            Severity::Info => summary.info += 1,
        }
    }
    summary.risk_score = summary.critical * 5 + summary.violation * 3 + summary.warning;
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeKind;
    use crate::integrity::predicate::{
        Constraint, Direction, EdgeSelector, IntegrityPredicate, NodeSelector,
    };
    use crate::integrity::rule::{IntegrityRule, IntegrityScope};

    fn sample_rules() -> Vec<IntegrityRule> {
        vec![IntegrityRule {
            id: "rule-1".to_string(),
            name: "No orphan initiatives".to_string(),
            description: "initiatives require dependency".to_string(),
            scope: IntegrityScope::Global,
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("initiative".to_string()),
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::MinCount(1),
            },
            severity: Severity::Warning,
            remediation_hint: None,
        }]
    }

    #[test]
    fn deterministic_session_replay_is_stable_for_same_seed() {
        let base_graph = Graph::default();
        let scenario = ScenarioDefinition {
            scenario: super::super::scenario::ScenarioMetadata {
                id: "scenario-1".to_string(),
                name: "Determinism".to_string(),
                seed: 42,
                commons_version: "nostra-core-v0".to_string(),
                siqs_version: "1.0.0".to_string(),
            },
            constraints: super::super::scenario::ScenarioConstraints {
                max_mutations: 50,
                max_rounds: 5,
                max_runtime_ms: 500,
            },
            rounds: vec![super::super::scenario::ScenarioRound {
                round: 1,
                actions: vec![super::super::scenario::ScenarioRoundAction {
                    actor: "citizen-1".to_string(),
                    action: "add_node".to_string(),
                    node_id: Some("initiative-1".to_string()),
                    node_type: Some("initiative".to_string()),
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
                }],
            }],
        };

        let first = run_deterministic_session(&base_graph, &sample_rules(), &scenario);
        let second = run_deterministic_session(&base_graph, &sample_rules(), &scenario);
        assert_eq!(first.result, second.result);
    }
}
