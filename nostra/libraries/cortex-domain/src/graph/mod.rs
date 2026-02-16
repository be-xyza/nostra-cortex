use serde::{Deserialize, Serialize};

pub mod diff;
pub mod traversal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    DependsOn,
    Produces,
    References,
    DerivesFrom,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}
