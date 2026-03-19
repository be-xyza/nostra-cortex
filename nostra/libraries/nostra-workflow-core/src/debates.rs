//! Debate Workflow Definition
//!
//! A Temporal governance workflow for structured deliberation.
//!
//! Phases:
//! 1. Open - Debate parameters defined
//! 2. Deliberation - Arguments submitted
//! 3. Rebuttal - Counter-arguments
//! 4. Synthesis - AI/human summarization
//! 5. Resolved - Output artifact created
//!
//! Per Strategy 034c: Epistemic & Deliberative Governance.

use crate::primitives::{
    Action, AsyncProviderStrategy, AsyncRetryPolicy, Step, Transition,
};
use crate::types::{StepId, WorkflowDefinition};
use std::collections::HashMap;

/// Debate workflow phase identifiers.
pub mod phases {
    pub const OPEN: &str = "debate.open";
    pub const DELIBERATION: &str = "debate.deliberation";
    pub const REBUTTAL: &str = "debate.rebuttal";
    pub const SYNTHESIS: &str = "debate.synthesis";
    pub const RESOLVED: &str = "debate.resolved";
}

/// Create a new Debate workflow definition.
///
/// This workflow orchestrates structured deliberation with the following phases:
/// - Open: Define participants, rules, duration, target
/// - Deliberation: Collect arguments (nostra.argument)
/// - Rebuttal: Collect critiques (nostra.critique)
/// - Synthesis: AI/human summarization
/// - Resolved: Create synthesis artifact
///
/// # Arguments
///
/// * `id` - Unique workflow definition ID
///
/// # Example
///
/// ```rust,ignore
/// use nostra_workflow_core::debates::create_debate_workflow;
///
/// let debate = create_debate_workflow("debate-proposal-123");
/// let instance = WorkflowInstance::new("inst-1", debate);
/// ```
pub fn create_debate_workflow(id: impl Into<String>) -> WorkflowDefinition {
    let mut steps: HashMap<StepId, Step> = HashMap::new();

    // Phase 1: Open
    let open_step = Step::new(phases::OPEN, "Initialize debate parameters")
        .with_action(Action::UserTask {
            description: "Define debate rules, participants, duration, and target artifact".into(),
            candidate_roles: vec!["moderator".into()],
            candidate_users: vec![],
            a2ui_schema: Some(DEBATE_OPEN_A2UI.into()),
        })
        .with_transition(Transition::to(phases::DELIBERATION));

    // Phase 2: Deliberation
    let deliberation_step = Step::new(phases::DELIBERATION, "Collect arguments")
        .with_action(Action::UserTask {
            description: "Participants submit arguments (nostra.argument)".into(),
            candidate_roles: vec!["participant".into()],
            candidate_users: vec![],
            a2ui_schema: Some(ARGUMENT_SUBMISSION_A2UI.into()),
        })
        .with_transition(Transition::to(phases::REBUTTAL));

    // Phase 3: Rebuttal
    let rebuttal_step = Step::new(phases::REBUTTAL, "Counter-arguments and critiques")
        .with_action(Action::UserTask {
            description: "Participants submit rebuttals and critiques".into(),
            candidate_roles: vec!["participant".into()],
            candidate_users: vec![],
            a2ui_schema: Some(REBUTTAL_A2UI.into()),
        })
        .with_transition(Transition::to(phases::SYNTHESIS));

    // Phase 4: Synthesis
    let synthesis_step = Step::new(phases::SYNTHESIS, "Summarize and synthesize")
        .with_action(Action::AsyncExternalOp {
            target: "nostra.ai.summarizer".into(),
            input: r#"{"action": "synthesize_debate", "include_arguments": true}"#.into(),
            timeout_secs: 300,
            retry_policy: AsyncRetryPolicy {
                max_retries: 2,
                ..Default::default()
            },
            provider_strategy: AsyncProviderStrategy::Single,
        })
        .with_transition(Transition::to(phases::RESOLVED));

    // Phase 5: Resolved
    let resolved_step =
        Step::new(phases::RESOLVED, "Create synthesis artifact").with_action(Action::SystemOp {
            op_type: "Graph.CreateNode".into(),
            payload: r#"{"schema": "nostra.contribution", "type": "debate_synthesis"}"#.into(),
        });

    steps.insert(phases::OPEN.into(), open_step);
    steps.insert(phases::DELIBERATION.into(), deliberation_step);
    steps.insert(phases::REBUTTAL.into(), rebuttal_step);
    steps.insert(phases::SYNTHESIS.into(), synthesis_step);
    steps.insert(phases::RESOLVED.into(), resolved_step);

    WorkflowDefinition {
        id: id.into(),
        steps,
        start_step_id: phases::OPEN.into(),
    }
}

