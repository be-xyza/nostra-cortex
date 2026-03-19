use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub mod diff;
pub mod traversal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    DependsOn,
    Contradicts,
    Supersedes,
    Implements,
    Invalidates,
    Requires,
    Assumes,
    ConstitutionalBasis,
    DerivesFrom,
    ForkedInto,
    Governs,
    Produces,
    References,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Graph {
    #[serde(default)]
    pub nodes: BTreeMap<String, Node>,
    #[serde(default)]
    pub edges: BTreeSet<Edge>,
}

impl Graph {
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
        self.edges
            .retain(|edge| edge.from != node_id && edge.to != node_id);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn remove_edge(&mut self, edge: &Edge) {
        self.edges.remove(edge);
    }

    pub fn root_hash_hex(&self) -> String {
        let raw = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(raw);
        hex::encode(hasher.finalize())
    }
}
