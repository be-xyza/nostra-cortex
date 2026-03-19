//! Saga Pattern Implementation
//!
//! Implements the Saga pattern for distributed transaction management:
//! - Compensation step tracking
//! - Automatic rollback on failure
//! - Compensation history logging

use crate::types::{StepId, WorkflowDefinition, WorkflowInstance, WorkflowStatus};

/// Saga execution state tracking.
#[derive(Debug, Clone)]
pub struct SagaState {
    /// Steps that have been successfully executed (in order).
    pub executed_steps: Vec<StepId>,
    /// Steps that have been compensated (in reverse order).
    pub compensated_steps: Vec<StepId>,
    /// Whether the saga is currently rolling back.
    pub is_rolling_back: bool,
}

impl Default for SagaState {
    fn default() -> Self {
        Self::new()
    }
}

impl SagaState {
    /// Create a new saga state.
    pub fn new() -> Self {
        Self {
            executed_steps: Vec::new(),
            compensated_steps: Vec::new(),
            is_rolling_back: false,
        }
    }

    /// Record a successfully executed step.
    pub fn record_execution(&mut self, step_id: StepId) {
        self.executed_steps.push(step_id);
    }

    /// Record a compensated step.
    pub fn record_compensation(&mut self, step_id: StepId) {
        self.compensated_steps.push(step_id);
    }

    /// Start rollback mode.
    pub fn start_rollback(&mut self) {
        self.is_rolling_back = true;
    }

    /// Check if all executed steps have been compensated.
    pub fn is_fully_compensated(&self) -> bool {
        self.executed_steps.len() == self.compensated_steps.len()
    }
}

/// Saga coordinator for managing compensations.
pub struct SagaCoordinator;

impl SagaCoordinator {
    /// Get the compensation chain for a failed workflow.
    ///
    /// Returns the list of compensation step IDs in reverse execution order.
    pub fn get_compensation_chain(
        definition: &WorkflowDefinition,
        executed_steps: &[StepId],
    ) -> Vec<StepId> {
        let mut compensations = Vec::new();

        // Iterate in reverse order (most recent first)
        for step_id in executed_steps.iter().rev() {
            if let Some(step) = definition.steps.get(step_id) {
                if let Some(compensation_id) = &step.compensated_by {
                    compensations.push(compensation_id.clone());
                }
            }
        }

        compensations
    }

    /// Execute compensation for a failed workflow.
    ///
    /// This runs the compensation steps in reverse order.
    pub fn compensate(
        instance: &mut WorkflowInstance,
        saga_state: &mut SagaState,
    ) -> CompensationResult {
        saga_state.start_rollback();

        let compensation_chain =
            Self::get_compensation_chain(&instance.definition, &saga_state.executed_steps);

        if compensation_chain.is_empty() {
            instance
                .context
                .log("No compensation steps defined".to_string());
            return CompensationResult::NoCompensation;
        }

        instance.context.log(format!(
            "Starting compensation: {} steps to rollback",
            compensation_chain.len()
        ));

        let mut failed_compensations = Vec::new();

        for comp_step_id in compensation_chain {
            match Self::execute_compensation_step(instance, &comp_step_id) {
                Ok(()) => {
                    saga_state.record_compensation(comp_step_id.clone());
                    instance
                        .context
                        .log(format!("Compensation successful: {}", comp_step_id));
                }
                Err(e) => {
                    instance
                        .context
                        .log(format!("Compensation failed for {}: {}", comp_step_id, e));
                    failed_compensations.push((comp_step_id, e));
                }
            }
        }

        if failed_compensations.is_empty() {
            instance.status = WorkflowStatus::Failed("Rolled back successfully".to_string());
            CompensationResult::FullyCompensated
        } else {
            instance.status = WorkflowStatus::Failed(format!(
                "Partial rollback: {} compensations failed",
                failed_compensations.len()
            ));
            CompensationResult::PartialCompensation(failed_compensations)
        }
    }

    /// Execute a single compensation step.
    fn execute_compensation_step(
        instance: &mut WorkflowInstance,
        step_id: &StepId,
    ) -> Result<(), String> {
        let step = instance
            .definition
            .steps
            .get(step_id)
            .ok_or_else(|| format!("Compensation step not found: {}", step_id))?;

        // Log the compensation action
        match &step.action {
            crate::primitives::Action::SystemOp { op_type, payload } => {
                instance.context.log(format!(
                    "Compensating with SystemOp: {} payload: {}",
                    op_type, payload
                ));
                // In production, this would call the actual compensation handler
                Ok(())
            }
            crate::primitives::Action::None => {
                // No-op compensation
                Ok(())
            }
            _ => {
                // UserTask and AsyncExternalOp can't be automatically compensated
                Err("Cannot auto-compensate interactive steps".to_string())
            }
        }
    }

