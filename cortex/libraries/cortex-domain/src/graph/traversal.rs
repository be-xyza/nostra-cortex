use super::{EdgeKind, Graph};
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

pub fn dependency_walk(graph: &Graph, node_id: &str, edge_kind: EdgeKind) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut queue = VecDeque::from([node_id.to_string()]);

    while let Some(current) = queue.pop_front() {
        let mut neighbors = graph
            .edges
            .iter()
            .filter(|edge| edge.kind == edge_kind && edge.from == current)
            .map(|edge| edge.to.clone())
            .collect::<Vec<_>>();
        neighbors.sort();

        for neighbor in neighbors {
            if out.insert(neighbor.clone()) {
                queue.push_back(neighbor);
            }
        }
    }

    out
}

pub fn find_orphans(graph: &Graph) -> Vec<String> {
    let mut incoming_counts: BTreeMap<String, usize> =
        graph.nodes.keys().map(|id| (id.clone(), 0usize)).collect();
    for edge in &graph.edges {
        if let Some(count) = incoming_counts.get_mut(&edge.to) {
            *count += 1;
        }
    }

    incoming_counts
        .into_iter()
        .filter_map(|(node_id, incoming)| (incoming == 0).then_some(node_id))
        .collect()
}

pub fn detect_cycles(graph: &Graph, edge_kind: EdgeKind) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut seen = BTreeSet::new();

    for node_id in graph.nodes.keys() {
        if seen.contains(node_id) {
            continue;
        }
        let mut stack = Vec::new();
        dfs_cycle(
            graph,
            node_id,
            &edge_kind,
            &mut stack,
            &mut seen,
            &mut cycles,
        );
    }

    cycles
}

pub fn topological_sort(
    graph: &Graph,
    edge_kind: EdgeKind,
) -> Result<Vec<String>, Vec<Vec<String>>> {
    let cycles = detect_cycles(graph, edge_kind.clone());
    if !cycles.is_empty() {
        return Err(cycles);
    }

    let mut in_degree: BTreeMap<String, usize> =
        graph.nodes.keys().map(|id| (id.clone(), 0usize)).collect();
    let mut outgoing: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for edge in &graph.edges {
        if edge.kind != edge_kind {
            continue;
        }
        *in_degree.entry(edge.to.clone()).or_default() += 1;
        outgoing
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    for edges in outgoing.values_mut() {
        edges.sort();
    }

    let mut ready = in_degree
        .iter()
        .filter_map(|(node_id, degree)| (*degree == 0).then_some(node_id.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut ordered = Vec::new();
    let mut queue = VecDeque::from(ready);

    while let Some(node_id) = queue.pop_front() {
        ordered.push(node_id.clone());
        if let Some(children) = outgoing.get(&node_id) {
            for child in children {
                if let Some(degree) = in_degree.get_mut(child) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        queue.push_back(child.clone());
                    }
                }
            }
        }
    }

    Ok(ordered)
}

fn dfs_cycle(
    graph: &Graph,
    node_id: &str,
    edge_kind: &EdgeKind,
    stack: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(position) = stack.iter().position(|node| node == node_id) {
        cycles.push(stack[position..].to_vec());
        return;
    }

    if !seen.insert(node_id.to_string()) {
        return;
    }

    stack.push(node_id.to_string());
    let mut neighbors = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == *edge_kind && edge.from == node_id)
        .map(|edge| edge.to.clone())
        .collect::<Vec<_>>();
    neighbors.sort();
    for neighbor in neighbors {
        dfs_cycle(graph, &neighbor, edge_kind, stack, seen, cycles);
    }
    stack.pop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Graph, Node};
    use std::collections::{BTreeMap, BTreeSet};

    fn sample_graph() -> Graph {
        let mut graph = Graph::default();
        graph.add_node(Node {
            id: "a".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_node(Node {
            id: "b".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_node(Node {
            id: "c".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph
    }

    #[test]
    fn traversal_order_is_deterministic() {
        let mut adjacency = Adjacency::new();
        adjacency.insert(
            "a".to_string(),
            vec![
                ("c".to_string(), EdgeKind::References),
                ("b".to_string(), EdgeKind::DependsOn),
            ],
        );
        adjacency.insert("b".to_string(), vec![("d".to_string(), EdgeKind::Produces)]);

        let visited = bfs("a", &adjacency);
        assert_eq!(visited, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn dependency_walk_collects_transitive_edges() {
        let graph = sample_graph();
        let deps = dependency_walk(&graph, "a", EdgeKind::DependsOn);
        assert_eq!(deps, BTreeSet::from(["b".to_string(), "c".to_string()]));
    }

    #[test]
    fn detects_cycles_for_edge_kind() {
        let mut graph = sample_graph();
        graph.add_edge(Edge {
            from: "c".to_string(),
            to: "a".to_string(),
            kind: EdgeKind::DependsOn,
        });
        let cycles = detect_cycles(&graph, EdgeKind::DependsOn);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn finds_orphans() {
        let graph = sample_graph();
        let orphans = find_orphans(&graph);
        assert!(orphans.iter().any(|entry| entry == "a"));
    }
}
