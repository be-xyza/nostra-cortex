use candid::{CandidType, Int, Principal};
use ic_cdk::api;
use nostra_workflow_core::{Action, Step, WorkflowDefinition};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub schema_ref: Option<String>,
    pub file_ref: Option<String>,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub flows: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub topic: Option<String>,
    pub variant: String,
    pub conditional: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowGraph {
    pub id: String,
    pub workflow_id: String,
    pub version: String,
    pub generated_at: u64,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowNodePosition {
    pub node_id: String,
    pub x: Int,
    pub y: Int,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowHandlePosition {
    pub handle_id: String,
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowLayoutInput {
    pub workflow_id: String,
    pub graph_version: String,
    pub node_positions: Vec<FlowNodePosition>,
    pub handle_positions: Vec<FlowHandlePosition>,
    pub collapsed_groups: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct FlowLayout {
    pub workflow_id: String,
    pub graph_version: String,
    pub node_positions: Vec<FlowNodePosition>,
    pub handle_positions: Vec<FlowHandlePosition>,
    pub collapsed_groups: Vec<String>,
    pub updated_by: Principal,
    pub updated_at: u64,
}

thread_local! {
    static FLOW_LAYOUTS: RefCell<BTreeMap<String, FlowLayout>> = RefCell::new(BTreeMap::new());
    static FLOW_LAYOUT_LATEST: RefCell<BTreeMap<String, String>> = RefCell::new(BTreeMap::new());
}

fn layout_key(workflow_id: &str, graph_version: &str) -> String {
    format!("{}::{}", workflow_id, graph_version)
}

fn action_type(step: &Step) -> String {
    match step.action {
        Action::UserTask { .. } => "user_task",
        Action::SystemOp { .. } => "system_op",
        Action::AsyncExternalOp { .. } => "async_external_op",
        Action::None => "noop",
    }
    .to_string()
}

fn node_tags(step: &Step) -> Vec<String> {
    let mut tags = Vec::new();
    match step.action {
        Action::UserTask { .. } => tags.push("human".to_string()),
        Action::SystemOp { .. } => tags.push("system".to_string()),
        Action::AsyncExternalOp { .. } => tags.push("async".to_string()),
        Action::None => tags.push("noop".to_string()),
    }
    tags
}

fn hash_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn edge_id(source: &str, target: &str, variant: &str) -> String {
    format!("edge:{}", hash_hex(&format!("{}:{}:{}", source, target, variant)))
}

fn graph_version(nodes: &[FlowNode], edges: &[FlowEdge], definition_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(definition_id.as_bytes());
    for node in nodes {
        hasher.update(node.id.as_bytes());
        hasher.update(node.name.as_bytes());
        hasher.update(node.node_type.as_bytes());
    }
    for edge in edges {
        hasher.update(edge.id.as_bytes());
        hasher.update(edge.source.as_bytes());
        hasher.update(edge.target.as_bytes());
        hasher.update(edge.variant.as_bytes());
    }
    hex::encode(hasher.finalize())
}

pub fn derive_graph(definition: &WorkflowDefinition, version_override: Option<String>) -> FlowGraph {
    let mut nodes: Vec<FlowNode> = definition
        .steps
        .values()
        .map(|step| FlowNode {
            id: step.id.clone(),
            name: step.description.clone(),
            node_type: action_type(step),
            schema_ref: None,
            file_ref: None,
            language: None,
            tags: node_tags(step),
            flows: Vec::new(),
        })
        .collect();

    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();
    for step in definition.steps.values() {
        for transition in &step.transitions {
            let source = step.id.clone();
            let target = transition.target_step_id.clone();
            let id = edge_id(&source, &target, "transition");
            if seen.insert(id.clone()) {
                edges.push(FlowEdge {
                    id,
                    source,
                    target,
                    topic: None,
                    variant: "transition".to_string(),
                    conditional: None,
                });
            }
        }
        if let Some(comp) = step.compensated_by.as_ref() {
            let source = step.id.clone();
            let target = comp.clone();
            let id = edge_id(&source, &target, "compensation");
            if seen.insert(id.clone()) {
                edges.push(FlowEdge {
                    id,
                    source,
                    target,
                    topic: None,
                    variant: "compensation".to_string(),
                    conditional: None,
                });
            }
        }
    }

    edges.sort_by(|a, b| a.id.cmp(&b.id));

    let version = version_override.unwrap_or_else(|| graph_version(&nodes, &edges, &definition.id));
    let id = format!("flow_graph:{}:{}", definition.id, version);

    FlowGraph {
        id,
        workflow_id: definition.id.clone(),
        version,
        generated_at: api::time(),
        nodes,
        edges,
    }
}

pub fn get_flow_layout(
    workflow_id: String,
    graph_version: Option<String>,
) -> Result<FlowLayout, String> {
    if workflow_id.trim().is_empty() {
        return Err("workflow_id is required".to_string());
    }

    let version = match graph_version {
        Some(v) if !v.trim().is_empty() => v,
        _ => FLOW_LAYOUT_LATEST.with(|latest| {
            latest
                .borrow()
                .get(&workflow_id)
                .cloned()
                .unwrap_or_default()
        }),
    };

    if version.trim().is_empty() {
        return Err("graph_version is required".to_string());
    }

    let key = layout_key(&workflow_id, &version);
    FLOW_LAYOUTS.with(|layouts| {
        layouts
            .borrow()
            .get(&key)
            .cloned()
            .ok_or_else(|| "flow layout not found".to_string())
    })
}

pub fn set_flow_layout(input: FlowLayoutInput) -> Result<FlowLayout, String> {
    if input.workflow_id.trim().is_empty() {
        return Err("workflow_id is required".to_string());
    }
    if input.graph_version.trim().is_empty() {
        return Err("graph_version is required".to_string());
    }

    let layout = FlowLayout {
        workflow_id: input.workflow_id.clone(),
        graph_version: input.graph_version.clone(),
        node_positions: input.node_positions,
        handle_positions: input.handle_positions,
        collapsed_groups: input.collapsed_groups,
        updated_by: ic_cdk::api::msg_caller(),
        updated_at: api::time(),
    };

    let key = layout_key(&input.workflow_id, &input.graph_version);
    FLOW_LAYOUTS.with(|layouts| {
        layouts.borrow_mut().insert(key, layout.clone());
    });
    FLOW_LAYOUT_LATEST.with(|latest| {
        latest
            .borrow_mut()
            .insert(input.workflow_id, input.graph_version);
    });

    Ok(layout)
}
