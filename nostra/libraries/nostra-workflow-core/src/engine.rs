//! Workflow execution engine (FSM-based).
//!
//! Implements deterministic state machine execution per DEC-010.

use crate::primitives::{Action, AsyncProviderStrategy, AsyncRetryPolicy};
use crate::traits::WorkflowExecutor;
use crate::types::{Context, StepId, WorkflowInstance, WorkflowStatus};
use std::collections::HashMap;

/// The core workflow execution engine.
///
/// This is a stateless engine that operates on WorkflowInstance references.
/// Persistence is handled externally via the WorkflowStore trait.
pub struct Engine;

impl Engine {
    /// Advance the workflow instance based on inputs or auto-transitions.
    ///
    /// This is the main "tick" function that drives workflow execution.
    /// It processes the current step and transitions to the next if possible.
    pub fn step(instance: &mut WorkflowInstance) {
        // Don't process terminal states
        if instance.is_terminal() {
            return;
        }

        let current_step_id = match &instance.current_step_id {
            Some(id) => id.clone(),
            None => {
                instance.status = WorkflowStatus::Completed;
                return;
            }
        };

        let step = instance
            .definition
            .steps
            .get(&current_step_id)
            .expect("Step not found in definition");

        log::debug!("Processing Step: {}", step.id);

        let action = step.action.clone();

        // Execute Action
        match action {
            Action::SystemOp { op_type, payload } => {
                instance
                    .context
                    .log(format!("Executed SystemOp: {} with {}", op_type, payload));
                // In production, this dispatches to SystemPrimitive handlers
                Self::transition(instance);
            }
            Action::UserTask { description, .. } => {
                instance
                    .context
                    .log(format!("Waiting for User Task: {}", description));
                // UserTask pauses execution until complete_user_task is called
                instance.status = WorkflowStatus::Paused;
            }
            Action::AsyncExternalOp {
                target,
                input,
                timeout_secs,
                retry_policy,
                provider_strategy,
            } => {
                Self::dispatch_async_external_op(
                    instance,
                    &current_step_id,
                    &target,
                    &input,
                    timeout_secs,
                    &retry_policy,
                    &provider_strategy,
                );
            }
            Action::None => {
                Self::transition(instance);
            }
        }
    }

    /// Evaluate transitions from the current step and move to the next.
    fn transition(instance: &mut WorkflowInstance) {
        let current_step_id = instance.current_step_id.as_ref().unwrap().clone();
        let step = instance.definition.steps.get(&current_step_id).unwrap();

        // Try to find a valid transition
        for transition in &step.transitions {
            let should_transition = match transition.condition {
                Some(cond) => cond(&instance.context),
                None => true,
            };

            if should_transition {
                instance.context.log(format!(
                    "Transitioning from {} to {}",
                    current_step_id, transition.target_step_id
                ));
                instance.current_step_id = Some(transition.target_step_id.clone());
                instance.status = WorkflowStatus::Running;
                return;
            }
        }

        // No transition found
        if step.transitions.is_empty() {
            // End state - no outgoing transitions
            instance
                .context
                .log(format!("Workflow Completed at step {}", current_step_id));
            instance.status = WorkflowStatus::Completed;
            instance.current_step_id = None;
        } else {
            // Conditions exist but none met - stay in current step
            log::debug!(
                "Transition conditions not met, staying in {}",
                current_step_id
            );
        }
    }

    /// Complete a user task or async operation.
    ///
    /// This is called when external input is received to resume workflow execution.
    pub fn complete_user_task(
        instance: &mut WorkflowInstance,
        result_data: Option<HashMap<String, String>>,
    ) {
        if let Some(data) = result_data {
            instance.context.variables.extend(data);
        }

        // Resume execution and attempt transition
        instance.status = WorkflowStatus::Running;
        Self::transition(instance);
    }

    /// Get the A2UI schema for the current step, if it's a UserTask.
    ///
    /// Used by the frontend to render dynamic forms (DEC-009).
    pub fn get_a2ui_schema(instance: &WorkflowInstance) -> Option<&str> {
        let step_id = instance.current_step_id.as_ref()?;
        let step = instance.definition.steps.get(step_id)?;

        match &step.action {
            Action::UserTask { a2ui_schema, .. } => a2ui_schema.as_deref(),
            _ => None,
        }
    }

