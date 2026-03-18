use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::activities::reasoning::{
    ProviderExecutionTrace, ReasoningActivity, ReasoningInput, ReasoningOutput,
};
use crate::activities::simulation::{
    EvaluatePlanInput, EvaluatePlanOutput, EvaluateSimulationPlanActivity,
};
use crate::temporal::{ActivityOptions, MockWorkflowContext, Workflow};
use cortex_domain::simulation::feedback::ApprovalDecision;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionExecutionInput {
    pub contribution_id: String,
    pub space_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionExecutionOutput {
    pub simulation_passed: bool,
    pub siqs_score: f32,
    pub provider_trace: ProviderExecutionTrace,
}

pub struct ArchitectAndEvaluateWorkflow;

#[async_trait]
impl Workflow<ContributionExecutionInput, ContributionExecutionOutput>
    for ArchitectAndEvaluateWorkflow
{
    const NAME: &'static str = "ArchitectAndEvaluateWorkflow";

    async fn execute(
        &self,
        ctx: &MockWorkflowContext,
        input: ContributionExecutionInput,
    ) -> Result<ContributionExecutionOutput, String> {
        // 1. Invoke the reasoning activity with provider routing (OpenAI -> Anthropic -> fallback).
        let reasoning_input = ReasoningInput {
            contribution_id: input.contribution_id.clone(),
            space_id: input.space_id.clone(),
        };
        let reasoning_options = ActivityOptions {
            task_queue: "SIMULATION_TASK_QUEUE".into(),
        };
        let reasoning_result: ReasoningOutput = ctx
            .execute_activity(ReasoningActivity, reasoning_input, reasoning_options)
            .await?;

        // 2. Route the output into the GSMS Simulation Evaluator Activity
        let eval_input = EvaluatePlanInput {
            scenario_id: format!("sim-{}", input.contribution_id),
            action_targets_json: reasoning_result.action_targets_json.clone(),
        };

        let options = ActivityOptions {
            task_queue: "SIMULATION_TASK_QUEUE".into(),
        };

        let eval_result: EvaluatePlanOutput = ctx
            .execute_activity(EvaluateSimulationPlanActivity, eval_input, options)
            .await?;

        // 3. Pause for human review
        let scenario_id = format!("sim-{}", input.contribution_id);
        let approval_timeout_seconds = std::env::var("CORTEX_AGENT_APPROVAL_TIMEOUT_SECONDS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(3600)
            .max(1);
        let approval = ctx
            .wait_for_human_approval(
                &input.space_id,
                &scenario_id,
                Duration::from_secs(approval_timeout_seconds),
            )
            .await;
        let approval = approval.map_err(|err| format!("Approval gate failed: {err}"))?;

        if approval.decision != ApprovalDecision::Approved {
            return Err(format!(
                "Contribution rejected by human: {:?}",
                approval.rationale
            ));
        }

        // 4. Check SIQS and format the final workflow response
        Ok(ContributionExecutionOutput {
            simulation_passed: eval_result.success,
            siqs_score: eval_result.siqs_score,
            provider_trace: reasoning_result.provider_trace,
        })
    }
}
