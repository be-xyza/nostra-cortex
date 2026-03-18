use super::predicate::{Constraint, Direction, EdgeSelector, NodeSelector};
use super::rule::{IntegrityRule, IntegrityScope, Severity};
use crate::graph::traversal::detect_cycles;
use crate::graph::{EdgeKind, Graph, Node};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityViolation {
    pub rule_id: String,
    pub affected_nodes: Vec<String>,
    pub severity: Severity,
    pub explanation: String,
}

pub fn evaluate_rule(rule: &IntegrityRule, graph: &Graph) -> Vec<IntegrityViolation> {
    let candidate_nodes = select_nodes(graph, &rule.scope, &rule.predicate.target);

    match &rule.predicate.constraint {
        Constraint::NoCycles => {
            let edge_kind = rule
                .predicate
                .relation
                .as_ref()
                .map(|relation| relation.edge_kind.clone())
                .unwrap_or(EdgeKind::DependsOn);
            let cycles = detect_cycles(graph, edge_kind);
            if cycles.is_empty() {
                Vec::new()
            } else {
                cycles
                    .into_iter()
                    .map(|cycle| IntegrityViolation {
                        rule_id: rule.id.clone(),
                        affected_nodes: cycle.clone(),
                        severity: rule.severity.clone(),
                        explanation: format!(
                            "Cycle detected for rule '{}': {}",
                            rule.name,
                            cycle.join(" -> ")
                        ),
                    })
                    .collect()
            }
        }
        _ => candidate_nodes
            .iter()
            .filter_map(|node| evaluate_node(rule, graph, node))
            .collect(),
    }
}

pub fn evaluate_all(rules: &[IntegrityRule], graph: &Graph) -> Vec<IntegrityViolation> {
    let mut out = Vec::new();
    for rule in rules {
        out.extend(evaluate_rule(rule, graph));
    }
    out
}

fn evaluate_node(rule: &IntegrityRule, graph: &Graph, node: &Node) -> Option<IntegrityViolation> {
    let relation_count = count_relations(graph, node.id.as_str(), rule.predicate.relation.as_ref());

    let violated = match rule.predicate.constraint {
        Constraint::MustExist => relation_count == 0,
        Constraint::MustNotExist => relation_count > 0,
        Constraint::MinCount(min) => relation_count < min,
        Constraint::MaxCount(max) => relation_count > max,
        Constraint::NoConflicts => {
            count_relations(
                graph,
                node.id.as_str(),
                Some(&EdgeSelector {
                    edge_kind: EdgeKind::Contradicts,
                    direction: Direction::Outgoing,
                }),
            ) > 0
        }
        Constraint::RequiresConstitutionalReference => {
            count_relations(
                graph,
                node.id.as_str(),
                Some(&EdgeSelector {
                    edge_kind: EdgeKind::ConstitutionalBasis,
                    direction: Direction::Outgoing,
                }),
            ) == 0
        }
        Constraint::NoCycles => false,
    };

    violated.then_some(IntegrityViolation {
        rule_id: rule.id.clone(),
        affected_nodes: vec![node.id.clone()],
        severity: rule.severity.clone(),
        explanation: format!(
            "Rule '{}' violated by node '{}' with relation count {}",
            rule.name, node.id, relation_count
        ),
    })
}

fn select_nodes<'a>(
    graph: &'a Graph,
    scope: &IntegrityScope,
    selector: &NodeSelector,
) -> Vec<&'a Node> {
    let scope_space = match scope {
        IntegrityScope::Space(space_id) => Some(space_id.as_str()),
        _ => None,
    };

    let mut nodes = graph
        .nodes
        .values()
        .filter(|node| {
            if let Some(entity_type) = selector.entity_type.as_ref() {
                if &node.node_type != entity_type {
                    return false;
                }
            }

            if let Some(space_id) = scope_space {
                let node_space = node.attributes.get("space_id").map(String::as_str);
                if node_space != Some(space_id) {
                    return false;
                }
            }

            if let Some(tags) = selector.tags.as_ref() {
                if tags.is_empty() {
                    return true;
                }
                let node_tags = node
                    .attributes
                    .get("tags")
                    .map(|raw| {
                        raw.split(',')
                            .map(str::trim)
                            .filter(|entry| !entry.is_empty())
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_default();
                return tags.iter().all(|tag| node_tags.contains(tag.as_str()));
            }

            true
        })
        .collect::<Vec<_>>();

    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes
}

fn count_relations(graph: &Graph, node_id: &str, relation: Option<&EdgeSelector>) -> usize {
    let Some(relation) = relation else {
        return 0;
    };

    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == relation.edge_kind)
        .filter(|edge| match relation.direction {
            Direction::Outgoing => edge.from == node_id,
            Direction::Incoming => edge.to == node_id,
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Graph, Node};
    use std::collections::BTreeMap;

    fn sample_graph() -> Graph {
        let mut graph = Graph::default();
        graph.add_node(Node {
            id: "i1".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_node(Node {
            id: "i2".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: "i1".to_string(),
            to: "i2".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph
    }

    #[test]
    fn min_count_rule_flags_missing_dependencies() {
        let graph = sample_graph();
        let rule = IntegrityRule {
            id: "r1".to_string(),
            name: "No orphans".to_string(),
            description: "Every initiative depends_on at least one node".to_string(),
            scope: IntegrityScope::EntityType("initiative".to_string()),
            predicate: super::super::predicate::IntegrityPredicate {
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
        };

        let violations = evaluate_rule(&rule, &graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].affected_nodes, vec!["i2".to_string()]);
    }

    #[test]
    fn no_cycles_rule_finds_cycles() {
        let mut graph = sample_graph();
        graph.add_edge(Edge {
            from: "i2".to_string(),
            to: "i1".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let rule = IntegrityRule {
            id: "r2".to_string(),
            name: "No cycles".to_string(),
            description: "No dependency cycles".to_string(),
            scope: IntegrityScope::Global,
            predicate: super::super::predicate::IntegrityPredicate {
                target: NodeSelector {
                    entity_type: None,
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::NoCycles,
            },
            severity: Severity::Violation,
            remediation_hint: None,
        };

        let violations = evaluate_rule(&rule, &graph);
        assert!(!violations.is_empty());
    }
}