    fn dispatch_async_external_op(
        instance: &mut WorkflowInstance,
        step_id: &str,
        default_target: &str,
        input: &str,
        timeout_secs: u64,
        retry_policy: &AsyncRetryPolicy,
        provider_strategy: &AsyncProviderStrategy,
    ) {
        let attempts_key = format!("__async_total_attempts::{}", step_id);
        let attempts_done = parse_u32(instance.context.get(&attempts_key));

        let dispatch = select_dispatch_target(
            default_target,
            attempts_done,
            retry_policy,
            provider_strategy,
        );

        if let Some(err) = dispatch.error {
            instance.context.log(err.clone());
            instance.status = WorkflowStatus::Failed(err);
            return;
        }

        if let Some(target) = dispatch.target {
            instance.context.log(format!(
                "Dispatched AsyncOp to {} (strategy={}, attempt={}/{}, timeout_secs={}): {}",
                target,
                dispatch.strategy_name,
                dispatch.attempt_number,
                dispatch.attempt_budget,
                timeout_secs,
                input
            ));

            if let Some(backoff_ms) = dispatch.next_backoff_ms {
                instance.context.log(format!(
                    "AsyncOp retry scheduled for step {} after {}ms",
                    step_id, backoff_ms
                ));
            }

            instance
                .context
                .set(attempts_key, dispatch.attempt_number.to_string());
            instance.status = WorkflowStatus::Paused;
        }
    }
}

#[derive(Debug, Clone)]
struct DispatchDecision {
    strategy_name: &'static str,
    target: Option<String>,
    attempt_number: u32,
    attempt_budget: u32,
    next_backoff_ms: Option<u64>,
    error: Option<String>,
}

fn select_dispatch_target(
    default_target: &str,
    attempts_done: u32,
    retry_policy: &AsyncRetryPolicy,
    provider_strategy: &AsyncProviderStrategy,
) -> DispatchDecision {
    let attempts_per_provider = retry_policy.max_retries.saturating_add(1);

    match provider_strategy {
        AsyncProviderStrategy::Single => {
            if attempts_done >= attempts_per_provider {
                return exhausted(default_target, "single");
            }
            let attempt_number = attempts_done.saturating_add(1);
            let retry_index = attempts_done;
            DispatchDecision {
                strategy_name: "single",
                target: Some(default_target.to_string()),
                attempt_number,
                attempt_budget: attempts_per_provider,
                next_backoff_ms: (attempt_number <= retry_policy.max_retries)
                    .then(|| retry_policy.backoff_ms(retry_index)),
                error: None,
            }
        }
        AsyncProviderStrategy::Fallback { targets } => {
            let merged_targets = merge_targets(default_target, targets);
            let target_count = u32::try_from(merged_targets.len()).unwrap_or(u32::MAX);
            let total_budget = attempts_per_provider.saturating_mul(target_count.max(1));

            if attempts_done >= total_budget {
                return exhausted(default_target, "fallback");
            }

            let provider_index = attempts_done / attempts_per_provider;
            let provider_attempt_index = attempts_done % attempts_per_provider;
            let attempt_number = attempts_done.saturating_add(1);
            let target = merged_targets
                .get(provider_index as usize)
                .cloned()
                .unwrap_or_else(|| default_target.to_string());

            DispatchDecision {
                strategy_name: "fallback",
                target: Some(target),
                attempt_number,
                attempt_budget: total_budget,
                next_backoff_ms: (provider_attempt_index < retry_policy.max_retries)
                    .then(|| retry_policy.backoff_ms(provider_attempt_index)),
                error: None,
            }
        }
        AsyncProviderStrategy::RoundRobin {
            targets,
            start_index,
        } => {
            let merged_targets = merge_targets(default_target, targets);
            let total_budget = attempts_per_provider;

            if attempts_done >= total_budget {
                return exhausted(default_target, "round_robin");
            }

            let idx = (start_index.saturating_add(attempts_done as usize)) % merged_targets.len();
            let attempt_number = attempts_done.saturating_add(1);
            DispatchDecision {
                strategy_name: "round_robin",
                target: Some(merged_targets[idx].clone()),
                attempt_number,
                attempt_budget: total_budget,
                next_backoff_ms: (attempt_number <= retry_policy.max_retries)
                    .then(|| retry_policy.backoff_ms(attempts_done)),
                error: None,
            }
        }
    }
}

