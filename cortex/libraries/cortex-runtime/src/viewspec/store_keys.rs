use cortex_domain::viewspec::{
    ViewSpecScope, current_viewspec_key, history_viewspec_key, proposal_store_key, scope_key,
    viewspec_events_key,
};

pub fn scope_store_key(scope: &ViewSpecScope) -> String {
    scope_key(scope)
}

pub fn current_key(scope: &ViewSpecScope, view_spec_id: &str) -> String {
    current_viewspec_key(scope, view_spec_id)
}

pub fn history_key(scope: &ViewSpecScope, view_spec_id: &str, timestamp: &str) -> String {
    history_viewspec_key(scope, view_spec_id, timestamp)
}

pub fn proposal_key(scope: &ViewSpecScope, proposal_id: &str) -> String {
    proposal_store_key(scope, proposal_id)
}

pub fn events_key(day_yyyy_mm_dd: &str) -> String {
    viewspec_events_key(day_yyyy_mm_dd)
}
