use crate::workflows::op_registry::{ACP_COLLECT_OP, ACP_EVALUATE_OP, ACP_PUBLISH_OP};
use nostra_workflow_core::{
    WorkflowDefinition,
    primitives::{Action, Step, Transition},
    types::Context,
};
use serde_json::json;
use std::collections::HashMap;

const DEFAULT_ACP_METRICS_BASE_URL: &str = "http://127.0.0.1:3000";
const DEFAULT_ACP_METRICS_LOG: &str = "/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl";
const DEFAULT_ACP_EVIDENCE_FILE: &str =
    "/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md";

fn is_slo_pass(context: &Context) -> bool {
    context
        .get("acp.slo.result")
        .map(|value| value.eq_ignore_ascii_case("PASS"))
        .unwrap_or(false)
}

fn is_slo_not_pass(context: &Context) -> bool {
    !is_slo_pass(context)
}

pub fn create_acp_pilot_ops_workflow() -> WorkflowDefinition {
    let mut steps = HashMap::new();

    let metrics_base_url = std::env::var("CORTEX_ACP_METRICS_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_ACP_METRICS_BASE_URL.to_string());
    let metrics_log = std::env::var("CORTEX_ACP_METRICS_LOG_PATH")
        .unwrap_or_else(|_| DEFAULT_ACP_METRICS_LOG.to_string());
    let evidence_file = std::env::var("CORTEX_ACP_EVIDENCE_FILE")
        .unwrap_or_else(|_| DEFAULT_ACP_EVIDENCE_FILE.to_string());

    let collect_metrics = Step::new("collect_metrics", "Collect ACP Metrics")
        .with_action(Action::SystemOp {
            op_type: ACP_COLLECT_OP.to_string(),
            payload: json!({
                "base_url": metrics_base_url,
                "out_file": metrics_log,
            })
            .to_string(),
        })
        .with_transition(Transition::to("evaluate_slo"));

    let evaluate_slo = Step::new("evaluate_slo", "Evaluate ACP SLO")
        .with_action(Action::SystemOp {
            op_type: ACP_EVALUATE_OP.to_string(),
            payload: json!({
                "input_file": metrics_log,
            })
            .to_string(),
        })
        .with_transition(Transition::to("publish_evidence"));

    let publish_evidence = Step::new("publish_evidence", "Publish ACP Evidence")
        .with_action(Action::SystemOp {
            op_type: ACP_PUBLISH_OP.to_string(),
            payload: json!({
                "evidence_file": evidence_file,
                "confidence_score": 0.85,
                "source_reliability": "high",
                "validation_proof": "/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh",
            })
            .to_string(),
        })
        .with_transition(Transition::to("steward_gate").when(is_slo_pass))
        .with_transition(Transition::to("complete").when(is_slo_not_pass));

    let steward_gate = Step::new("steward_gate", "Steward Promotion Gate")
        .with_action(Action::UserTask {
            description: "Review ACP evidence and approve promotion decision request".to_string(),
            candidate_roles: vec!["steward".to_string()],
            candidate_users: vec![],
            a2ui_schema: None,
        })
        .with_transition(Transition::to("complete"));

    let complete = Step::new("complete", "ACP Pilot Ops Complete").with_action(Action::None);

    steps.insert("collect_metrics".to_string(), collect_metrics);
    steps.insert("evaluate_slo".to_string(), evaluate_slo);
    steps.insert("publish_evidence".to_string(), publish_evidence);
    steps.insert("steward_gate".to_string(), steward_gate);
    steps.insert("complete".to_string(), complete);

    WorkflowDefinition {
        id: "acp_pilot_ops".to_string(),
        steps,
        start_step_id: "collect_metrics".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_step_routes_to_steward_gate_on_pass() {
        let workflow = create_acp_pilot_ops_workflow();
        let step = workflow.steps.get("publish_evidence").unwrap();

        let mut context = Context::new();
        context.set("acp.slo.result", "PASS");

        assert_eq!(step.transitions.len(), 2);
        assert!(step.transitions[0].condition.unwrap()(&context));
        assert!(!step.transitions[1].condition.unwrap()(&context));
    }

    #[test]
    fn publish_step_routes_to_complete_on_fail() {
        let workflow = create_acp_pilot_ops_workflow();
        let step = workflow.steps.get("publish_evidence").unwrap();

        let mut context = Context::new();
        context.set("acp.slo.result", "FAIL");

        assert_eq!(step.transitions.len(), 2);
        assert!(!step.transitions[0].condition.unwrap()(&context));
        assert!(step.transitions[1].condition.unwrap()(&context));
    }
}
