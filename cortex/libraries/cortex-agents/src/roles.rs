use crate::provider::{
    CompletionRequest, Message, ModelProviderPort, ModelResponse, ProviderError, ProviderTool,
};
use crate::tools::CortexTool;

#[async_trait::async_trait]
pub trait CortexAgent: Send + Sync {
    /// The canonical identity/role name of the Agent.
    fn role_name(&self) -> &'static str;

    /// The instructions or persona context provided to the LLM.
    fn base_instructions(&self) -> &'static str;

    /// In the MVK, reason() takes a state history and returns an Ironclaw ModelResponse
    /// (Typically a list of ToolCalls that map to ActionTargets).
    async fn reason(
        &self,
        provider: &dyn ModelProviderPort,
        messages: &[Message],
        tools: &[&dyn CortexTool],
    ) -> Result<ModelResponse, ProviderError> {
        let system_prompt = self.base_instructions();
        let provider_tools: Vec<ProviderTool> = tools
            .iter()
            .map(|t| ProviderTool {
                name: t.name().to_string(),
                description: format!("Execute the {} action.", t.name()),
                parameters: t.json_schema(),
            })
            .collect();
        let mut prompt_messages = Vec::with_capacity(messages.len() + 1);
        prompt_messages.push(Message {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        });
        prompt_messages.extend(messages.iter().cloned());

        provider
            .complete(CompletionRequest {
                messages: prompt_messages,
                tools: provider_tools,
                model: std::env::var("NOSTRA_AGENT_MODEL")
                    .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            })
            .await
    }
}

/// 1. The Research Analyst
/// Ingests unstructured research initiatives and proposes formal Requirements/Briefs.
pub struct ResearchAnalyst;

impl CortexAgent for ResearchAnalyst {
    fn role_name(&self) -> &'static str {
        "ResearchAnalyst"
    }

    fn base_instructions(&self) -> &'static str {
        "You are the Research Analyst. Your goal is to ingest raw research markdown files and convert them into formal ContextNodes and Requirements in the Nostra Graph. You focus on synthesis and definition."
    }
}

/// 2. The Systems Architect
/// Translates Requirements into specific Graph Schema updates or Code Action Plans.
pub struct SystemsArchitect;

impl CortexAgent for SystemsArchitect {
    fn role_name(&self) -> &'static str {
        "SystemsArchitect"
    }

    fn base_instructions(&self) -> &'static str {
        "You are the Systems Architect. You translate Requirements into proposed ActionTargets for execution. You enforce the Constitutional Principles: meaning is sovereign, execution is interchangeable. You also propose new A2UI Standard Catalog items for deep simulation visualization."
    }
}

/// 3. The Simulation Evaluator
/// Takes ActionTargets and routes them through the GSMS (Governance Simulation Mode System) to deterministically bench test outcomes.
pub struct SimulationEvaluator;

impl CortexAgent for SimulationEvaluator {
    fn role_name(&self) -> &'static str {
        "SimulationEvaluator"
    }

    fn base_instructions(&self) -> &'static str {
        "You are the Simulation Evaluator. You receive proposed ActionTargets and route them through the GSMS. You evaluate the resulting SIQS scores and structural diffs. If there are violations, you reject the plan and provide feedback to the Architect."
    }
}

/// 4. The Process Steward
/// A highly trusted meta-agent or protocol actor representing the Human-in-the-loop review.
pub struct ProcessSteward;

impl CortexAgent for ProcessSteward {
    fn role_name(&self) -> &'static str {
        "ProcessSteward"
    }

    fn base_instructions(&self) -> &'static str {
        "You are the Process Steward. You hold the authority to approve Governance Proposals generated from successful deep simulation runs. You represent the human-in-the-loop interacting via the A2UI Space Workbench."
    }
}
