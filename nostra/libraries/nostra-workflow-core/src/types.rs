//! Core type definitions for the Workflow Engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a step within a workflow definition.
pub type StepId = String;

/// Unique identifier for a workflow definition.
pub type WorkflowId = String;

/// Unique identifier for a user/principal.
pub type UserId = String;

/// Unique identifier for a role.
pub type RoleId = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimulationReport {
    pub workflow_id: String,
    pub blocked_mutations: Vec<String>,
    pub simulated_state_changes: Vec<String>,
    pub confidence_score: f64,
}

impl SimulationReport {
    pub fn is_approved(&self, threshold: f64) -> bool {
        self.confidence_score >= threshold
    }
}

/// The current status of a workflow instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is actively executing.
    Running,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed with an error message.
    Failed(String),
    /// Workflow is paused, waiting for external input (e.g., UserTask).
    Paused,
}

/// Execution context passed between workflow steps.
///
/// The "Traveler Context" pattern (DEC-004): a key-value map that is passed
/// from step to step. Steps read from Context and write results back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// User-defined variables accessible to all steps.
    pub variables: HashMap<String, String>,
    /// Audit log of events during execution.
    pub history: Vec<String>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Append an event to the execution history.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.history.push(msg.into());
    }

    /// Get a variable value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Set a variable value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }
}

/// An immutable workflow definition template.
///
/// Per DEC-001: Strictly separate WorkflowDefinition (Immutable Template)
/// from WorkflowInstance (Stateful Execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique identifier for this workflow definition.
    pub id: WorkflowId,
    /// All steps in this workflow, keyed by StepId.
    pub steps: HashMap<StepId, crate::Step>,
    /// The ID of the step to start execution from.
    pub start_step_id: StepId,
}

/// A running instance of a workflow.
///
/// This holds the mutable state that must be persisted to survive upgrades.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Unique identifier for this instance.
    pub id: String,
    /// The workflow definition being executed (immutable reference pattern).
    pub definition: WorkflowDefinition,
    /// Current step ID, or None if completed/failed.
    pub current_step_id: Option<StepId>,
    /// Current execution status.
    pub status: WorkflowStatus,
    /// Execution context with variables and history.
    pub context: Context,
}

impl WorkflowInstance {
    /// Create a new workflow instance from a definition.
    pub fn new(id: impl Into<String>, definition: WorkflowDefinition) -> Self {
        let start_step = definition.start_step_id.clone();
        Self {
            id: id.into(),
            definition,
            current_step_id: Some(start_step),
            status: WorkflowStatus::Running,
            context: Context::new(),
        }
    }

    /// Check if the workflow is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            WorkflowStatus::Completed | WorkflowStatus::Failed(_)
        )
    }
}
