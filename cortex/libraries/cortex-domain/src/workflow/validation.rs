use crate::workflow::types::{
    WorkflowCompileResult, WorkflowDefinitionV1, WorkflowDraftV1, WorkflowEdgeV1,
    WorkflowMotifKind, WorkflowNodeKind, WorkflowNodeV1, WorkflowScope, WorkflowValidationIssue,
    WorkflowValidationResult, WORKFLOW_CONTEXT_SECTIONS,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn issue(
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> WorkflowValidationIssue {
    WorkflowValidationIssue {
        code: code.to_string(),
        path: path.into(),
        message: message.into(),
    }
}

fn context_section(value: &str) -> Option<&str> {
    value.split('.').next()
}

fn validate_context_paths(
    values: &[String],
    path_prefix: &str,
    allowed_sections: &BTreeSet<String>,
    errors: &mut Vec<WorkflowValidationIssue>,
) {
    for (idx, value) in values.iter().enumerate() {
        let Some(section) = context_section(value) else {
            errors.push(issue(
                "invalid_context_path",
                format!("{path_prefix}[{idx}]"),
                "context path must include a section prefix",
            ));
            continue;
        };
        if !allowed_sections.contains(section) {
            errors.push(issue(
                "undeclared_context_section",
                format!("{path_prefix}[{idx}]"),
                format!("context section '{section}' is not declared in the contract"),
            ));
        }
    }
}

fn validate_node(
    node: &WorkflowNodeV1,
    idx: usize,
    allowed_sections: &BTreeSet<String>,
    errors: &mut Vec<WorkflowValidationIssue>,
) {
    let path = format!("graph.nodes[{idx}]");
    if is_blank(&node.node_id) {
        errors.push(issue(
            "missing_field",
            format!("{path}.nodeId"),
            "nodeId is required",
        ));
    }
    if is_blank(&node.label) {
        errors.push(issue(
            "missing_field",
            format!("{path}.label"),
            "label is required",
        ));
    }

    validate_context_paths(
        &node.reads,
        &format!("{path}.reads"),
        allowed_sections,
        errors,
    );
    validate_context_paths(
        &node.writes,
        &format!("{path}.writes"),
        allowed_sections,
        errors,
    );

    match node.kind {
        WorkflowNodeKind::CapabilityCall => {
            if node.authority_requirements.is_empty() {
                errors.push(issue(
                    "missing_authority_requirement",
                    format!("{path}.authorityRequirements"),
                    "capability_call nodes require at least one authority requirement",
                ));
            }
        }
        WorkflowNodeKind::HumanCheckpoint => match node.checkpoint_policy.as_ref() {
            Some(policy) if policy.resume_allowed && policy.cancel_allowed => {}
            Some(_) => errors.push(issue(
                "invalid_checkpoint_policy",
                format!("{path}.checkpointPolicy"),
                "human_checkpoint nodes must allow both resume and cancel in v1",
            )),
            None => errors.push(issue(
                "missing_checkpoint_policy",
                format!("{path}.checkpointPolicy"),
                "human_checkpoint nodes require checkpointPolicy",
            )),
        },
        WorkflowNodeKind::Loop => match node.loop_policy.as_ref() {
            Some(policy)
                if policy.max_iterations.unwrap_or_default() > 0
                    && policy
                        .termination_expression
                        .as_ref()
                        .map(|value| !is_blank(value))
                        .unwrap_or(false) => {}
            Some(_) => errors.push(issue(
                "unbounded_loop",
                format!("{path}.loopPolicy"),
                "loop nodes require maxIterations and terminationExpression",
            )),
            None => errors.push(issue(
                "missing_loop_policy",
                format!("{path}.loopPolicy"),
                "loop nodes require loopPolicy",
            )),
        },
        WorkflowNodeKind::SubflowRef => {
            if node
                .subflow_ref
                .as_ref()
                .map(|value| is_blank(value))
                .unwrap_or(true)
            {
                errors.push(issue(
                    "missing_subflow_ref",
                    format!("{path}.subflowRef"),
                    "subflow_ref nodes require subflowRef",
                ));
            }
        }
        _ => {}
    }
}

fn detect_non_loop_cycles(
    nodes: &[WorkflowNodeV1],
    edges: &[WorkflowEdgeV1],
) -> Option<Vec<String>> {
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in nodes {
        adjacency.entry(node.node_id.clone()).or_default();
    }
    for edge in edges {
        if edge.relation == "loop_back" {
            continue;
        }
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Visited,
    }

    fn visit(
        node: &str,
        adjacency: &BTreeMap<String, Vec<String>>,
        marks: &mut BTreeMap<String, Mark>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        marks.insert(node.to_string(), Mark::Visiting);
        stack.push(node.to_string());

        if let Some(neighbors) = adjacency.get(node) {
            for next in neighbors {
                match marks.get(next) {
                    Some(Mark::Visiting) => {
                        let mut cycle = stack.clone();
                        cycle.push(next.clone());
                        return Some(cycle);
                    }
                    Some(Mark::Visited) => continue,
                    None => {
                        if let Some(cycle) = visit(next, adjacency, marks, stack) {
                            return Some(cycle);
                        }
                    }
                }
            }
        }

        stack.pop();
        marks.insert(node.to_string(), Mark::Visited);
        None
    }

    let mut marks = BTreeMap::new();
    let mut stack = Vec::new();
    for node in nodes {
        if marks.contains_key(&node.node_id) {
            continue;
        }
        if let Some(cycle) = visit(&node.node_id, &adjacency, &mut marks, &mut stack) {
            return Some(cycle);
        }
    }
    None
}

fn validate_motif_rules(draft: &WorkflowDraftV1, errors: &mut Vec<WorkflowValidationIssue>) {
    let node_count = |kind: WorkflowNodeKind| {
        draft
            .graph
            .nodes
            .iter()
            .filter(|node| node.kind == kind)
            .count()
    };

    match draft.motif_kind {
        WorkflowMotifKind::Sequential => {
            if node_count(WorkflowNodeKind::CapabilityCall) < 1
                || node_count(WorkflowNodeKind::Terminal) != 1
            {
                errors.push(issue(
                    "invalid_motif",
                    "motifKind",
                    "sequential requires at least one capability_call and exactly one terminal",
                ));
            }
        }
        WorkflowMotifKind::ParallelCompare => {
            if node_count(WorkflowNodeKind::Parallel) != 1
                || node_count(WorkflowNodeKind::CapabilityCall) < 2
                || node_count(WorkflowNodeKind::EvaluationGate) != 1
            {
                errors.push(issue(
                    "invalid_motif",
                    "motifKind",
                    "parallel_compare requires one parallel node, at least two capability_call nodes, and exactly one evaluation_gate",
                ));
            }
        }
        WorkflowMotifKind::RepairLoop => {
            if node_count(WorkflowNodeKind::Loop) != 1
                || node_count(WorkflowNodeKind::EvaluationGate) != 1
                || !draft
                    .graph
                    .edges
                    .iter()
                    .any(|edge| edge.relation == "loop_back")
            {
                errors.push(issue(
                    "invalid_motif",
                    "motifKind",
                    "repair_loop requires one loop node, one evaluation_gate, and a loop_back edge",
                ));
            }
        }
        WorkflowMotifKind::FanOutJoin => {
            if node_count(WorkflowNodeKind::Parallel) != 1
                || node_count(WorkflowNodeKind::CapabilityCall) < 2
            {
                errors.push(issue(
                    "invalid_motif",
                    "motifKind",
                    "fan_out_join requires one parallel node and at least two capability_call nodes",
                ));
            }
        }
        WorkflowMotifKind::HumanGate => {
            if node_count(WorkflowNodeKind::HumanCheckpoint) != 1 {
                errors.push(issue(
                    "invalid_motif",
                    "motifKind",
                    "human_gate requires exactly one human_checkpoint node",
                ));
            }
        }
    }
}

pub fn validate_workflow_draft(draft: &WorkflowDraftV1) -> WorkflowValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if is_blank(&draft.workflow_draft_id) {
        errors.push(issue(
            "missing_field",
            "workflowDraftId",
            "workflowDraftId is required",
        ));
    }
    if is_blank(&draft.intent) {
        errors.push(issue("missing_field", "intent", "intent is required"));
    }
    if !(0.0..=1.0).contains(&draft.confidence.score) {
        errors.push(issue(
            "invalid_confidence",
            "confidence.score",
            "confidence score must be between 0.0 and 1.0",
        ));
    }

    let allowed_sections = draft
        .context_contract
        .allowed_sections
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for (idx, node) in draft.graph.nodes.iter().enumerate() {
        validate_node(node, idx, &allowed_sections, &mut errors);
    }

    let node_ids = draft
        .graph
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect::<BTreeSet<_>>();
    for (idx, edge) in draft.graph.edges.iter().enumerate() {
        let path = format!("graph.edges[{idx}]");
        if is_blank(&edge.edge_id) {
            errors.push(issue(
                "missing_field",
                format!("{path}.edgeId"),
                "edgeId is required",
            ));
        }
        if !node_ids.contains(edge.from.as_str()) {
            errors.push(issue(
                "unknown_edge_node",
                format!("{path}.from"),
                format!("edge source '{}' does not exist", edge.from),
            ));
        }
        if !node_ids.contains(edge.to.as_str()) {
            errors.push(issue(
                "unknown_edge_node",
                format!("{path}.to"),
                format!("edge target '{}' does not exist", edge.to),
            ));
        }
    }

    if let Some(cycle) = detect_non_loop_cycles(&draft.graph.nodes, &draft.graph.edges) {
        errors.push(issue(
            "cycle_outside_loop",
            "graph.edges",
            format!("non-loop cycle detected: {}", cycle.join(" -> ")),
        ));
    }

    validate_motif_rules(draft, &mut errors);

    for section in WORKFLOW_CONTEXT_SECTIONS {
        if !allowed_sections.contains(*section) {
            warnings.push(issue(
                "missing_context_section",
                "contextContract.allowedSections",
                format!("recommended context section '{section}' is not declared"),
            ));
        }
    }

    WorkflowValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

pub fn validate_workflow_definition(definition: &WorkflowDefinitionV1) -> WorkflowValidationResult {
    validate_workflow_draft(&WorkflowDraftV1 {
        schema_version: definition.schema_version.clone(),
        workflow_draft_id: definition.definition_id.clone(),
        scope: definition.scope.clone(),
        intent_ref: definition.intent_ref.clone(),
        intent: definition.intent.clone(),
        motif_kind: definition.motif_kind.clone(),
        constraints: definition.constraints.clone(),
        graph: definition.graph.clone(),
        context_contract: definition.context_contract.clone(),
        confidence: definition.confidence.clone(),
        lineage: definition.lineage.clone(),
        policy: definition.policy.clone(),
        provenance: definition.provenance.clone(),
    })
}

fn sort_nodes(nodes: &[WorkflowNodeV1]) -> Vec<&WorkflowNodeV1> {
    let mut sorted = nodes.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    sorted
}

fn sort_edges(edges: &[WorkflowEdgeV1]) -> Vec<&WorkflowEdgeV1> {
    let mut sorted = edges.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    sorted
}

fn build_normalized_graph(draft: &WorkflowDraftV1) -> Value {
    let nodes = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .map(|node| {
            json!({
                "nodeId": node.node_id,
                "label": node.label,
                "kind": node.kind,
                "reads": node.reads,
                "writes": node.writes,
                "evidenceOutputs": node.evidence_outputs,
                "authorityRequirements": node.authority_requirements,
                "checkpointPolicy": node.checkpoint_policy,
                "loopPolicy": node.loop_policy,
                "subflowRef": node.subflow_ref,
                "config": node.config,
            })
        })
        .collect::<Vec<_>>();
    let edges = sort_edges(&draft.graph.edges)
        .into_iter()
        .map(|edge| {
            json!({
                "edgeId": edge.edge_id,
                "from": edge.from,
                "to": edge.to,
                "relation": edge.relation,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "workflowDraftId": draft.workflow_draft_id,
        "motifKind": draft.motif_kind,
        "contextContract": draft.context_contract,
        "nodes": nodes,
        "edges": edges,
    })
}

fn build_a2ui_projection(draft: &WorkflowDraftV1) -> Value {
    let surfaces = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .filter(|node| {
            matches!(
                node.kind,
                WorkflowNodeKind::HumanCheckpoint | WorkflowNodeKind::EvaluationGate
            )
        })
        .map(|node| {
            json!({
                "surfaceId": format!("workflow:{}:{}", draft.workflow_draft_id, node.node_id),
                "nodeId": node.node_id,
                "kind": node.kind,
                "title": node.label,
                "checkpointPolicy": node.checkpoint_policy,
                "meta": {
                    "workflowDraftId": draft.workflow_draft_id,
                    "motifKind": draft.motif_kind,
                },
            })
        })
        .collect::<Vec<_>>();
    json!({ "surfaces": surfaces })
}

fn build_flow_graph_projection(draft: &WorkflowDraftV1) -> Value {
    let nodes = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .map(|node| {
            let tags = match node.kind {
                WorkflowNodeKind::CapabilityCall => vec!["capability", "async"],
                WorkflowNodeKind::HumanCheckpoint => vec!["human", "checkpoint"],
                WorkflowNodeKind::EvaluationGate => vec!["evaluation"],
                WorkflowNodeKind::Parallel => vec!["parallel"],
                WorkflowNodeKind::Switch => vec!["switch"],
                WorkflowNodeKind::Loop => vec!["loop"],
                WorkflowNodeKind::SubflowRef => vec!["subflow"],
                WorkflowNodeKind::Terminal => vec!["terminal"],
            };
            json!({
                "id": node.node_id,
                "name": node.label,
                "type": node.kind,
                "tags": tags,
            })
        })
        .collect::<Vec<_>>();
    let edges = sort_edges(&draft.graph.edges)
        .into_iter()
        .map(|edge| {
            json!({
                "id": edge.edge_id,
                "source": edge.from,
                "target": edge.to,
                "variant": edge.relation,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": format!("flow_graph:{}", draft.workflow_draft_id),
        "workflowDraftId": draft.workflow_draft_id,
        "nodes": nodes,
        "edges": edges,
    })
}

fn build_generic_states(draft: &WorkflowDraftV1) -> Vec<Value> {
    let sorted_nodes = sort_nodes(&draft.graph.nodes);
    let outgoing = sort_edges(&draft.graph.edges).into_iter().fold(
        BTreeMap::<String, Vec<&WorkflowEdgeV1>>::new(),
        |mut acc, edge| {
            acc.entry(edge.from.clone()).or_default().push(edge);
            acc
        },
    );

    sorted_nodes
        .into_iter()
        .map(|node| {
            let transitions = outgoing
                .get(&node.node_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|edge| edge.to.clone())
                .collect::<Vec<_>>();
            let state_type = match node.kind {
                WorkflowNodeKind::CapabilityCall => "operation",
                WorkflowNodeKind::HumanCheckpoint => "event",
                WorkflowNodeKind::EvaluationGate => "operation",
                WorkflowNodeKind::Parallel => "parallel",
                WorkflowNodeKind::Switch => "switch",
                WorkflowNodeKind::Loop => "operation",
                WorkflowNodeKind::SubflowRef => "operation",
                WorkflowNodeKind::Terminal => "operation",
            };
            json!({
                "name": node.node_id,
                "type": state_type,
                "transition": transitions.first().cloned(),
                "end": matches!(node.kind, WorkflowNodeKind::Terminal),
                "metadata": {
                    "kind": node.kind,
                    "label": node.label,
                    "reads": node.reads,
                    "writes": node.writes,
                }
            })
        })
        .collect()
}

fn build_parallel_projection(draft: &WorkflowDraftV1) -> Value {
    let parallel_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::Parallel)
        .expect("validated parallel node");
    let branch_nodes = sort_edges(&draft.graph.edges)
        .into_iter()
        .filter(|edge| edge.from == parallel_node.node_id && edge.relation == "parallel_branch")
        .map(|edge| edge.to.clone())
        .collect::<Vec<_>>();
    let eval_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::EvaluationGate)
        .map(|node| node.node_id.clone());

    json!({
        "id": draft.workflow_draft_id,
        "specVersion": "0.8",
        "name": draft.intent,
        "start": parallel_node.node_id,
        "states": [
            {
                "name": parallel_node.node_id,
                "type": "parallel",
                "branches": branch_nodes.iter().map(|branch| {
                    json!({
                        "name": branch,
                        "actions": [
                            {
                                "functionRef": {
                                    "refName": branch,
                                }
                            }
                        ]
                    })
                }).collect::<Vec<_>>(),
                "transition": eval_node,
                "metadata": {
                    "motifKind": draft.motif_kind,
                    "preserveAllBranchOutputs": true,
                }
            },
            {
                "name": eval_node,
                "type": "operation",
                "actions": [
                    {
                        "functionRef": {
                            "refName": "evaluation_gate",
                        }
                    }
                ],
                "end": true
            }
        ]
    })
}

fn build_repair_loop_projection(draft: &WorkflowDraftV1) -> Value {
    let repair_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::CapabilityCall)
        .expect("validated capability");
    let eval_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::EvaluationGate)
        .expect("validated evaluation");
    let loop_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::Loop)
        .expect("validated loop");
    let terminal_node = sort_nodes(&draft.graph.nodes)
        .into_iter()
        .find(|node| node.kind == WorkflowNodeKind::Terminal)
        .expect("validated terminal");
    let max_iterations = loop_node
        .loop_policy
        .as_ref()
        .and_then(|policy| policy.max_iterations)
        .unwrap_or(1);

    json!({
        "id": draft.workflow_draft_id,
        "specVersion": "0.8",
        "name": draft.intent,
        "start": repair_node.node_id,
        "states": [
            {
                "name": repair_node.node_id,
                "type": "operation",
                "actions": [{ "functionRef": { "refName": "repair_capability" } }],
                "transition": eval_node.node_id,
            },
            {
                "name": eval_node.node_id,
                "type": "switch",
                "dataConditions": [
                    {
                        "condition": "${ .evaluation.pass == true }",
                        "transition": terminal_node.node_id,
                    }
                ],
                "defaultCondition": {
                    "transition": repair_node.node_id,
                },
                "metadata": {
                    "loopNode": loop_node.node_id,
                    "maxIterations": max_iterations,
                }
            },
            {
                "name": terminal_node.node_id,
                "type": "operation",
                "end": true
            }
        ]
    })
}

fn build_serverless_projection(draft: &WorkflowDraftV1) -> Value {
    match draft.motif_kind {
        WorkflowMotifKind::ParallelCompare | WorkflowMotifKind::FanOutJoin => {
            build_parallel_projection(draft)
        }
        WorkflowMotifKind::RepairLoop => build_repair_loop_projection(draft),
        _ => {
            let start = sort_nodes(&draft.graph.nodes)
                .first()
                .map(|node| node.node_id.clone())
                .unwrap_or_default();
            json!({
                "id": draft.workflow_draft_id,
                "specVersion": "0.8",
                "name": draft.intent,
                "start": start,
                "states": build_generic_states(draft),
            })
        }
    }
}

pub fn workflow_digest_hex(value: &Value) -> String {
    let mut hasher = Sha256::new();
    if let Ok(bytes) = serde_json::to_vec(value) {
        hasher.update(bytes);
    }
    hex::encode(hasher.finalize())
}

pub fn compile_workflow_draft(
    draft: &WorkflowDraftV1,
) -> Result<WorkflowCompileResult, WorkflowValidationResult> {
    let validation = validate_workflow_draft(draft);
    if !validation.valid {
        return Err(validation);
    }

    let normalized_graph = build_normalized_graph(draft);
    let serverless_workflow_projection = build_serverless_projection(draft);
    let a2ui_surface_projection = build_a2ui_projection(draft);
    let flow_graph_projection = build_flow_graph_projection(draft);
    let digest = workflow_digest_hex(&json!({
        "normalizedGraph": normalized_graph,
        "serverlessWorkflowProjection": serverless_workflow_projection,
        "a2uiSurfaceProjection": a2ui_surface_projection,
        "flowGraphProjection": flow_graph_projection,
    }));

    Ok(WorkflowCompileResult {
        valid: true,
        normalized_graph,
        serverless_workflow_projection,
        a2ui_surface_projection,
        flow_graph_projection,
        warnings: validation.warnings,
        digest,
    })
}

pub fn scope_key(scope: &WorkflowScope) -> String {
    let mut parts = Vec::new();
    if let Some(space_id) = &scope.space_id {
        parts.push(format!("space-{}", sanitize_scope_token(space_id)));
    }
    if let Some(route_id) = &scope.route_id {
        parts.push(format!("route-{}", sanitize_scope_token(route_id)));
    }
    if let Some(role) = &scope.role {
        parts.push(format!("role-{}", sanitize_scope_token(role)));
    }

    if parts.is_empty() {
        "global".to_string()
    } else {
        parts.join("__")
    }
}

fn sanitize_scope_token(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::synthesis::generate_candidate_drafts;

    fn repair_loop_draft() -> WorkflowDraftV1 {
        generate_candidate_drafts(
            WorkflowScope::default(),
            "repair loop",
            WorkflowMotifKind::RepairLoop,
            &[],
            1,
            "tester",
            "human",
            "2026-03-11T00:00:00Z",
            "seed",
        )
        .remove(0)
    }

    #[test]
    fn rejects_non_loop_cycles() {
        let mut draft = repair_loop_draft();
        draft.graph.edges.push(WorkflowEdgeV1 {
            edge_id: "cycle".to_string(),
            from: draft.graph.nodes[1].node_id.clone(),
            to: draft.graph.nodes[0].node_id.clone(),
            relation: "transition".to_string(),
        });
        let result = validate_workflow_draft(&draft);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "cycle_outside_loop"));
    }

    #[test]
    fn repair_loop_rejects_missing_max_iterations() {
        let mut draft = repair_loop_draft();
        let loop_node = draft
            .graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == WorkflowNodeKind::Loop)
            .unwrap();
        loop_node.loop_policy.as_mut().unwrap().max_iterations = None;
        let result = validate_workflow_draft(&draft);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "unbounded_loop"));
    }

    #[test]
    fn human_checkpoint_requires_resume_and_cancel() {
        let mut draft = crate::workflow::synthesis::generate_candidate_drafts(
            WorkflowScope::default(),
            "human gate",
            WorkflowMotifKind::HumanGate,
            &[],
            1,
            "tester",
            "human",
            "2026-03-11T00:00:00Z",
            "seed",
        )
        .remove(0);
        let checkpoint = draft
            .graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == WorkflowNodeKind::HumanCheckpoint)
            .unwrap();
        checkpoint
            .checkpoint_policy
            .as_mut()
            .unwrap()
            .cancel_allowed = false;
        let result = validate_workflow_draft(&draft);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "invalid_checkpoint_policy"));
    }

    #[test]
    fn parallel_compare_projection_orders_branches_stably() {
        let draft = crate::workflow::synthesis::generate_candidate_drafts(
            WorkflowScope::default(),
            "parallel compare",
            WorkflowMotifKind::ParallelCompare,
            &[],
            1,
            "tester",
            "human",
            "2026-03-11T00:00:00Z",
            "seed",
        )
        .remove(0);
        let compiled = compile_workflow_draft(&draft).unwrap();
        let branches = compiled.serverless_workflow_projection["states"][0]["branches"]
            .as_array()
            .unwrap();
        assert_eq!(branches.len(), 2);
        let branch_names = branches
            .iter()
            .map(|branch| branch["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        let mut sorted = branch_names.clone();
        sorted.sort();
        assert_eq!(branch_names, sorted);
    }

    #[test]
    fn capability_call_requires_authority() {
        let mut draft = crate::workflow::synthesis::generate_candidate_drafts(
            WorkflowScope::default(),
            "sequential",
            WorkflowMotifKind::Sequential,
            &[],
            1,
            "tester",
            "human",
            "2026-03-11T00:00:00Z",
            "seed",
        )
        .remove(0);
        let capability = draft
            .graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == WorkflowNodeKind::CapabilityCall)
            .unwrap();
        capability.authority_requirements.clear();
        let result = validate_workflow_draft(&draft);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "missing_authority_requirement"));
    }
}
