use crate::api::{create_agent, process_ai_query};
use dioxus::prelude::*;

pub struct StandardsAgentService;

impl StandardsAgentService {
    /// Calls the agent to enhance a raw constraint into one that enforces Nostra Standards.
    pub async fn enhance_constraint(raw_constraint: String) -> Result<String, String> {
        let agent = create_agent().await;

        let prompt = format!(
            "You are a Senior Nostra Architect. Your goal is to rewrite user constraints to strictly enforce Nostra System Standards (040-schema-standards, 013-workflow-engine).
            
            Input Constraint: '{}'
            
            Rewrite this constraint to be specific, technical, and refer to standard schemas where applicable. 
            Output ONLY the rewritten constraint text, no chatter.",
            raw_constraint
        );

        let response = process_ai_query(&agent, prompt).await?;
        Ok(response.trim().to_string())
    }

    /// Calls the agent to suggest constraints based on a goal.
    pub async fn suggest_constraints(goal: String) -> Result<Vec<String>, String> {
        let agent = create_agent().await;

        let prompt = format!(
            "Given the goal: '{}', list 3-5 critical technical constraints that must be met for a valid Nostra A2UI implementation.
            Focus on data integrity, schema compliance, and accessibility.
            Output as a JSON string array of strings only. Example: [\"Constraint 1\", \"Constraint 2\"]",
            goal
        );

        let response = process_ai_query(&agent, prompt).await?;

        // Attempt to parse JSON from the response (it might be wrapped in md blocks)
        let cleaned = if let Some(start) = response.find("[") {
            let end = response.rfind("]").unwrap_or(response.len()) + 1;
            &response[start..end]
        } else {
            &response
        };

        match serde_json::from_str::<Vec<String>>(cleaned) {
            Ok(list) => Ok(list),
            Err(e) => Err(format!(
                "Failed to parse agent response: {} Raw: {}",
                e, response
            )),
        }
    }
}