/// A2UI schema for the debate initialization form.
const DEBATE_OPEN_A2UI: &str = r#"{
    "surfaceId": "debate-open",
    "beginRendering": { "root": "form-container" },
    "surfaceUpdate": {
        "components": [
            {
                "id": "form-container",
                "componentProperties": {
                    "Column": {
                        "children": { "explicitList": ["title", "target-input", "duration-input", "rules-input", "submit-btn"] }
                    }
                }
            },
            {
                "id": "title",
                "componentProperties": { "Heading": { "text": "Initialize Debate", "level": 2 } }
            },
            {
                "id": "target-input",
                "componentProperties": { "TextField": { "label": "Target Artifact ID", "required": true } }
            },
            {
                "id": "duration-input",
                "componentProperties": { "TextField": { "label": "Duration (hours)", "required": true } }
            },
            {
                "id": "rules-input",
                "componentProperties": { "TextField": { "label": "Debate Rules", "multiline": true } }
            },
            {
                "id": "submit-btn",
                "componentProperties": {
                    "Button": {
                        "label": "Start Debate",
                        "action": { "actionType": "Submit", "submitName": "start_debate" }
                    }
                }
            }
        ]
    }
}"#;

/// A2UI schema for argument submission.
const ARGUMENT_SUBMISSION_A2UI: &str = r#"{
    "surfaceId": "argument-submit",
    "beginRendering": { "root": "arg-form" },
    "surfaceUpdate": {
        "components": [
            {
                "id": "arg-form",
                "componentProperties": {
                    "Column": {
                        "children": { "explicitList": ["title", "claim-input", "premises-input", "conclusion-input", "stance-select", "submit-btn"] }
                    }
                }
            },
            {
                "id": "title",
                "componentProperties": { "Heading": { "text": "Submit Argument", "level": 2 } }
            },
            {
                "id": "claim-input",
                "componentProperties": { "TextField": { "label": "Claim", "required": true } }
            },
            {
                "id": "premises-input",
                "componentProperties": { "TextField": { "label": "Premises (supporting evidence)", "multiline": true } }
            },
            {
                "id": "conclusion-input",
                "componentProperties": { "TextField": { "label": "Conclusion", "required": true } }
            },
            {
                "id": "stance-select",
                "componentProperties": {
                    "MultipleChoice": {
                        "selections": [
                            { "id": "support", "label": "Support" },
                            { "id": "oppose", "label": "Oppose" },
                            { "id": "alternative", "label": "Alternative" }
                        ]
                    }
                }
            },
            {
                "id": "submit-btn",
                "componentProperties": {
                    "Button": {
                        "label": "Submit Argument",
                        "action": { "actionType": "Submit", "submitName": "submit_argument" }
                    }
                }
            }
        ]
    }
}"#;

/// A2UI schema for rebuttal/critique submission.
const REBUTTAL_A2UI: &str = r#"{
    "surfaceId": "rebuttal-submit",
    "beginRendering": { "root": "rebuttal-form" },
    "surfaceUpdate": {
        "components": [
            {
                "id": "rebuttal-form",
                "componentProperties": {
                    "Column": {
                        "children": { "explicitList": ["title", "target-arg", "rebuttal-text", "submit-btn"] }
                    }
                }
            },
            {
                "id": "title",
                "componentProperties": { "Heading": { "text": "Submit Rebuttal", "level": 2 } }
            },
            {
                "id": "target-arg",
                "componentProperties": { "TextField": { "label": "Target Argument ID", "required": true } }
            },
            {
                "id": "rebuttal-text",
                "componentProperties": { "TextField": { "label": "Rebuttal", "multiline": true, "required": true } }
            },
            {
                "id": "submit-btn",
                "componentProperties": {
                    "Button": {
                        "label": "Submit Rebuttal",
                        "action": { "actionType": "Submit", "submitName": "submit_rebuttal" }
                    }
                }
            }
        ]
    }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_debate_workflow() {
        let workflow = create_debate_workflow("test-debate");

        assert_eq!(workflow.id, "test-debate");
        assert_eq!(workflow.start_step_id, phases::OPEN);
        assert_eq!(workflow.steps.len(), 5);

        // Verify all phases exist
        assert!(workflow.steps.contains_key(phases::OPEN));
        assert!(workflow.steps.contains_key(phases::DELIBERATION));
        assert!(workflow.steps.contains_key(phases::REBUTTAL));
        assert!(workflow.steps.contains_key(phases::SYNTHESIS));
        assert!(workflow.steps.contains_key(phases::RESOLVED));
    }
}
