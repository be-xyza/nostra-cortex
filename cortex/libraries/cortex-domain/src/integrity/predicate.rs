use crate::graph::EdgeKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityPredicate {
    pub target: NodeSelector,
    #[serde(default)]
    pub relation: Option<EdgeSelector>,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeSelector {
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EdgeSelector {
    pub edge_kind: EdgeKind,
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Outgoing,
    Incoming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Constraint {
    MustExist,
    MustNotExist,
    MinCount(usize),
    MaxCount(usize),
    NoCycles,
    NoConflicts,
    RequiresConstitutionalReference,
}
