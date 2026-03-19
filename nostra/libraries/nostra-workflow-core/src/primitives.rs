//! Workflow primitives: Actions, Steps, and Transitions.

use crate::types::{Context, RoleId, StepId, UserId};
use serde::{Deserialize, Serialize};

/// Retry strategy for async external operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AsyncRetryStrategy {
    ConstantDelay {
        #[serde(default)]
        delay_ms: u64,
    },
    ExponentialBackoff {
        #[serde(default)]
        delay_ms: u64,
        #[serde(default = "default_retry_multiplier")]
        multiplier: u32,
        #[serde(default = "default_retry_max_delay")]
        max_delay_ms: u64,
    },
}

impl Default for AsyncRetryStrategy {
    fn default() -> Self {
        Self::ConstantDelay { delay_ms: 0 }
    }
}

/// Retry policy for async external operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncRetryPolicy {
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default)]
    pub strategy: AsyncRetryStrategy,
}

impl AsyncRetryPolicy {
    /// Compute backoff delay for a zero-based retry index.
    pub fn backoff_ms(&self, retry_index: u32) -> u64 {
        match self.strategy {
            AsyncRetryStrategy::ConstantDelay { delay_ms } => delay_ms,
            AsyncRetryStrategy::ExponentialBackoff {
                delay_ms,
                multiplier,
                max_delay_ms,
            } => {
                let pow = multiplier.max(1).saturating_pow(retry_index);
                let next = delay_ms.saturating_mul(u64::from(pow));
                next.min(max_delay_ms)
            }
        }
    }
}

/// Provider selection strategy for async external operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AsyncProviderStrategy {
    Single,
    Fallback {
        #[serde(default)]
        targets: Vec<String>,
    },
    RoundRobin {
        #[serde(default)]
        targets: Vec<String>,
        #[serde(default)]
        start_index: usize,
    },
}

impl Default for AsyncProviderStrategy {
    fn default() -> Self {
        Self::Single
    }
}

fn default_retry_multiplier() -> u32 {
    2
}

fn default_retry_max_delay() -> u64 {
    u64::MAX
}

/// Action types that can be executed in a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// A manual task that waits for a user interaction.
    /// Uses A2UI protocol for dynamic form rendering (DEC-009).
    UserTask {
        /// Human-readable description of the task.
        description: String,
        /// Roles allowed to complete this task.
        candidate_roles: Vec<RoleId>,
        /// Specific users allowed to complete this task.
        candidate_users: Vec<UserId>,
        /// Optional A2UI Surface JSON schema for dynamic form rendering.
        /// When present, the frontend A2UIRenderer displays this instead of a generic form.
        /// See: research/028-a2ui-integration-feasibility
        #[serde(skip_serializing_if = "Option::is_none")]
        a2ui_schema: Option<String>,
    },
    /// An automated system operation (e.g., Graph.CreateNode, Ledger.Transfer).
    SystemOp {
        /// Operation type identifier.
        op_type: String,
        /// JSON payload for the operation.
        payload: String,
    },
    /// An async external operation (e.g., AI Agent call).
    /// Per 013 DEC-005: Uses AsyncExternalOp primitive.
    AsyncExternalOp {
        /// The agent or service to call.
        target: String,
        /// Input payload for the operation.
        input: String,
        /// Timeout in seconds.
        timeout_secs: u64,
        /// Retry policy controlling attempt budget and backoff behavior.
        #[serde(default)]
        retry_policy: AsyncRetryPolicy,
        /// Provider strategy controlling target selection order.
        #[serde(default)]
        provider_strategy: AsyncProviderStrategy,
    },
    /// A no-op or pass-through step.
    None,
}

/// A condition function type for transitions.
pub type ConditionFn = fn(&Context) -> bool;

/// A transition from one step to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// The target step to transition to.
    pub target_step_id: StepId,
    /// Optional condition that must be true for this transition.
    /// If None, the transition is unconditional.
    #[serde(skip)]
    pub condition: Option<ConditionFn>,
}

impl Transition {
    /// Create an unconditional transition to a target step.
    pub fn to(target: impl Into<StepId>) -> Self {
        Self {
            target_step_id: target.into(),
            condition: None,
        }
    }

    /// Add a condition to this transition.
    pub fn when(mut self, condition: ConditionFn) -> Self {
        self.condition = Some(condition);
        self
    }
}

/// A workflow step definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique identifier for this step.
    pub id: StepId,
    /// Human-readable description.
    pub description: String,
    /// The action to execute when this step is reached.
    pub action: Action,
    /// Possible transitions out of this step.
    pub transitions: Vec<Transition>,
    /// Optional compensation step ID for Saga pattern (FR-12).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensated_by: Option<StepId>,
}

impl Step {
    /// Create a new step with the given ID and description.
    pub fn new(id: impl Into<StepId>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            action: Action::None,
            transitions: Vec::new(),
            compensated_by: None,
        }
    }

    /// Set the action for this step.
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    /// Add a transition to this step.
    pub fn with_transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Set the compensation step for Saga pattern.
    pub fn compensate_with(mut self, step_id: impl Into<StepId>) -> Self {
        self.compensated_by = Some(step_id.into());
        self
    }
}

/// Module exports for workflow manifests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExport {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExport {
    pub id: String,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleExports {
    #[serde(default)]
    pub agents: Vec<AgentExport>,
    #[serde(default)]
    pub workflows: Vec<WorkflowExport>,
}

/// A workflow module manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowModule {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub exports: Option<ModuleExports>,
}
