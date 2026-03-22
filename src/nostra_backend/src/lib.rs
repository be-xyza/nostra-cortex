use ic_cdk_macros::{init, query, update};
use nostra_workflow_engine::execution::StateMachine;
use nostra_workflow_engine::registry::WorkflowRegistry;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REGISTRY: RefCell<WorkflowRegistry> = RefCell::new(WorkflowRegistry::new());
    static ENGINE: RefCell<StateMachine> = RefCell::new(StateMachine::new());
}

#[init]
fn init() {
    // Registry auto-initializes with defaults
}

#[query]
fn list_workflows() -> Vec<String> {
    // In a real app we'd return full definitions, for now just IDs
    vec![
        "WORKFLOW_TEMPLATE_WIZARD".to_string(),
        "WORKFLOW_REQUEST_REVIEW".to_string(),
        "WORKFLOW_PUBLISH_EDITION".to_string(),
    ]
}

#[update]
fn trigger_workflow(workflow_id: String) -> String {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        // Start in RENDER_FORM state for valid wizards
        engine.create_instance(workflow_id, "RENDER_FORM".to_string())
    })
}

#[query]
fn get_workflow_state(instance_id: String) -> String {
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        if let Some(instance) = engine.get_instance(&instance_id) {
            // If in RENDER_FORM, generate A2UI JSON
            if instance.current_state == "RENDER_FORM" {
                // TODO: Look up actual fields from Registry
                // For MVP, returning a mock A2UI JSON string
                return r#"{ "surfaceUpdate": { "components": [] } }"#.to_string();
            }
            return serde_json::to_string(&instance).unwrap_or_default();
        }
        "{}".to_string()
    })
}

#[update]
fn submit_workflow_step(instance_id: String, data: HashMap<String, String>) -> String {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        match engine.transition(&instance_id, "SUBMIT", data) {
            Ok(new_state) => new_state,
            Err(e) => format!("Error: {}", e),
        }
    })
}
