use crate::services::knowledge_graph_query::InMemoryTriple;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const TOPOLOGY_SCHEMA_VERSION: &str = "1.0.0";
const TOPOLOGY_ORDERING_STRATEGY: &str = "canonical";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreTopologyNode {
    pub node_id: String,
    pub label: String,
    pub node_kind: String,
    pub graph_scopes: Vec<String>,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreTopologyEdge {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub predicate: String,
    pub graph_scope: String,
    pub provenance_scope: String,
    pub source_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreTopologyView {
    pub schema_version: String,
    pub space_id: String,
    pub generated_from: String,
    pub ordering_strategy: String,
    pub nodes: Vec<ExploreTopologyNode>,
    pub edges: Vec<ExploreTopologyEdge>,
}

#[derive(Default)]
struct NodeAccumulator {
    label: String,
    node_kind: String,
    graph_scopes: BTreeSet<String>,
    source_refs: BTreeSet<String>,
}

pub fn build_topology_view(
    space_id: &str,
    generated_from: &str,
    triples: &[InMemoryTriple],
) -> ExploreTopologyView {
    let mut node_index: BTreeMap<String, NodeAccumulator> = BTreeMap::new();
    let mut edges: Vec<ExploreTopologyEdge> = Vec::new();

    for triple in triples {
        register_node(&mut node_index, &triple.subject, &triple.graph_scope, &triple.source_ref);
        register_node(&mut node_index, &triple.object, &triple.graph_scope, &triple.source_ref);
        edges.push(ExploreTopologyEdge {
            edge_id: format!(
                "{}|{}|{}|{}|{}",
                triple.graph_scope, triple.subject, triple.predicate, triple.object, triple.source_ref
            ),
            source_node_id: triple.subject.clone(),
            target_node_id: triple.object.clone(),
            predicate: triple.predicate.clone(),
            graph_scope: triple.graph_scope.clone(),
            provenance_scope: triple.provenance_scope.clone(),
            source_ref: triple.source_ref.clone(),
        });
    }

    let nodes = node_index
        .into_iter()
        .map(|(node_id, item)| ExploreTopologyNode {
            node_id,
            label: item.label,
            node_kind: item.node_kind,
            graph_scopes: item.graph_scopes.into_iter().collect(),
            source_refs: item.source_refs.into_iter().collect(),
        })
        .collect();

    edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));

    ExploreTopologyView {
        schema_version: TOPOLOGY_SCHEMA_VERSION.to_string(),
        space_id: space_id.to_string(),
        generated_from: generated_from.to_string(),
        ordering_strategy: TOPOLOGY_ORDERING_STRATEGY.to_string(),
        nodes,
        edges,
    }
}

fn register_node(
    index: &mut BTreeMap<String, NodeAccumulator>,
    node_id: &str,
    graph_scope: &str,
    source_ref: &str,
) {
    let entry = index.entry(node_id.to_string()).or_insert_with(|| NodeAccumulator {
        label: resource_label(node_id),
        node_kind: resource_kind(node_id),
        graph_scopes: BTreeSet::new(),
        source_refs: BTreeSet::new(),
    });
    entry.graph_scopes.insert(graph_scope.to_string());
    entry.source_refs.insert(source_ref.to_string());
}

fn resource_label(resource_ref: &str) -> String {
    resource_ref
        .rsplit('/')
        .next()
        .unwrap_or(resource_ref)
        .to_string()
}

fn resource_kind(resource_ref: &str) -> String {
    let mut parts = resource_ref.split('/');
    let _scheme = parts.next();
    let _empty = parts.next();
    parts.next().unwrap_or("resource").to_string()
}
