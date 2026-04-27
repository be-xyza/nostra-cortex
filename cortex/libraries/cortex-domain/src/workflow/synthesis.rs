use crate::workflow::types::{
    ContextContractV1, WorkflowCandidateEnvelope, WorkflowCandidateSet, WorkflowConfidence,
    WorkflowConstraintRule, WorkflowDraftPolicyV1, WorkflowDraftV1, WorkflowEdgeV1,
    WorkflowGenerationMode, WorkflowGenerationTrace, WorkflowGraphV1, WorkflowMotifKind,
    WorkflowNodeKind, WorkflowNodeV1, WorkflowProvenance, WorkflowScope, WORKFLOW_SCHEMA_VERSION,
};
use crate::workflow::validation::{compile_workflow_draft, scope_key, validate_workflow_draft};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const WORKFLOW_CANDIDATE_SET_INDEX_KEY: &str = "/cortex/workflows/drafts/candidates/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSeedSpec {
    pub candidate_index: usize,
    pub seed: String,
}

fn sanitize_token(value: &str) -> String {
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

pub fn candidate_set_store_key(scope: &WorkflowScope, candidate_set_id: &str) -> String {
    format!(
        "/cortex/workflows/drafts/candidates/{}/{}.json",
        scope_key(scope),
        sanitize_token(candidate_set_id)
    )
}

fn node(node_id: String, label: String, kind: WorkflowNodeKind) -> WorkflowNodeV1 {
    WorkflowNodeV1 {
        node_id,
        label,
        kind,
        reads: Vec::new(),
        writes: Vec::new(),
        evidence_outputs: Vec::new(),
        authority_requirements: Vec::new(),
        checkpoint_policy: None,
        loop_policy: None,
        subflow_ref: None,
        config: json!({}),
    }
}

fn edge(from: &str, to: &str, relation: &str) -> WorkflowEdgeV1 {
    WorkflowEdgeV1 {
        edge_id: format!("{from}__{relation}__{to}"),
        from: from.to_string(),
        to: to.to_string(),
        relation: relation.to_string(),
    }
}

fn build_graph(motif_kind: &WorkflowMotifKind, seed: &WorkflowSeedSpec) -> WorkflowGraphV1 {
    let prefix = format!(
        "wf{}_{}",
        seed.candidate_index + 1,
        sanitize_token(&seed.seed)
    );
    match motif_kind {
        WorkflowMotifKind::Sequential => {
            let capability_id = format!("{prefix}_task");
            let terminal_id = format!("{prefix}_done");
            let mut capability = node(
                capability_id.clone(),
                "Execute capability".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            capability.reads = vec!["inputs.prompt".to_string()];
            capability.writes = vec!["artifacts.primary".to_string()];
            capability.authority_requirements = vec!["execution.apply".to_string()];
            capability.evidence_outputs = vec!["artifact://primary".to_string()];

            WorkflowGraphV1 {
                nodes: vec![
                    capability,
                    node(
                        terminal_id.clone(),
                        "Complete".to_string(),
                        WorkflowNodeKind::Terminal,
                    ),
                ],
                edges: vec![edge(&capability_id, &terminal_id, "transition")],
            }
        }
        WorkflowMotifKind::ParallelCompare => {
            let parallel_id = format!("{prefix}_parallel");
            let left_id = format!("{prefix}_candidate_a");
            let right_id = format!("{prefix}_candidate_b");
            let eval_id = format!("{prefix}_eval");
            let terminal_id = format!("{prefix}_done");

            let mut parallel = node(
                parallel_id.clone(),
                "Compare branches".to_string(),
                WorkflowNodeKind::Parallel,
            );
            parallel.reads = vec!["inputs.prompt".to_string()];
            parallel.writes = vec!["visibility.branch_plan".to_string()];

            let mut left = node(
                left_id.clone(),
                "Capability branch A".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            left.reads = vec!["inputs.prompt".to_string()];
            left.writes = vec!["artifacts.branch_a".to_string()];
            left.authority_requirements = vec!["execution.apply".to_string()];
            left.evidence_outputs = vec!["artifact://branch_a".to_string()];

            let mut right = node(
                right_id.clone(),
                "Capability branch B".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            right.reads = vec!["inputs.prompt".to_string()];
            right.writes = vec!["artifacts.branch_b".to_string()];
            right.authority_requirements = vec!["execution.apply".to_string()];
            right.evidence_outputs = vec!["artifact://branch_b".to_string()];

            let mut eval = node(
                eval_id.clone(),
                "Evaluate branches".to_string(),
                WorkflowNodeKind::EvaluationGate,
            );
            eval.reads = vec![
                "artifacts.branch_a".to_string(),
                "artifacts.branch_b".to_string(),
            ];
            eval.writes = vec!["evaluation.selected".to_string()];
            eval.evidence_outputs = vec!["evaluation://grader".to_string()];
            eval.config = json!({
                "selector": "highest_score",
                "preserveDiscards": true,
            });

            WorkflowGraphV1 {
                nodes: vec![
                    parallel,
                    left,
                    right,
                    eval,
                    node(
                        terminal_id.clone(),
                        "Complete".to_string(),
                        WorkflowNodeKind::Terminal,
                    ),
                ],
                edges: vec![
                    edge(&parallel_id, &left_id, "parallel_branch"),
                    edge(&parallel_id, &right_id, "parallel_branch"),
                    edge(&left_id, &eval_id, "branch_join"),
                    edge(&right_id, &eval_id, "branch_join"),
                    edge(&eval_id, &terminal_id, "transition"),
                ],
            }
        }
        WorkflowMotifKind::RepairLoop => {
            let capability_id = format!("{prefix}_repair");
            let eval_id = format!("{prefix}_eval");
            let loop_id = format!("{prefix}_loop");
            let terminal_id = format!("{prefix}_done");

            let mut capability = node(
                capability_id.clone(),
                "Repair capability".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            capability.reads = vec![
                "inputs.prompt".to_string(),
                "evaluation.feedback".to_string(),
            ];
            capability.writes = vec!["artifacts.repair_candidate".to_string()];
            capability.authority_requirements = vec!["execution.apply".to_string()];
            capability.evidence_outputs = vec!["artifact://repair_candidate".to_string()];

            let mut eval = node(
                eval_id.clone(),
                "Check repair".to_string(),
                WorkflowNodeKind::EvaluationGate,
            );
            eval.reads = vec!["artifacts.repair_candidate".to_string()];
            eval.writes = vec![
                "evaluation.pass".to_string(),
                "evaluation.feedback".to_string(),
            ];
            eval.evidence_outputs = vec!["evaluation://repair_gate".to_string()];

            let mut loop_node = node(
                loop_id.clone(),
                "Bounded repair loop".to_string(),
                WorkflowNodeKind::Loop,
            );
            loop_node.reads = vec!["evaluation.pass".to_string()];
            loop_node.writes = vec!["control.iteration".to_string()];
            loop_node.loop_policy = Some(crate::workflow::types::WorkflowLoopPolicyV1 {
                max_iterations: Some(3),
                termination_expression: Some("evaluation.pass == true".to_string()),
            });

            WorkflowGraphV1 {
                nodes: vec![
                    capability,
                    eval,
                    loop_node,
                    node(
                        terminal_id.clone(),
                        "Complete".to_string(),
                        WorkflowNodeKind::Terminal,
                    ),
                ],
                edges: vec![
                    edge(&capability_id, &eval_id, "transition"),
                    edge(&eval_id, &loop_id, "transition"),
                    edge(&loop_id, &capability_id, "loop_back"),
                    edge(&eval_id, &terminal_id, "transition"),
                ],
            }
        }
        WorkflowMotifKind::FanOutJoin => {
            let parallel_id = format!("{prefix}_parallel");
            let left_id = format!("{prefix}_fan_a");
            let right_id = format!("{prefix}_fan_b");
            let terminal_id = format!("{prefix}_done");

            let mut parallel = node(
                parallel_id.clone(),
                "Fan out".to_string(),
                WorkflowNodeKind::Parallel,
            );
            parallel.reads = vec!["inputs.prompt".to_string()];
            parallel.writes = vec!["visibility.fan_out".to_string()];

            let mut left = node(
                left_id.clone(),
                "Capability shard A".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            left.reads = vec!["inputs.prompt".to_string()];
            left.writes = vec!["artifacts.fan_a".to_string()];
            left.authority_requirements = vec!["execution.apply".to_string()];

            let mut right = node(
                right_id.clone(),
                "Capability shard B".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            right.reads = vec!["inputs.prompt".to_string()];
            right.writes = vec!["artifacts.fan_b".to_string()];
            right.authority_requirements = vec!["execution.apply".to_string()];

            WorkflowGraphV1 {
                nodes: vec![
                    parallel,
                    left,
                    right,
                    node(
                        terminal_id.clone(),
                        "Complete".to_string(),
                        WorkflowNodeKind::Terminal,
                    ),
                ],
                edges: vec![
                    edge(&parallel_id, &left_id, "parallel_branch"),
                    edge(&parallel_id, &right_id, "parallel_branch"),
                    edge(&left_id, &terminal_id, "branch_join"),
                    edge(&right_id, &terminal_id, "branch_join"),
                ],
            }
        }
        WorkflowMotifKind::HumanGate => {
            let capability_id = format!("{prefix}_task");
            let checkpoint_id = format!("{prefix}_review");
            let terminal_id = format!("{prefix}_done");

            let mut capability = node(
                capability_id.clone(),
                "Prepare candidate".to_string(),
                WorkflowNodeKind::CapabilityCall,
            );
            capability.reads = vec!["inputs.prompt".to_string()];
            capability.writes = vec!["artifacts.candidate".to_string()];
            capability.authority_requirements = vec!["execution.apply".to_string()];

            let mut checkpoint = node(
                checkpoint_id.clone(),
                "Human checkpoint".to_string(),
                WorkflowNodeKind::HumanCheckpoint,
            );
            checkpoint.reads = vec!["artifacts.candidate".to_string()];
            checkpoint.writes = vec!["control.approval".to_string()];
            checkpoint.checkpoint_policy =
                Some(crate::workflow::types::WorkflowCheckpointPolicyV1 {
                    resume_allowed: true,
                    cancel_allowed: true,
                    pause_allowed: true,
                    timeout_seconds: Some(3600),
                });
            checkpoint.config = json!({
                "surfaceTemplate": "workflow_review_card",
            });

            WorkflowGraphV1 {
                nodes: vec![
                    capability,
                    checkpoint,
                    node(
                        terminal_id.clone(),
                        "Complete".to_string(),
                        WorkflowNodeKind::Terminal,
                    ),
                ],
                edges: vec![
                    edge(&capability_id, &checkpoint_id, "transition"),
                    edge(&checkpoint_id, &terminal_id, "transition"),
                ],
            }
        }
    }
}

fn build_trace(mode: &WorkflowGenerationMode, idx: usize) -> WorkflowGenerationTrace {
    let mut policy_flags = BTreeMap::new();
    policy_flags.insert("autonomous_promotion_disabled".to_string(), true);
    policy_flags.insert("shadow_execution_advisory_only".to_string(), true);
    policy_flags.insert("direct_runtime_execution_disabled".to_string(), true);

    WorkflowGenerationTrace {
        strategy: mode.as_str().to_string(),
        seed_refs: vec![format!("workflow.seed.{}", idx + 1)],
        policy_flags,
    }
}

pub fn compute_candidate_input_hash(
    draft: &WorkflowDraftV1,
    trace: &WorkflowGenerationTrace,
    mode: &WorkflowGenerationMode,
) -> String {
    let canonical = serde_json::json!({
        "mode": mode.as_str(),
        "draft": draft,
        "generationTrace": trace,
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn blocked_count(candidates: &[WorkflowCandidateEnvelope]) -> u32 {
    candidates
        .iter()
        .filter(|candidate| !candidate.validation.valid)
        .count() as u32
}

#[allow(clippy::too_many_arguments)]
pub fn generate_candidate_drafts(
    scope: WorkflowScope,
    intent: &str,
    motif_kind: WorkflowMotifKind,
    constraints: &[WorkflowConstraintRule],
    count: usize,
    created_by: &str,
    source_mode: &str,
    created_at: &str,
    seed: &str,
) -> Vec<WorkflowDraftV1> {
    (0..count)
        .map(|idx| {
            let candidate_seed = WorkflowSeedSpec {
                candidate_index: idx,
                seed: format!("{seed}_{idx}"),
            };
            WorkflowDraftV1 {
                schema_version: WORKFLOW_SCHEMA_VERSION.to_string(),
                workflow_draft_id: format!("workflow_draft_{}_{}", sanitize_token(seed), idx + 1),
                scope: scope.clone(),
                intent_ref: None,
                intent: intent.to_string(),
                motif_kind: motif_kind.clone(),
                constraints: constraints.to_vec(),
                graph: build_graph(&motif_kind, &candidate_seed),
                context_contract: ContextContractV1::default(),
                confidence: WorkflowConfidence {
                    score: 0.72,
                    rationale: "Deterministic scaffold generated from motif library".to_string(),
                },
                lineage: Default::default(),
                policy: WorkflowDraftPolicyV1 {
                    recommendation_only: true,
                    require_review: true,
                    allow_shadow_execution: false,
                },
                provenance: WorkflowProvenance {
                    created_by: created_by.to_string(),
                    created_at: created_at.to_string(),
                    source_mode: source_mode.to_string(),
                },
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn generate_candidate_set(
    scope: WorkflowScope,
    intent: &str,
    motif_kind: WorkflowMotifKind,
    constraints: &[WorkflowConstraintRule],
    count: usize,
    created_by: &str,
    source_mode: &str,
    mode: WorkflowGenerationMode,
    candidate_set_id: &str,
    created_at: &str,
    candidate_seed: &str,
) -> WorkflowCandidateSet {
    let mut drafts = generate_candidate_drafts(
        scope.clone(),
        intent,
        motif_kind.clone(),
        constraints,
        count,
        created_by,
        source_mode,
        created_at,
        candidate_seed,
    );

    if mode == WorkflowGenerationMode::MotifHybrid {
        for (idx, draft) in drafts.iter_mut().enumerate() {
            if let Some(parallel_node) = draft
                .graph
                .nodes
                .iter_mut()
                .find(|node| node.kind == WorkflowNodeKind::Parallel)
            {
                parallel_node.config = json!({
                    "mergeStrategy": if idx % 2 == 0 { "highest_score" } else { "human_select" },
                });
            }
        }
    }

    let candidates = drafts
        .into_iter()
        .enumerate()
        .map(|(idx, draft)| {
            let validation = validate_workflow_draft(&draft);
            let compile_result = if validation.valid {
                compile_workflow_draft(&draft).ok()
            } else {
                None
            };
            let generation_trace = build_trace(&mode, idx);
            let input_hash = compute_candidate_input_hash(&draft, &generation_trace, &mode);
            WorkflowCandidateEnvelope {
                candidate_id: draft.workflow_draft_id.clone(),
                workflow_draft: draft,
                validation,
                compile_result,
                generation_trace,
                input_hash,
            }
        })
        .collect();

    WorkflowCandidateSet {
        candidate_set_id: candidate_set_id.to_string(),
        scope_key: scope_key(&scope),
        intent: intent.to_string(),
        motif_kind,
        constraints: constraints.to_vec(),
        mode,
        created_by: created_by.to_string(),
        created_at: created_at.to_string(),
        candidates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_loop_generation_contains_loop_back_edge() {
        let drafts = generate_candidate_drafts(
            WorkflowScope::default(),
            "repair",
            WorkflowMotifKind::RepairLoop,
            &[],
            1,
            "tester",
            "human",
            "2026-03-11T00:00:00Z",
            "seed",
        );
        assert_eq!(drafts.len(), 1);
        assert!(drafts[0]
            .graph
            .edges
            .iter()
            .any(|edge| edge.relation == "loop_back"));
    }
}
