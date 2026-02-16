use super::EdgeKind;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type Adjacency = BTreeMap<String, Vec<(String, EdgeKind)>>;

pub fn bfs(start: &str, adjacency: &Adjacency) -> Vec<String> {
    let mut queue = VecDeque::new();
    let mut visited = BTreeSet::new();
    let mut out = Vec::new();

    queue.push_back(start.to_string());

    while let Some(node) = queue.pop_front() {
        if !visited.insert(node.clone()) {
            continue;
        }
        out.push(node.clone());

        let mut neighbors = adjacency.get(&node).cloned().unwrap_or_default();
        neighbors.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        for (neighbor, _) in neighbors {
            if !visited.contains(&neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traversal_order_is_deterministic() {
        let mut graph = Adjacency::new();
        graph.insert(
            "a".to_string(),
            vec![
                ("c".to_string(), EdgeKind::References),
                ("b".to_string(), EdgeKind::DependsOn),
            ],
        );
        graph.insert("b".to_string(), vec![("d".to_string(), EdgeKind::Produces)]);

        let visited = bfs("a", &graph);
        assert_eq!(visited, vec!["a", "b", "c", "d"]);
    }
}