fn exhausted(default_target: &str, strategy_name: &'static str) -> DispatchDecision {
    DispatchDecision {
        strategy_name,
        target: None,
        attempt_number: 0,
        attempt_budget: 0,
        next_backoff_ms: None,
        error: Some(format!(
            "AsyncExternalOp retries exhausted for target '{}' using strategy '{}'",
            default_target, strategy_name
        )),
    }
}

fn merge_targets(default_target: &str, extras: &[String]) -> Vec<String> {
    let mut targets = Vec::with_capacity(extras.len().saturating_add(1));
    targets.push(default_target.to_string());

    for target in extras {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !targets.iter().any(|existing| existing == trimmed) {
            targets.push(trimmed.to_string());
        }
    }

    targets
}

fn parse_u32(value: Option<&String>) -> u32 {
    value.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0)
}

impl WorkflowExecutor for WorkflowInstance {
    fn tick(&mut self) -> Option<StepId> {
        Engine::step(self);
        self.current_step_id.clone()
    }

    fn complete_user_task(&mut self, data: Option<HashMap<String, String>>) {
        Engine::complete_user_task(self, data);
    }

    fn current_step(&self) -> Option<&StepId> {
        self.current_step_id.as_ref()
    }

    fn is_waiting(&self) -> bool {
        matches!(self.status, WorkflowStatus::Paused)
    }

    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{
        Action, AsyncProviderStrategy, AsyncRetryPolicy, AsyncRetryStrategy, Step, Transition,
    };
    use crate::types::WorkflowDefinition;

    fn create_simple_workflow() -> WorkflowDefinition {
        let mut steps = HashMap::new();

        let start = Step {
            id: "start".to_string(),
            description: "Start".to_string(),
            action: Action::None,
            transitions: vec![Transition::to("end")],
            compensated_by: None,
        };

        let end = Step {
            id: "end".to_string(),
            description: "End".to_string(),
            action: Action::None,
            transitions: vec![],
            compensated_by: None,
        };

        steps.insert("start".to_string(), start);
        steps.insert("end".to_string(), end);

        WorkflowDefinition {
            id: "test".to_string(),
            steps,
            start_step_id: "start".to_string(),
        }
    }

    #[test]
    fn test_simple_workflow_completes() {
        let def = create_simple_workflow();
        let mut instance = WorkflowInstance::new("inst-1", def);

        // First tick: start -> end
        Engine::step(&mut instance);
        assert_eq!(instance.current_step_id.as_deref(), Some("end"));

        // Second tick: end -> complete
        Engine::step(&mut instance);
        assert_eq!(instance.status, WorkflowStatus::Completed);
    }

    #[test]
    fn test_user_task_pauses() {
        let mut steps = HashMap::new();

        let start = Step {
            id: "start".to_string(),
            description: "Start".to_string(),
            action: Action::UserTask {
                description: "Do something".to_string(),
                candidate_roles: vec![],
                candidate_users: vec![],
                a2ui_schema: None,
            },
            transitions: vec![Transition::to("end")],
            compensated_by: None,
        };

        let end = Step {
            id: "end".to_string(),
            description: "End".to_string(),
            action: Action::None,
            transitions: vec![],
            compensated_by: None,
        };

        steps.insert("start".to_string(), start);
        steps.insert("end".to_string(), end);

        let def = WorkflowDefinition {
            id: "test".to_string(),
            steps,
            start_step_id: "start".to_string(),
        };

        let mut instance = WorkflowInstance::new("inst-1", def);

        // Tick should pause on UserTask
        Engine::step(&mut instance);
        assert_eq!(instance.status, WorkflowStatus::Paused);
        assert_eq!(instance.current_step_id.as_deref(), Some("start"));

        // Complete the task
        Engine::complete_user_task(&mut instance, None);
        assert_eq!(instance.current_step_id.as_deref(), Some("end"));
    }

