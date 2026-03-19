use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    pub session_id: String,
    pub space_id: String,
    pub contribution_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerOptions {
    pub max_iterations: u32,
    pub enable_streaming: bool,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            enable_streaming: true,
        }
    }
}

/// A stateless orchestrator responsible for executing an Agent's logic.
/// Derived from the Google ADK Python pattern.
/// It yields discrete Event objects instead of holding internal state loop buffers.
pub struct Runner {
    pub options: RunnerOptions,
}

impl Runner {
    pub fn new(options: RunnerOptions) -> Self {
        Self { options }
    }

    /// Executing an agent asynchronously, yielding an event stream list.
    pub async fn run_agent(
        &self,
        agent_id: &str,
        context: &InvocationContext,
    ) -> Result<Vec<crate::agent::contracts::AgentRunEvent>, crate::DomainError> {
        let mut events = Vec::new();
        let run_id = format!("run_{}", context.session_id); // Basic deterministic id for demonstration

        // Push initial event
        events.push(crate::agent::contracts::AgentRunEvent {
            event_type: "RunStarted".to_string(),
            run_id: run_id.clone(),
            space_id: context.space_id.clone(),
            timestamp: "2026-02-28T12:00:00Z".to_string(), // Placeholder format
            sequence: 0,
            payload: serde_json::json!({
                "agent_id": agent_id,
                "msg": "Agent invocation started (Stateless Runner)"
            }),
        });

        // The core agent loop processes tools statelessly, calling out to ModelProviders.
        // Conceptually, for every execution iteration, it emits progress events.

        events.push(crate::agent::contracts::AgentRunEvent {
            event_type: "RunCompleted".to_string(),
            run_id,
            space_id: context.space_id.clone(),
            timestamp: "2026-02-28T12:00:01Z".to_string(),
            sequence: 1,
            payload: serde_json::json!({
                "agent_id": agent_id,
                "msg": "Agent invocation completed"
            }),
        });

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runner_yields_expected_event_sequence() {
        let runner = Runner::new(RunnerOptions::default());
        let context = InvocationContext {
            session_id: "test_session".to_string(),
            space_id: "test_space".to_string(),
            contribution_id: None,
        };

        let events = runner.run_agent("mock_agent", &context).await.unwrap();

        // As per ADK pattern, we expect discrete events emitted sequentially rather than one final block.
        assert!(events.len() >= 2);
        assert_eq!(events[0].event_type, "RunStarted");
        assert_eq!(events.last().unwrap().event_type, "RunCompleted");
    }
}
