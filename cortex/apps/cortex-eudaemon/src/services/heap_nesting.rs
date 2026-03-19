use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NestingProjectionRecord {
    pub artifact_id: String,
    pub block_id: String,
    pub title: String,
    pub block_type: String,
    pub updated_at: String,
    #[serde(default)]
    pub emitted_at: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mentions_inline: Vec<String>,
    #[serde(default)]
    pub page_links: Vec<String>,
    #[serde(default)]
    pub attributes: Option<std::collections::BTreeMap<String, String>>,
    pub surface_json: Value,
}

pub fn build_nested_a2ui_tree_by_artifact(
    records: &[NestingProjectionRecord],
    max_depth: usize,
) -> HashMap<String, Value> {
    if max_depth == 0 || records.is_empty() {
        return HashMap::new();
    }

    let mut children_by_parent_block_id: HashMap<String, Vec<&NestingProjectionRecord>> =
        HashMap::new();
    for candidate in records {
        for parent_block_id in candidate.tags.iter().chain(candidate.page_links.iter()) {
            children_by_parent_block_id
                .entry(parent_block_id.clone())
                .or_default()
                .push(candidate);
        }
    }

    for children in children_by_parent_block_id.values_mut() {
        children.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.artifact_id.cmp(&left.artifact_id))
        });
    }

    let mut result = HashMap::new();
    for record in records {
        let mut visited = HashSet::new();
        visited.insert(record.artifact_id.clone());
        let nested_children = build_children_nodes(
            &record.block_id,
            &children_by_parent_block_id,
            max_depth,
            &mut visited,
        );
        if nested_children.is_empty() {
            continue;
        }

        result.insert(
            record.artifact_id.clone(),
            json!({
                "id": format!("nested_children:{}", record.artifact_id),
                "type": "Column",
                "componentProperties": {
                    "Column": {
                        "title": "Nested Blocks"
                    }
                },
                "children": {
                    "explicitList": nested_children
                }
            }),
        );
    }

    result
}

fn build_children_nodes(
    parent_block_id: &str,
    children_by_parent_block_id: &HashMap<String, Vec<&NestingProjectionRecord>>,
    depth_remaining: usize,
    visited: &mut HashSet<String>,
) -> Vec<Value> {
    if depth_remaining == 0 {
        return Vec::new();
    }

    let Some(children) = children_by_parent_block_id.get(parent_block_id) else {
        return Vec::new();
    };

    let mut nodes = Vec::new();
    for child in children {
        if !visited.insert(child.artifact_id.clone()) {
            continue;
        }

        let nested_children = build_children_nodes(
            &child.block_id,
            children_by_parent_block_id,
            depth_remaining.saturating_sub(1),
            visited,
        );

        let mut node = json!({
            "id": format!("nested_block:{}", child.artifact_id),
            "type": "HeapBlockCard",
            "componentProperties": {
                "HeapBlockCard": {
                    "artifactId": child.artifact_id,
                    "title": child.title,
                    "blockType": child.block_type,
                    "timestamp": child
                        .emitted_at
                        .clone()
                        .unwrap_or_else(|| child.updated_at.clone()),
                    "tags": child.tags,
                    "mentions": child.mentions_inline,
                    "pageLinks": child.page_links,
                    "attributes": child.attributes,
                    "surfaceJson": child.surface_json,
                }
            }
        });
        if !nested_children.is_empty() {
            node["children"] = json!({ "explicitList": nested_children });
        }

        nodes.push(node);
        visited.remove(&child.artifact_id);
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(
        artifact_id: &str,
        block_id: &str,
        tags: Vec<&str>,
        page_links: Vec<&str>,
        updated_at: &str,
    ) -> NestingProjectionRecord {
        NestingProjectionRecord {
            artifact_id: artifact_id.to_string(),
            block_id: block_id.to_string(),
            title: format!("Block {artifact_id}"),
            block_type: "note".to_string(),
            updated_at: updated_at.to_string(),
            emitted_at: Some(updated_at.to_string()),
            tags: tags.into_iter().map(str::to_string).collect(),
            mentions_inline: vec![],
            page_links: page_links.into_iter().map(str::to_string).collect(),
            attributes: None,
            surface_json: json!({ "payload_type": "rich_text", "text": artifact_id }),
        }
    }

    #[test]
    fn nesting_projection_builds_tree_with_cycle_protection() {
        let parent = sample_record("a", "A", vec![], vec![], "2026-03-01T10:00:00Z");
        let child = sample_record("b", "B", vec!["A"], vec![], "2026-03-01T10:01:00Z");
        let cycle = sample_record("c", "A", vec!["B"], vec![], "2026-03-01T10:02:00Z");

        let trees = build_nested_a2ui_tree_by_artifact(&[parent, child, cycle], 3);
        let parent_tree = trees
            .get("a")
            .expect("parent should receive nested tree from linked child");
        let explicit_list = parent_tree
            .get("children")
            .and_then(|children| children.get("explicitList"))
            .and_then(Value::as_array)
            .expect("parent nested explicit list should exist");
        assert_eq!(explicit_list.len(), 1);

        let child_node = &explicit_list[0];
        assert_eq!(
            child_node.get("type").and_then(Value::as_str),
            Some("HeapBlockCard")
        );
        let child_children = child_node
            .get("children")
            .and_then(|children| children.get("explicitList"))
            .and_then(Value::as_array)
            .expect("child should include second level node");
        assert_eq!(child_children.len(), 1);

        let second_level_children = child_children[0]
            .get("children")
            .and_then(|children| children.get("explicitList"))
            .and_then(Value::as_array)
            .map(|items| items.len())
            .unwrap_or(0);
        assert_eq!(second_level_children, 0);
    }
}
