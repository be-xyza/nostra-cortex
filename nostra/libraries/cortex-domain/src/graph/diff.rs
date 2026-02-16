use super::Edge;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralDiff {
    pub added_edges: Vec<Edge>,
    pub removed_edges: Vec<Edge>,
}

pub fn structural(before: &[Edge], after: &[Edge]) -> StructuralDiff {
    let before_set: BTreeSet<Edge> = before.iter().cloned().collect();
    let after_set: BTreeSet<Edge> = after.iter().cloned().collect();

    let added_edges = after_set.difference(&before_set).cloned().collect();
    let removed_edges = before_set.difference(&after_set).cloned().collect();

    StructuralDiff {
        added_edges,
        removed_edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeKind;

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
        assert_eq!(result.added_edges.len(), 1);
        assert_eq!(result.removed_edges.len(), 1);
    }
}