    /// Mark a workflow as failed and trigger compensation.
    pub fn fail_with_compensation(
        instance: &mut WorkflowInstance,
        saga_state: &mut SagaState,
        reason: &str,
    ) -> CompensationResult {
        instance.context.log(format!(
            "Workflow failed: {}. Initiating compensation.",
            reason
        ));
        instance.status = WorkflowStatus::Failed(reason.to_string());
        Self::compensate(instance, saga_state)
    }
}

/// Result of a compensation attempt.
#[derive(Debug)]
pub enum CompensationResult {
    /// All executed steps were successfully compensated.
    FullyCompensated,
    /// Some compensation steps failed.
    PartialCompensation(Vec<(StepId, String)>),
    /// No compensation steps were defined.
    NoCompensation,
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Action, Transition};

    fn create_saga_workflow() -> WorkflowDefinition {
        let mut steps = HashMap::new();

        // Step 1: Debit account (with compensation: Credit account)
        let debit = Step {
            id: "debit".to_string(),
            description: "Debit source account".to_string(),
            action: Action::SystemOp {
                op_type: "Ledger.Debit".to_string(),
                payload: r#"{"amount": 100}"#.to_string(),
            },
            transitions: vec![Transition::to("credit")],
            compensated_by: Some("refund".to_string()),
        };

        // Step 2: Credit account (with compensation)
        let credit = Step {
            id: "credit".to_string(),
            description: "Credit destination account".to_string(),
            action: Action::SystemOp {
                op_type: "Ledger.Credit".to_string(),
                payload: r#"{"amount": 100}"#.to_string(),
            },
            transitions: vec![Transition::to("complete")],
            compensated_by: Some("reverse_credit".to_string()),
        };

        // Step 3: Complete
        let complete = Step {
            id: "complete".to_string(),
            description: "Transfer complete".to_string(),
            action: Action::None,
            transitions: vec![],
            compensated_by: None,
        };

        // Compensation: Refund
        let refund = Step {
            id: "refund".to_string(),
            description: "Refund debited amount".to_string(),
            action: Action::SystemOp {
                op_type: "Ledger.Credit".to_string(),
                payload: r#"{"amount": 100, "reason": "compensation"}"#.to_string(),
            },
            transitions: vec![],
            compensated_by: None,
        };

        // Compensation: Reverse credit
        let reverse_credit = Step {
            id: "reverse_credit".to_string(),
            description: "Reverse credited amount".to_string(),
            action: Action::SystemOp {
                op_type: "Ledger.Debit".to_string(),
                payload: r#"{"amount": 100, "reason": "compensation"}"#.to_string(),
            },
            transitions: vec![],
            compensated_by: None,
        };

        steps.insert("debit".to_string(), debit);
        steps.insert("credit".to_string(), credit);
        steps.insert("complete".to_string(), complete);
        steps.insert("refund".to_string(), refund);
        steps.insert("reverse_credit".to_string(), reverse_credit);

        WorkflowDefinition {
            id: "saga_transfer".to_string(),
            steps,
            start_step_id: "debit".to_string(),
        }
    }

    #[test]
    fn test_get_compensation_chain() {
        let def = create_saga_workflow();
        let executed = vec!["debit".to_string(), "credit".to_string()];

        let chain = SagaCoordinator::get_compensation_chain(&def, &executed);

        // Should be in reverse order: credit's compensation first, then debit's
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "reverse_credit");
        assert_eq!(chain[1], "refund");
    }

    #[test]
    fn test_saga_state_tracking() {
        let mut state = SagaState::new();

        state.record_execution("step1".to_string());
        state.record_execution("step2".to_string());
        assert_eq!(state.executed_steps.len(), 2);

        state.start_rollback();
        assert!(state.is_rolling_back);

        state.record_compensation("step2".to_string());
        state.record_compensation("step1".to_string());
        assert!(state.is_fully_compensated());
    }

    #[test]
    fn test_fail_with_compensation() {
        let def = create_saga_workflow();
        let mut instance = WorkflowInstance::new("saga-1", def);
        let mut saga_state = SagaState::new();

        // Simulate executing two steps
        saga_state.record_execution("debit".to_string());
        saga_state.record_execution("credit".to_string());

        // Now fail and compensate
        let result = SagaCoordinator::fail_with_compensation(
            &mut instance,
            &mut saga_state,
            "Downstream service unavailable",
        );

        assert!(matches!(result, CompensationResult::FullyCompensated));
        assert!(saga_state.is_rolling_back);
        assert_eq!(saga_state.compensated_steps.len(), 2);
    }
}
