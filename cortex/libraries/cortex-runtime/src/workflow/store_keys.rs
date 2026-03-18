use cortex_domain::workflow::{WorkflowScope, scope_key};

pub fn intent_key(scope: &WorkflowScope, workflow_intent_id: &str) -> String {
    format!(
        "/cortex/workflows/intents/{}/{}.json",
        scope_key(scope),
        sanitize_token(workflow_intent_id)
    )
}

pub fn candidate_set_key(scope: &WorkflowScope, candidate_set_id: &str) -> String {
    format!(
        "/cortex/workflows/drafts/candidates/{}/{}.json",
        scope_key(scope),
        sanitize_token(candidate_set_id)
    )
}

pub fn proposal_key(scope: &WorkflowScope, proposal_id: &str) -> String {
    format!(
        "/cortex/workflows/drafts/proposals/{}/{}.json",
        scope_key(scope),
        sanitize_token(proposal_id)
    )
}

pub fn compiled_definition_key(scope: &WorkflowScope, definition_id: &str) -> String {
    format!(
        "/cortex/workflows/definitions/compiled/{}/{}.json",
        scope_key(scope),
        sanitize_token(definition_id)
    )
}

pub fn active_definition_key(scope: &WorkflowScope) -> String {
    format!("/cortex/workflows/definitions/active/{}.json", scope_key(scope))
}

pub fn replay_key(proposal_id: &str, run_id: &str) -> String {
    format!(
        "/cortex/workflows/replay/{}/{}.json",
        sanitize_token(proposal_id),
        sanitize_token(run_id)
    )
}

pub fn events_key(day_yyyy_mm_dd: &str) -> String {
    format!("/cortex/workflows/events/{}.jsonl", sanitize_token(day_yyyy_mm_dd))
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