    fn create_async_workflow(action: Action) -> WorkflowDefinition {
        let mut steps = HashMap::new();
        let dispatch = Step {
            id: "dispatch".to_string(),
            description: "Dispatch".to_string(),
            action,
            transitions: vec![Transition::to("done")],
            compensated_by: None,
        };
        let done = Step {
            id: "done".to_string(),
            description: "Done".to_string(),
            action: Action::None,
            transitions: vec![],
            compensated_by: None,
        };
        steps.insert("dispatch".to_string(), dispatch);
        steps.insert("done".to_string(), done);
        WorkflowDefinition {
            id: "async-test".to_string(),
            steps,
            start_step_id: "dispatch".to_string(),
        }
    }

    #[test]
    fn test_async_single_retries_then_fails() {
        let def = create_async_workflow(Action::AsyncExternalOp {
            target: "nostra.ai.primary".to_string(),
            input: "{\"task\":\"summarize\"}".to_string(),
            timeout_secs: 30,
            retry_policy: AsyncRetryPolicy {
                max_retries: 2,
                strategy: AsyncRetryStrategy::ConstantDelay { delay_ms: 100 },
            },
            provider_strategy: AsyncProviderStrategy::Single,
        });

        let mut instance = WorkflowInstance::new("inst-async-single", def);

        Engine::step(&mut instance);
        assert_eq!(instance.status, WorkflowStatus::Paused);
        Engine::step(&mut instance);
        assert_eq!(instance.status, WorkflowStatus::Paused);
        Engine::step(&mut instance);
        assert_eq!(instance.status, WorkflowStatus::Paused);

        Engine::step(&mut instance);
        assert!(matches!(instance.status, WorkflowStatus::Failed(_)));
    }

    #[test]
    fn test_async_fallback_advances_provider_after_retries() {
        let def = create_async_workflow(Action::AsyncExternalOp {
            target: "nostra.ai.primary".to_string(),
            input: "{\"task\":\"summarize\"}".to_string(),
            timeout_secs: 30,
            retry_policy: AsyncRetryPolicy {
                max_retries: 1,
                strategy: AsyncRetryStrategy::ConstantDelay { delay_ms: 50 },
            },
            provider_strategy: AsyncProviderStrategy::Fallback {
                targets: vec!["nostra.ai.backup".to_string()],
            },
        });

        let mut instance = WorkflowInstance::new("inst-async-fallback", def);

        Engine::step(&mut instance);
        Engine::step(&mut instance);
        Engine::step(&mut instance);

        let dispatch_logs: Vec<&String> = instance
            .context
            .history
            .iter()
            .filter(|entry| entry.starts_with("Dispatched AsyncOp to "))
            .collect();

        assert!(dispatch_logs[0].contains("nostra.ai.primary"));
        assert!(dispatch_logs[1].contains("nostra.ai.primary"));
        assert!(dispatch_logs[2].contains("nostra.ai.backup"));
    }

    #[test]
    fn test_async_round_robin_rotates_targets() {
        let def = create_async_workflow(Action::AsyncExternalOp {
            target: "nostra.ai.primary".to_string(),
            input: "{\"task\":\"summarize\"}".to_string(),
            timeout_secs: 30,
            retry_policy: AsyncRetryPolicy {
                max_retries: 2,
                strategy: AsyncRetryStrategy::ExponentialBackoff {
                    delay_ms: 10,
                    multiplier: 2,
                    max_delay_ms: 100,
                },
            },
            provider_strategy: AsyncProviderStrategy::RoundRobin {
                targets: vec![
                    "nostra.ai.backup-a".to_string(),
                    "nostra.ai.backup-b".to_string(),
                ],
                start_index: 0,
            },
        });

        let mut instance = WorkflowInstance::new("inst-async-rr", def);

        Engine::step(&mut instance);
        Engine::step(&mut instance);
        Engine::step(&mut instance);

        let dispatch_logs: Vec<&String> = instance
            .context
            .history
            .iter()
            .filter(|entry| entry.starts_with("Dispatched AsyncOp to "))
            .collect();

        assert!(dispatch_logs[0].contains("nostra.ai.primary"));
        assert!(dispatch_logs[1].contains("nostra.ai.backup-a"));
        assert!(dispatch_logs[2].contains("nostra.ai.backup-b"));
    }
}
