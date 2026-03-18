use super::{Edge, Graph};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeChange {
    pub node_id: String,
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralDiff {
    pub nodes_added: Vec<String>,
    pub nodes_removed: Vec<String>,
    pub edges_added: Vec<Edge>,
    pub edges_removed: Vec<Edge>,
    pub attributes_changed: Vec<AttributeChange>,
}

pub fn structural(before: &[Edge], after: &[Edge]) -> StructuralDiff {
    let before_set: BTreeSet<Edge> = before.iter().cloned().collect();
    let after_set: BTreeSet<Edge> = after.iter().cloned().collect();

    let edges_added = after_set.difference(&before_set).cloned().collect();
    let edges_removed = before_set.difference(&after_set).cloned().collect();

    StructuralDiff {
        nodes_added: Vec::new(),
        nodes_removed: Vec::new(),
        edges_added,
        edges_removed,
        attributes_changed: Vec::new(),
    }
}

pub fn structural_graph(before: &Graph, after: &Graph) -> StructuralDiff {
    let before_nodes = before.nodes.keys().cloned().collect::<BTreeSet<_>>();
    let after_nodes = after.nodes.keys().cloned().collect::<BTreeSet<_>>();
    let nodes_added = after_nodes
        .difference(&before_nodes)
        .cloned()
        .collect::<Vec<_>>();
    let nodes_removed = before_nodes
        .difference(&after_nodes)
        .cloned()
        .collect::<Vec<_>>();

    let before_edges = before.edges.iter().cloned().collect::<Vec<_>>();
    let after_edges = after.edges.iter().cloned().collect::<Vec<_>>();
    let edge_diff = structural(&before_edges, &after_edges);

    let mut attributes_changed = Vec::new();
    for node_id in before_nodes.intersection(&after_nodes) {
        let before_node = &before.nodes[node_id];
        let after_node = &after.nodes[node_id];
        let keys = collect_attribute_keys(&before_node.attributes, &after_node.attributes);
        for key in keys {
            let before_value = before_node.attributes.get(&key).cloned();
            let after_value = after_node.attributes.get(&key).cloned();
            if before_value != after_value {
                attributes_changed.push(AttributeChange {
                    node_id: node_id.clone(),
                    key,
                    before: before_value,
                    after: after_value,
                });
            }
        }
    }

    StructuralDiff {
        nodes_added,
        nodes_removed,
        edges_added: edge_diff.edges_added,
        edges_removed: edge_diff.edges_removed,
        attributes_changed,
    }
}

fn collect_attribute_keys(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut keys = before.keys().cloned().collect::<BTreeSet<_>>();
    keys.extend(after.keys().cloned());
    keys.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{EdgeKind, Node};
    use std::collections::BTreeMap;

    #[test]
    fn diff_detects_adds_and_removes() {
        let before = vec![Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            kind: EdgeKind::DependsOn,
        }];

        let after = vec![Edge {
            from: "a".to_string(),
            to: "c".to_string(),
            kind: EdgeKind::Produces,
        }];

        let result = structural(&before, &after);
        assert_eq!(result.edges_added.len(), 1);
        assert_eq!(result.edges_removed.len(), 1);
    }

    #[test]
    fn graph_diff_detects_node_and_attribute_changes() {
        let mut before = Graph::default();
        before.add_node(Node {
            id: "n1".to_string(),
            node_type: "space".to_string(),
            attributes: BTreeMap::from([("phase".to_string(), "draft".to_string())]),
        });

        let mut after = before.clone();
        after.add_node(Node {
            id: "n2".to_string(),
            node_type: "proposal".to_string(),
            attributes: BTreeMap::new(),
        });
        if let Some(node) = after.nodes.get_mut("n1") {
            node.attributes
                .insert("phase".to_string(), "operational".to_string());
        }

        let result = structural_graph(&before, &after);
        assert_eq!(result.nodes_added, vec!["n2".to_string()]);
        assert_eq!(result.attributes_changed.len(), 1);
    }
}
