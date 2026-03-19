//! Core traits for durable workflow execution.
//!
//! These traits define the contracts for workflow components,
//! enabling pluggable implementations (e.g., ICP-native vs test mocks).

use async_trait::async_trait;
use crate::types::{Context, StepId};

/// Result of an activity execution.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Activity completed successfully with optional output.
    Success(Option<String>),
    /// Activity failed with an error message.
    Failure(String),
    /// Activity needs to wait for external input (e.g., UserTask).
    Waiting(String),
    /// Activity completed and should transition to a specific step.
    TransitionTo(StepId),
}

/// Trait for durable activities.
///
/// Based on Temporal's Activity pattern: units of work that may fail,
/// timeout, or need compensation.
///
/// # Example
///
/// ```rust,ignore
/// #[async_trait]
/// impl DurableActivity for NotifyUser {
///     async fn execute(&self, ctx: &mut Context) -> ExecutionResult {
///         // Send notification...
///         ExecutionResult::Success(None)
///     }
///
///     fn compensation(&self) -> Option<Box<dyn DurableActivity>> {
///         // Return activity to undo the notification (if applicable)
///         None
///     }
/// }
/// ```
#[async_trait]
pub trait DurableActivity: Send + Sync {
    /// Execute the activity with the given context.
    async fn execute(&self, ctx: &mut Context) -> ExecutionResult;

    /// Return a compensation activity if this activity needs to be rolled back.
    /// Used for the Saga pattern (FR-12).
    fn compensation(&self) -> Option<Box<dyn DurableActivity>> {
        None
    }

    /// Maximum retry attempts before failing.
    fn max_retries(&self) -> u32 {
        3
    }

    /// Backoff strategy in milliseconds between retries.
    fn retry_backoff_ms(&self) -> Vec<u64> {
        vec![100, 500, 2000]
    }
}

/// Trait for the workflow execution engine.
///
/// Implementations must be deterministic: given the same inputs and history,
/// they must produce the same state transitions.
pub trait WorkflowExecutor {
    /// Advance the workflow by one step.
    ///
    /// Returns the next step ID, or None if the workflow is complete/paused.
    fn tick(&mut self) -> Option<StepId>;

    /// Complete a user task with the given input data.
    fn complete_user_task(&mut self, data: Option<std::collections::HashMap<String, String>>);

    /// Get the current step ID.
    fn current_step(&self) -> Option<&StepId>;

    /// Check if the workflow is waiting for external input.
    fn is_waiting(&self) -> bool;

    /// Get the execution context.
    fn context(&self) -> &Context;

    /// Get a mutable reference to the execution context.
    fn context_mut(&mut self) -> &mut Context;
}

/// Trait for persistence backends.
///
/// Implementations MUST store all state durably (DEC-003).
/// For ICP, this means `StableBTreeMap` in stable memory.
#[async_trait]
pub trait WorkflowStore: Send + Sync {
    /// Save a workflow instance.
    async fn save(&self, id: &str, state: &[u8]) -> anyhow::Result<()>;

    /// Load a workflow instance.
    async fn load(&self, id: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// List all workflow instance IDs.
    async fn list(&self) -> anyhow::Result<Vec<String>>;

    /// Delete a workflow instance.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
