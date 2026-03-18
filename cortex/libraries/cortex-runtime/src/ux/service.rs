use cortex_domain::ux::{
    UxCandidateEvaluation, UxLayoutEvaluationRequest, UxPromotionDecision, evaluate_cuqs,
};

pub fn evaluate_layout(
    request: UxLayoutEvaluationRequest,
) -> (UxCandidateEvaluation, Option<UxPromotionDecision>) {
    let evaluation = evaluate_cuqs(request);
    let promotion_decision = if evaluation.promotion_status == "eligible_hitl_approved" {
        match (
            evaluation.approved_by.clone(),
            evaluation.approval_rationale.clone(),
            evaluation.approved_at.clone(),
        ) {
            (Some(approved_by), Some(rationale), Some(timestamp)) => Some(UxPromotionDecision {
                decision_id: format!("ux_promotion_{}_{}", evaluation.candidate_id, timestamp),
                candidate_id: evaluation.candidate_id.clone(),
                route_id: evaluation.route_id.clone(),
                view_capability_id: evaluation.view_capability_id.clone(),
                promotion_action: "promote_structural_candidate".to_string(),
                approved_by,
                rationale,
                timestamp,
            }),
            _ => None,
        }
    } else {
        None
    };

    (evaluation, promotion_decision)
}
