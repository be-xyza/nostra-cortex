use crate::services::workflow_engine_client::WorkflowEngineClient;
use cortex_runtime::ports::{AgentEvaluationLoopRequest, AgentEvaluationLoopResult, GateOutcome};

fn parse_risk_score(request: &AgentEvaluationLoopRequest) -> Option<f64> {
    request
        .simulation
        .as_ref()
        .and_then(|value| value.get("riskScore"))
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_u64().map(|entry| entry as f64))
                .or_else(|| value.as_str().and_then(|entry| entry.parse::<f64>().ok()))
        })
}

fn fallback_result(request: &AgentEvaluationLoopRequest) -> AgentEvaluationLoopResult {
    let Some(risk_score) = parse_risk_score(request) else {
        return AgentEvaluationLoopResult {
            gate_outcome: GateOutcome::Warn,
            allowed: true,
            reasons: vec!["fallback_missing_risk_score".to_string()],
            confidence_score: None,
            source_reliability: None,
            policy_ref: None,
        };
    };

    if risk_score > 80.0 {
        AgentEvaluationLoopResult {
            gate_outcome: GateOutcome::Block,
            allowed: false,
            reasons: vec![format!("fallback_risk_score_blocked:{risk_score:.2}")],
            confidence_score: None,
            source_reliability: None,
            policy_ref: None,
        }
    } else if risk_score > 50.0 {
        AgentEvaluationLoopResult {
            gate_outcome: GateOutcome::RequireReview,
            allowed: false,
            reasons: vec![format!(
                "fallback_risk_score_requires_review:{risk_score:.2}"
            )],
            confidence_score: None,
            source_reliability: None,
            policy_ref: None,
        }
    } else {
        AgentEvaluationLoopResult {
            gate_outcome: GateOutcome::Pass,
            allowed: true,
            reasons: vec![format!("fallback_risk_score_passed:{risk_score:.2}")],
            confidence_score: None,
            source_reliability: None,
            policy_ref: None,
        }
    }
}

pub async fn evaluate_agent_gate(
    request: &AgentEvaluationLoopRequest,
) -> AgentEvaluationLoopResult {
    let client = match WorkflowEngineClient::from_env().await {
        Ok(client) => client,
        Err(_) => return fallback_result(request),
    };

    match client
        .get_epistemic_assessment_by_mutation(request.run_id.as_str())
        .await
    {
        Ok(Some(assessment)) => {
            let allowed = matches!(
                assessment.gate_outcome,
                GateOutcome::Pass | GateOutcome::Warn
            );
            AgentEvaluationLoopResult {
                gate_outcome: assessment.gate_outcome,
                allowed,
                reasons: assessment.reasons,
                confidence_score: Some(assessment.confidence_score),
                source_reliability: Some(assessment.source_reliability),
                policy_ref: None,
            }
        }
        Ok(None) | Err(_) => fallback_result(request),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(risk_score: Option<f64>) -> AgentEvaluationLoopRequest {
        AgentEvaluationLoopRequest {
            run_id: "run-1".to_string(),
            workflow_id: "wf-1".to_string(),
            space_id: "space-1".to_string(),
            authority_level: "l2".to_string(),
            simulation: risk_score.map(|score| json!({ "riskScore": score })),
            action_target: Some("ic://kg-canister/create_context_node".to_string()),
        }
    }

    #[test]
    fn fallback_blocks_high_risk() {
        let result = fallback_result(&request(Some(90.0)));
        assert_eq!(result.gate_outcome, GateOutcome::Block);
        assert!(!result.allowed);
    }

    #[test]
    fn fallback_requires_review_for_mid_risk() {
        let result = fallback_result(&request(Some(55.0)));
        assert_eq!(result.gate_outcome, GateOutcome::RequireReview);
        assert!(!result.allowed);
    }

    #[test]
    fn fallback_passes_low_risk() {
        let result = fallback_result(&request(Some(12.0)));
        assert_eq!(result.gate_outcome, GateOutcome::Pass);
        assert!(result.allowed);
    }

    #[test]
    fn fallback_warns_when_missing_risk() {
        let result = fallback_result(&request(None));
        assert_eq!(result.gate_outcome, GateOutcome::Warn);
        assert!(result.allowed);
    }
}
