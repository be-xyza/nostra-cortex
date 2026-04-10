use nostra_workflow_core::{
    WorkflowDefinition,
    primitives::{Action, Step, Transition},
};
use std::collections::HashMap;

pub fn create_gap_closure_workflow() -> WorkflowDefinition {
    let mut steps = HashMap::new();

    // 1. Research
    let research = Step::new("research", "Researching Context")
        .with_action(Action::SystemOp {
            op_type: "research_agent".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("plan"));

    // 2. Plan
    let plan = Step::new("plan", "Generating Execution Plan")
        .with_action(Action::SystemOp {
            op_type: "planning_agent".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("execute"));

    // 3. Execute
    let execute = Step::new("execute", "Executing Gap Closure")
        .with_action(Action::SystemOp {
            op_type: "execution_agent".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("verify"));

    // 4. Verify
    let verify = Step::new("verify", "Verifying Outcome")
        .with_action(Action::SystemOp {
            op_type: "verification_agent".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("done"));

    // 5. Done
    let done = Step::new("done", "Gap Closed").with_action(Action::None);

    steps.insert("research".to_string(), research);
    steps.insert("plan".to_string(), plan);
    steps.insert("execute".to_string(), execute);
    steps.insert("verify".to_string(), verify);
    steps.insert("done".to_string(), done);

    WorkflowDefinition {
        id: "gap_closure".to_string(),
        steps,
        start_step_id: "research".to_string(),
    }
}
